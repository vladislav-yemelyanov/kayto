use crate::spec;
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
    pub enum_values: Option<Vec<String>>,
    // TODO: add descripiton, default, nullable, format
}

#[derive(Debug, Clone)]
pub enum SchemaType {
    Primitive(Primitive),
    Array(Box<SchemaType>),
    Object(HashMap<String, SchemaType>),
    Ref(String),
}

#[derive(Debug)]
pub struct ParsedResponse {
    schema_type: Option<SchemaType>,
    schema_name: Option<String>,
}

#[derive(Debug)]
pub struct Request {
    pub path: String,
    pub method: String,
    pub params: Option<HashMap<String, SchemaType>>,
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
    reference.split("/").last()
}

fn get_schema_by_ref<'a>(openapi: &spec::OpenAPI, reference: &'a str) -> Option<spec::Schema> {
    let name = get_schema_name_by_ref(reference)?;
    let components = &openapi.components.as_ref()?;

    let schema1 = components.schemas.get(name); // v3

    if let Some(schema1) = schema1 {
        return schema1.clone();
    }

    let schema2 = components.definitions.as_ref()?.get(name); // v2

    if let Some(schema2) = schema2 {
        return schema2.clone();
    }

    return None;
}

fn try_parse_schema(
    schema: &spec::Schema,
    issues: &mut Vec<ParseIssue>,
    ctx: ParseCtx<'_>,
) -> Option<SchemaType> {
    let type_name = schema.type_name.as_ref();

    if let Some(reference) = &schema.reference {
        let schema_name = get_schema_name_by_ref(&reference)?;
        return Some(SchemaType::Ref(schema_name.to_string()));
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

                return Some(SchemaType::Object(s));
            }
            issue(
                issues,
                "schema.object",
                ctx,
                "object schema has no properties",
            );
            return None;
        }
        spec::SchemaType::STRING => Some(SchemaType::Primitive(Primitive {
            kind: PrimitiveType::String,
            enum_values: None,
        })),
        spec::SchemaType::NUMBER => Some(SchemaType::Primitive(Primitive {
            kind: PrimitiveType::Number,
            enum_values: None,
        })),
        spec::SchemaType::INTEGER => Some(SchemaType::Primitive(Primitive {
            kind: PrimitiveType::Integer,
            enum_values: None,
        })),
        spec::SchemaType::BOOLEAN => Some(SchemaType::Primitive(Primitive {
            kind: PrimitiveType::Boolean,
            enum_values: None,
        })),
        _ => {
            issue(issues, "schema", ctx, "schema type is unsupported");
            None
        }
    }
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

    let Some(json) = content.json.as_ref() else {
        issue(
            issues,
            "response",
            ctx,
            "response content has no application/json schema",
        );
        return None;
    };

    let Some(schema) = json.schema.as_ref() else {
        issue(issues, "response", ctx, "response json has no schema");
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

        let Some(parsed_response) = try_parse_response(
            &openapi,
            &response,
            issues,
            status_ctx,
        ) else {
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
) -> Option<HashMap<String, SchemaType>> {
    if let Some(params) = &method.parameters {
        let mut map: HashMap<String, SchemaType> = HashMap::new();

        for param in params {
            let Some(schema) = &param.schema else {
                issue(issues, "parameters", ctx, "parameter has no schema");
                continue;
            };

            let Some(schema_type) = try_parse_schema(&schema, issues, ctx) else {
                issue(issues, "parameters", ctx, "parameter schema is unsupported");
                continue;
            };

            let name = param.name.as_ref();

            let Some(name) = name else {
                issue(issues, "parameters", ctx, "parameter name is missing");
                continue;
            };

            map.insert(name.to_string(), schema_type);
        }

        return Some(map);
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
                let body = try_parse_response(
                    &openapi,
                    &method.request_body,
                    issues,
                    method_ctx,
                );

                let responses = try_parse_responses(&openapi, &method, issues, method_ctx);

                let req = Request {
                    path: pathname.to_string(),
                    method: method_name,
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
        None => Err("Paths is not found".to_string()),
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
