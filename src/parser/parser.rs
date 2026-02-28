use crate::spec;
use serde_json::Value;
use std::collections::BTreeMap;

mod diagnostics;
mod endpoint_requests;
mod reference_resolution;
mod request_parameters;
mod request_responses;
mod schema_mapping;

pub use diagnostics::ParseIssue;
pub(crate) use diagnostics::{issue, issue_with_code, ParseCtx};
pub(crate) use request_parameters::try_parse_parameters;
pub(crate) use request_responses::{try_parse_response, try_parse_responses};
pub(crate) use schema_mapping::try_parse_schema;

/// Primitive scalar categories used inside parser IR.
#[derive(Debug, Clone)]
pub enum PrimitiveType {
    String,
    Integer,
    Number,
    Boolean,
}

/// Parsed primitive schema metadata.
#[derive(Debug, Clone)]
pub struct Primitive {
    pub kind: PrimitiveType,
    pub enum_values: Option<Vec<Value>>,
    pub description: Option<String>,
    pub default_value: Option<Value>,
    pub nullable: Option<bool>,
    pub format: Option<String>,
}

/// Parsed object schema with deterministic property ordering.
#[derive(Debug, Clone)]
pub struct ObjectType {
    pub properties: BTreeMap<String, SchemaType>,
    pub required: Option<Vec<String>>,
}

/// Parsed request parameter ready for codegen backends.
#[derive(Debug)]
pub struct ParsedParameter {
    pub name: String,
    pub location: Option<String>,
    pub description: Option<String>,
    pub required: Option<bool>,
    pub schema_type: Option<SchemaType>,
}

/// Language-agnostic type model used as IR for code generation layers.
#[derive(Debug, Clone)]
pub enum SchemaType {
    Primitive(Primitive),
    Array(Box<SchemaType>),
    Object(ObjectType),
    Ref(String),
    OneOf(Vec<SchemaType>),
    AnyOf(Vec<SchemaType>),
    AllOf(Vec<SchemaType>),
    Unknown,
}

/// Parsed request/response schema payload with optional named model binding.
#[derive(Debug)]
pub struct ParsedResponse {
    pub schema_type: Option<SchemaType>,
    pub schema_name: Option<String>,
}

/// Intermediate representation (IR) for codegen modules (TypeScript, Dart, etc).
#[derive(Debug)]
pub struct Request {
    pub path: String,
    pub method: String,
    pub operation_id: Option<String>,
    pub params: Option<Vec<ParsedParameter>>,
    pub body: Option<ParsedResponse>,
    pub responses: Option<BTreeMap<u16, ParsedResponse>>,
}

/// Top-level parser output with generated IR and diagnostics.
#[derive(Debug)]
pub struct ParseOutput {
    pub requests: Vec<Request>,
    pub models: BTreeMap<String, SchemaType>,
    pub issues: Vec<ParseIssue>,
}

/// Parses reusable component schemas/definitions into IR model map.
fn parse_component_models(openapi: &spec::OpenAPI, issues: &mut Vec<ParseIssue>) -> BTreeMap<String, SchemaType> {
    let mut models: BTreeMap<String, SchemaType> = BTreeMap::new();

    let mut parse_component_group = |schemas: &BTreeMap<String, Option<spec::Schema>>, group_name: &str| {
        for (name, schema) in schemas {
            let Some(schema) = schema.as_ref() else {
                issue(
                    issues,
                    "schema.component",
                    ParseCtx::new(None, None, None),
                    format!("component schema '{name}' in '{group_name}' is empty"),
                );
                models.entry(name.clone()).or_insert(SchemaType::Unknown);
                continue;
            };

            let parsed = try_parse_schema(
                schema,
                issues,
                ParseCtx::new(None, None, None),
                "component schema",
                &format!("$.components.{group_name}.{name}"),
            )
            .unwrap_or(SchemaType::Unknown);

            models.entry(name.clone()).or_insert(parsed);
        }
    };

    if let Some(components) = openapi.components.as_ref() {
        if let Some(schemas) = components.schemas.as_ref() {
            parse_component_group(schemas, "schemas");
        }
        if let Some(definitions) = components.definitions.as_ref() {
            parse_component_group(definitions, "definitions");
        }
    }
    if let Some(definitions) = openapi.definitions.as_ref() {
        parse_component_group(definitions, "definitions");
    }

    models
}

/// Parses an OpenAPI document into request IR and diagnostics.
pub fn parse(openapi: &spec::OpenAPI) -> Result<ParseOutput, String> {
    let mut issues: Vec<ParseIssue> = vec![];
    let models = parse_component_models(openapi, &mut issues);
    let mut reqs: Vec<Request> = vec![];

    match &openapi.paths {
        None => Err("OpenAPI document has no 'paths' section".to_string()),
        Some(paths) => {
            for (pathname, methods) in paths {
                let path_reqs =
                    endpoint_requests::parse_requests_for_path(openapi, pathname, methods, &mut issues);

                match path_reqs {
                    Err(err) => issue(
                        &mut issues,
                        "path_methods",
                        ParseCtx::new(Some(pathname), None, None),
                        err,
                    ),
                    Ok(path_reqs) => reqs.extend(path_reqs),
                }
            }

            Ok(ParseOutput {
                requests: reqs,
                models,
                issues,
            })
        }
    }
}

