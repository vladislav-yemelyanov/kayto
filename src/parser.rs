use crate::spec;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum PrimitiveType {
    String,
    Integer,
    Number,
    Boolean,
}

#[derive(Debug, Clone)]
pub struct Primitive {
    pub kind: PrimitiveType,
    pub enum_values: Option<Vec<Value>>,
    pub description: Option<String>,
    pub default_value: Option<Value>,
    pub nullable: Option<bool>,
    pub format: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ObjectType {
    pub properties: HashMap<String, SchemaType>,
    pub required: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct ParsedParameter {
    pub name: String,
    pub location: Option<String>,
    pub description: Option<String>,
    pub required: Option<bool>,
    pub schema_type: Option<SchemaType>,
}

#[derive(Debug, Clone)]
/// Language-agnostic type model used as IR for code generation layers.
pub enum SchemaType {
    Primitive(Primitive),
    Array(Box<SchemaType>),
    Object(ObjectType),
    Ref(String),
}

#[derive(Debug)]
pub struct ParsedResponse {
    pub schema_type: Option<SchemaType>,
    pub schema_name: Option<String>,
}

#[derive(Debug)]
/// Intermediate representation (IR) for codegen modules (TypeScript, Dart, etc).
pub struct Request {
    pub path: String,
    pub method: String,
    pub operation_id: Option<String>,
    pub params: Option<Vec<ParsedParameter>>,
    pub body: Option<ParsedResponse>,
    pub responses: Option<HashMap<u16, ParsedResponse>>,
}

#[derive(Debug)]
pub struct ParseIssue {
    pub stage: &'static str,
    pub path: Option<String>,
    pub method: Option<String>,
    pub status: Option<String>,
    pub detail: String,
}

#[derive(Debug)]
pub struct ParseOutput {
    pub requests: Vec<Request>,
    pub issues: Vec<ParseIssue>,
}

#[derive(Debug, Clone, Copy, Default)]
struct ParseCtx<'a> {
    path: Option<&'a str>,
    method: Option<&'a str>,
    status: Option<&'a str>,
}

impl<'a> ParseCtx<'a> {
    fn new(path: Option<&'a str>, method: Option<&'a str>, status: Option<&'a str>) -> Self {
        Self {
            path,
            method,
            status,
        }
    }

    fn with_status(self, status: Option<&'a str>) -> Self {
        Self { status, ..self }
    }
}

fn issue(
    issues: &mut Vec<ParseIssue>,
    stage: &'static str,
    ctx: ParseCtx<'_>,
    detail: impl Into<String>,
) {
    issues.push(ParseIssue {
        stage,
        path: ctx.path.map(String::from),
        method: ctx.method.map(String::from),
        status: ctx.status.map(String::from),
        detail: detail.into(),
    });
}

fn get_schema_name_by_ref<'a>(reference: &'a str) -> Option<&'a str> {
    reference
        .split("/")
        .last()
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
}

fn get_schema_by_ref<'a>(openapi: &spec::OpenAPI, reference: &'a str) -> Option<spec::Schema> {
    let name = get_schema_name_by_ref(reference)?;
    let components = &openapi.components.as_ref()?;

    if let Some(schemas) = components.schemas.as_ref() {
        let schema_v3 = schemas.get(name);
        if let Some(schema_v3) = schema_v3 {
            return schema_v3.clone();
        }
    }

    let schema_v2 = components.definitions.as_ref()?.get(name);

    if let Some(schema_v2) = schema_v2 {
        return schema_v2.clone();
    }

    return None;
}

fn try_parse_schema(
    schema: &spec::Schema,
    issues: &mut Vec<ParseIssue>,
    ctx: ParseCtx<'_>,
) -> Option<SchemaType> {
    if let Some(reference) = &schema.reference {
        let Some(schema_name) = get_schema_name_by_ref(&reference) else {
            issue(
                issues,
                "schema.ref",
                ctx,
                format!("invalid $ref: '{reference}'"),
            );
            return None;
        };
        return Some(SchemaType::Ref(schema_name.to_string()));
    }

    let type_name = schema.type_name.as_ref();

    if type_name.is_none() {
        issue(
            issues,
            "schema",
            ctx,
            "schema has neither $ref nor explicit type",
        );
        return None;
    }

    fn to_primitive(kind: PrimitiveType, schema: &spec::Schema) -> SchemaType {
        SchemaType::Primitive(Primitive {
            kind,
            enum_values: schema.enum_variants.clone(),
            description: schema.description.clone(),
            default_value: schema.default_value.clone(),
            nullable: schema.nullable,
            format: schema.format.clone(),
        })
    }

    match type_name? {
        spec::SchemaType::ARRAY => {
            if let Some(items) = &schema.items {
                let schema_type = try_parse_schema(&items, issues, ctx)?;

                return Some(SchemaType::Array(Box::new(schema_type)));
            }
            issue(issues, "schema.array", ctx, "array schema has no items");
            return None;
        }
        spec::SchemaType::OBJECT => {
            if let Some(properties) = &schema.properties {
                let mut s = HashMap::new();

                for (key, value) in properties {
                    let schema = value.as_ref();

                    let Some(schema) = schema else {
                        issue(
                            issues,
                            "schema.object",
                            ctx,
                            format!("property '{key}' has no schema"),
                        );
                        continue;
                    };

                    let Some(schema_type) = try_parse_schema(&schema, issues, ctx) else {
                        issue(
                            issues,
                            "schema.object",
                            ctx,
                            format!("property '{key}' schema is unsupported"),
                        );
                        continue;
                    };

                    s.insert(key.to_string(), schema_type);
                }

                return Some(SchemaType::Object(ObjectType {
                    properties: s,
                    required: schema.required.clone(),
                }));
            }
            issue(
                issues,
                "schema.object",
                ctx,
                "object schema has no properties",
            );
            return None;
        }
        spec::SchemaType::STRING => Some(to_primitive(PrimitiveType::String, schema)),
        spec::SchemaType::NUMBER => Some(to_primitive(PrimitiveType::Number, schema)),
        spec::SchemaType::INTEGER => Some(to_primitive(PrimitiveType::Integer, schema)),
        spec::SchemaType::BOOLEAN => Some(to_primitive(PrimitiveType::Boolean, schema)),
        _ => {
            issue(issues, "schema", ctx, "schema type is unsupported");
            None
        }
    }
}

fn schema_from_parameter(param: &spec::MethodParams) -> Option<spec::Schema> {
    match param.schema.as_ref() {
        Some(schema) => Some(schema.clone()),
        None => match param.type_name.as_ref() {
            Some(type_name) => Some(spec::Schema {
                reference: None,
                type_name: Some(type_name.clone()),
                description: param.description.clone(),
                default_value: param.default_value.clone(),
                nullable: param.nullable,
                format: param.format.clone(),
                required: None,
                properties: None,
                enum_variants: param.enum_variants.clone(),
                items: param.items.clone(),
            }),
            None => None,
        },
    }
}

fn pick_content_schema<'a>(
    content: &'a spec::ResponseContent,
    issues: &mut Vec<ParseIssue>,
    ctx: ParseCtx<'_>,
) -> Option<&'a spec::Schema> {
    if content.media_types.is_empty() {
        issue(
            issues,
            "response",
            ctx,
            "response has no content media types",
        );
        return None;
    }

    let exact_json = content
        .media_types
        .get("application/json")
        .and_then(|c| c.schema.as_ref());

    if let Some(schema) = exact_json {
        return Some(schema);
    }

    let mut media_types_with_schema: Vec<(&str, &spec::Schema)> = content
        .media_types
        .iter()
        .filter_map(|(media_type, content)| {
            content.schema.as_ref().map(|s| (media_type.as_str(), s))
        })
        .collect();
    media_types_with_schema.sort_by(|a, b| a.0.cmp(b.0));

    if let Some((_, schema)) = media_types_with_schema
        .iter()
        .find(|(media_type, _)| media_type.contains("json"))
    {
        return Some(*schema);
    }

    if let Some((_, schema)) = media_types_with_schema.first() {
        return Some(*schema);
    }

    issue(
        issues,
        "response",
        ctx,
        "response content has no media type entries with schema",
    );
    None
}

fn try_parse_response(
    openapi: &spec::OpenAPI,
    response: &Option<spec::Response>,
    issues: &mut Vec<ParseIssue>,
    ctx: ParseCtx<'_>,
) -> Option<ParsedResponse> {
    let Some(response) = response.as_ref() else {
        return None;
    };

    let Some(content) = response.content.as_ref() else {
        issue(issues, "response", ctx, "response has no content");
        return None;
    };

    let Some(schema) = pick_content_schema(content, issues, ctx) else {
        return None;
    };

    match &schema.reference {
        None => Some(ParsedResponse {
            schema_type: try_parse_schema(schema, issues, ctx),
            schema_name: None,
        }),
        Some(reference) => {
            let Some(schema_name) = get_schema_name_by_ref(&reference) else {
                issue(
                    issues,
                    "response.ref",
                    ctx,
                    format!("invalid $ref: '{reference}'"),
                );
                return None;
            };

            let Some(schema) = get_schema_by_ref(&openapi, &reference) else {
                issue(
                    issues,
                    "response.ref",
                    ctx,
                    format!("schema not found by $ref: '{reference}'"),
                );
                return None;
            };

            let schema_type = try_parse_schema(&schema, issues, ctx);

            Some(ParsedResponse {
                schema_type: schema_type,
                schema_name: Some(schema_name.to_string()),
            })
        }
    }
}

fn try_parse_responses(
    openapi: &spec::OpenAPI,
    method: &spec::Method,
    issues: &mut Vec<ParseIssue>,
    ctx: ParseCtx<'_>,
) -> Option<HashMap<u16, ParsedResponse>> {
    let Some(responses) = &method.responses else {
        issue(issues, "response", ctx, "method has no responses");
        return None;
    };

    let mut map: HashMap<u16, ParsedResponse> = HashMap::new();

    for (status_code, response) in responses {
        let u = status_code.parse::<u16>();

        let Ok(u) = u else {
            issue(
                issues,
                "response.status",
                ctx.with_status(Some(status_code.as_str())),
                "status code is not a valid u16",
            );
            continue;
        };

        let status_ctx = ctx.with_status(Some(status_code.as_str()));

        let Some(parsed_response) = try_parse_response(&openapi, &response, issues, status_ctx)
        else {
            issue(issues, "response", status_ctx, "failed to parse response");
            continue;
        };

        map.insert(u, parsed_response);
    }

    return Some(map);
}

fn try_parse_parameters(
    method: &spec::Method,
    issues: &mut Vec<ParseIssue>,
    ctx: ParseCtx<'_>,
) -> Option<Vec<ParsedParameter>> {
    if let Some(params) = &method.parameters {
        let mut parsed_params: Vec<ParsedParameter> = Vec::with_capacity(params.len());

        for param in params {
            let Some(name) = param.name.as_ref() else {
                issue(issues, "parameters", ctx, "parameter name is missing");
                continue;
            };

            let schema = schema_from_parameter(param);

            if schema.is_none() {
                issue(
                    issues,
                    "parameters",
                    ctx,
                    format!("parameter '{name}' has no schema/type"),
                );
            }

            let schema_type = schema
                .as_ref()
                .and_then(|schema| try_parse_schema(schema, issues, ctx));

            if schema.is_some() && schema_type.is_none() {
                issue(
                    issues,
                    "parameters",
                    ctx,
                    format!("parameter '{name}' schema is unsupported"),
                );
            }

            parsed_params.push(ParsedParameter {
                name: name.to_string(),
                location: param.location.clone(),
                description: param.description.clone(),
                required: param.required,
                schema_type,
            });
        }

        return Some(parsed_params);
    }

    return None;
}

fn try_parse_path_methods(
    openapi: &spec::OpenAPI,
    pathname: &str,
    methods: &spec::PathMethods,
    issues: &mut Vec<ParseIssue>,
) -> Result<Vec<Request>, String> {
    match methods {
        None => Err(format!("Methods not found: {}", &pathname).to_string()),
        Some(methods) => {
            // path requests
            let mut reqs: Vec<Request> = vec![];

            for (variant, method) in methods {
                let method_name = variant.to_string();
                let method_name_str = method_name.as_str();
                let method_ctx = ParseCtx::new(Some(pathname), Some(method_name_str), None);

                let params = try_parse_parameters(&method, issues, method_ctx);
                let body = try_parse_response(&openapi, &method.request_body, issues, method_ctx);

                let responses = try_parse_responses(&openapi, &method, issues, method_ctx);

                let req = Request {
                    path: pathname.to_string(),
                    method: method_name,
                    operation_id: method.operation_id.clone(),
                    params: params,
                    body: body,
                    responses: responses,
                };

                reqs.push(req)
            }

            Ok(reqs)
        }
    }
}

pub fn parse(openapi: &spec::OpenAPI) -> Result<ParseOutput, String> {
    let mut issues: Vec<ParseIssue> = vec![];
    let mut reqs: Vec<Request> = vec![];

    match &openapi.paths {
        None => Err("OpenAPI document has no 'paths' section".to_string()),
        Some(paths) => {
            for (pathname, methods) in paths {
                let path_reqs = try_parse_path_methods(&openapi, &pathname, &methods, &mut issues);

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
                issues: issues,
            })
        }
    }
}
