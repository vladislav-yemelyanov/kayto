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

#[cfg(test)]
mod tests {
    use super::*;

    /// Parses OpenAPI JSON text into parser output for test scenarios.
    fn parse_json(input: &str) -> ParseOutput {
        let openapi: spec::OpenAPI = serde_json::from_str(input).expect("valid OpenAPI json");
        parse(&openapi).expect("parser should return output")
    }

    /// Ensures parameter `$ref` is resolved into a concrete parsed parameter.
    #[test]
    fn resolves_parameter_ref() {
        let parsed = parse_json(
            r##"{
              "paths": {
                "/repos": {
                  "get": {
                    "parameters": [
                      { "$ref": "#/components/parameters/org_param" }
                    ],
                    "responses": {
                      "200": {
                        "content": {
                          "application/json": {
                            "schema": { "type": "string" }
                          }
                        }
                      }
                    }
                  }
                }
              },
              "components": {
                "parameters": {
                  "org_param": {
                    "name": "org",
                    "in": "path",
                    "required": true,
                    "schema": { "type": "string" }
                  }
                }
              }
            }"##,
        );

        assert!(parsed.issues.is_empty());
        let req = parsed.requests.first().expect("one request");
        let params = req.params.as_ref().expect("params must be present");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "org");
    }

    /// Ensures cyclic parameter `$ref` chains produce a clear diagnostic.
    #[test]
    fn reports_cyclic_parameter_ref() {
        let parsed = parse_json(
            r##"{
              "paths": {
                "/repos": {
                  "get": {
                    "parameters": [
                      { "$ref": "#/components/parameters/a" }
                    ],
                    "responses": {
                      "200": {
                        "content": {
                          "application/json": {
                            "schema": { "type": "string" }
                          }
                        }
                      }
                    }
                  }
                }
              },
              "components": {
                "parameters": {
                  "a": { "$ref": "#/components/parameters/b" },
                  "b": { "$ref": "#/components/parameters/a" }
                }
              }
            }"##,
        );

        assert!(parsed
            .issues
            .iter()
            .any(|i| i.stage == "parameters.ref" && i.detail.contains("cyclic parameter $ref")));
    }

    /// Ensures body-less responses are accepted without parser diagnostics.
    #[test]
    fn accepts_response_without_content() {
        let parsed = parse_json(
            r##"{
              "paths": {
                "/ping": {
                  "get": {
                    "responses": {
                      "204": {}
                    }
                  }
                }
              }
            }"##,
        );

        assert!(parsed.issues.is_empty());
        let req = parsed.requests.first().expect("one request");
        let responses = req.responses.as_ref().expect("responses map");
        let r204 = responses.get(&204).expect("204 response");
        assert!(r204.schema_type.is_none());
        assert!(r204.schema_name.is_none());
    }

    /// Ensures empty schema nodes are represented as `SchemaType::Unknown`.
    #[test]
    fn maps_empty_schema_to_unknown() {
        let parsed = parse_json(
            r##"{
              "paths": {
                "/opaque": {
                  "get": {
                    "responses": {
                      "200": {
                        "content": {
                          "application/json": {
                            "schema": {}
                          }
                        }
                      }
                    }
                  }
                }
              }
            }"##,
        );

        assert!(parsed.issues.iter().any(|issue| {
            issue.code == Some("unknown_schema_missing_type_and_ref")
                && issue
                    .detail
                    .contains("is mapped to 'unknown' because it has neither '$ref' nor explicit 'type'")
        }));
        let req = parsed.requests.first().expect("one request");
        let responses = req.responses.as_ref().expect("responses map");
        let r200 = responses.get(&200).expect("200 response");
        match r200.schema_type.as_ref() {
            Some(SchemaType::Unknown) => {}
            other => panic!("expected Unknown schema, got: {:?}", other),
        }
    }

    /// Ensures `anyOf` response schemas are parsed into combinator IR variants.
    #[test]
    fn parses_any_of_schema() {
        let parsed = parse_json(
            r##"{
              "paths": {
                "/union": {
                  "get": {
                    "responses": {
                      "200": {
                        "content": {
                          "application/json": {
                            "schema": {
                              "anyOf": [
                                { "type": "string" },
                                { "type": "integer" }
                              ]
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }
            }"##,
        );

        assert!(parsed.issues.is_empty());
        let req = parsed.requests.first().expect("one request");
        let responses = req.responses.as_ref().expect("responses map");
        let r200 = responses.get(&200).expect("200 response");
        match r200.schema_type.as_ref() {
            Some(SchemaType::AnyOf(variants)) => assert_eq!(variants.len(), 2),
            other => panic!("expected AnyOf schema, got: {:?}", other),
        }
    }

    /// Ensures `oneOf` response schemas are parsed into combinator IR variants.
    #[test]
    fn parses_one_of_schema() {
        let parsed = parse_json(
            r##"{
              "paths": {
                "/one-of": {
                  "get": {
                    "responses": {
                      "200": {
                        "content": {
                          "application/json": {
                            "schema": {
                              "oneOf": [
                                { "type": "string" },
                                { "type": "integer" }
                              ]
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }
            }"##,
        );

        assert!(parsed.issues.is_empty());
        let req = parsed.requests.first().expect("one request");
        let responses = req.responses.as_ref().expect("responses map");
        let r200 = responses.get(&200).expect("200 response");
        match r200.schema_type.as_ref() {
            Some(SchemaType::OneOf(variants)) => assert_eq!(variants.len(), 2),
            other => panic!("expected OneOf schema, got: {:?}", other),
        }
    }

    /// Ensures `allOf` response schemas are parsed into combinator IR variants.
    #[test]
    fn parses_all_of_schema() {
        let parsed = parse_json(
            r##"{
              "paths": {
                "/all-of": {
                  "get": {
                    "responses": {
                      "200": {
                        "content": {
                          "application/json": {
                            "schema": {
                              "allOf": [
                                { "type": "object", "properties": { "id": { "type": "integer" } } },
                                { "type": "object", "properties": { "name": { "type": "string" } } }
                              ]
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }
            }"##,
        );

        assert!(parsed.issues.is_empty());
        let req = parsed.requests.first().expect("one request");
        let responses = req.responses.as_ref().expect("responses map");
        let r200 = responses.get(&200).expect("200 response");
        match r200.schema_type.as_ref() {
            Some(SchemaType::AllOf(variants)) => assert_eq!(variants.len(), 2),
            other => panic!("expected AllOf schema, got: {:?}", other),
        }
    }

    /// Ensures nested combinators inside array items are parsed recursively.
    #[test]
    fn parses_nested_combinator_in_array_items() {
        let parsed = parse_json(
            r##"{
              "paths": {
                "/nested": {
                  "get": {
                    "responses": {
                      "200": {
                        "content": {
                          "application/json": {
                            "schema": {
                              "type": "array",
                              "items": {
                                "anyOf": [
                                  { "type": "string" },
                                  { "type": "integer" }
                                ]
                              }
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }
            }"##,
        );

        assert!(parsed.issues.is_empty());
        let req = parsed.requests.first().expect("one request");
        let responses = req.responses.as_ref().expect("responses map");
        let r200 = responses.get(&200).expect("200 response");
        match r200.schema_type.as_ref() {
            Some(SchemaType::Array(inner)) => match inner.as_ref() {
                SchemaType::AnyOf(variants) => assert_eq!(variants.len(), 2),
                other => panic!("expected nested AnyOf schema, got: {:?}", other),
            },
            other => panic!("expected Array schema, got: {:?}", other),
        }
    }

    /// Ensures response `$ref` is resolved to named schema and parsed schema type.
    #[test]
    fn resolves_response_ref_schema() {
        let parsed = parse_json(
            r##"{
              "paths": {
                "/by-ref": {
                  "get": {
                    "responses": {
                      "200": {
                        "content": {
                          "application/json": {
                            "schema": { "$ref": "#/components/schemas/User" }
                          }
                        }
                      }
                    }
                  }
                }
              },
              "components": {
                "schemas": {
                  "User": {
                    "type": "object",
                    "properties": {
                      "id": { "type": "integer" }
                    }
                  }
                }
              }
            }"##,
        );

        assert!(parsed.issues.is_empty());
        let req = parsed.requests.first().expect("one request");
        let responses = req.responses.as_ref().expect("responses map");
        let r200 = responses.get(&200).expect("200 response");
        assert_eq!(r200.schema_name.as_deref(), Some("User"));
    }

    /// Ensures missing response `$ref` target is reported as `response.ref` diagnostic.
    #[test]
    fn reports_missing_response_ref_schema() {
        let parsed = parse_json(
            r##"{
              "paths": {
                "/broken-ref": {
                  "get": {
                    "responses": {
                      "200": {
                        "content": {
                          "application/json": {
                            "schema": { "$ref": "#/components/schemas/Missing" }
                          }
                        }
                      }
                    }
                  }
                }
              },
              "components": {
                "schemas": {
                  "User": { "type": "string" }
                }
              }
            }"##,
        );

        assert!(parsed
            .issues
            .iter()
            .any(|i| i.stage == "response.ref" && i.detail.contains("schema not found by $ref")));
    }

    /// Ensures `application/json` media type is preferred over other content types.
    #[test]
    fn prefers_application_json_media_type() {
        let parsed = parse_json(
            r##"{
              "paths": {
                "/media-priority": {
                  "get": {
                    "responses": {
                      "200": {
                        "content": {
                          "application/xml": {
                            "schema": { "type": "integer" }
                          },
                          "application/json": {
                            "schema": { "type": "string" }
                          }
                        }
                      }
                    }
                  }
                }
              }
            }"##,
        );

        assert!(parsed.issues.is_empty());
        let req = parsed.requests.first().expect("one request");
        let responses = req.responses.as_ref().expect("responses map");
        let r200 = responses.get(&200).expect("200 response");
        match r200.schema_type.as_ref() {
            Some(SchemaType::Primitive(p)) => match p.kind {
                PrimitiveType::String => {}
                _ => panic!("expected string schema from application/json"),
            },
            other => panic!("expected primitive schema, got: {:?}", other),
        }
    }

    /// Ensures `*+json` media types are preferred when exact `application/json` is absent.
    #[test]
    fn prefers_plus_json_media_type_when_json_absent() {
        let parsed = parse_json(
            r##"{
              "paths": {
                "/media-plus-json": {
                  "get": {
                    "responses": {
                      "200": {
                        "content": {
                          "application/xml": {
                            "schema": { "type": "integer" }
                          },
                          "application/problem+json": {
                            "schema": { "type": "string" }
                          }
                        }
                      }
                    }
                  }
                }
              }
            }"##,
        );

        assert!(parsed.issues.is_empty());
        let req = parsed.requests.first().expect("one request");
        let responses = req.responses.as_ref().expect("responses map");
        let r200 = responses.get(&200).expect("200 response");
        match r200.schema_type.as_ref() {
            Some(SchemaType::Primitive(p)) => match p.kind {
                PrimitiveType::String => {}
                _ => panic!("expected string schema from +json media type"),
            },
            other => panic!("expected primitive schema, got: {:?}", other),
        }
    }

    /// Ensures request body parser falls back to `application/*+json` media types.
    #[test]
    fn parses_request_body_from_plus_json_media_type() {
        let parsed = parse_json(
            r##"{
              "paths": {
                "/events": {
                  "post": {
                    "requestBody": {
                      "content": {
                        "application/vnd.api+json": {
                          "schema": { "type": "string" }
                        }
                      }
                    },
                    "responses": {
                      "200": {
                        "content": {
                          "application/json": {
                            "schema": { "type": "string" }
                          }
                        }
                      }
                    }
                  }
                }
              }
            }"##,
        );

        assert!(parsed.issues.is_empty());
        let req = parsed.requests.first().expect("one request");
        let body = req.body.as_ref().expect("body should be parsed");
        assert!(matches!(
            body.schema_type,
            Some(SchemaType::Primitive(Primitive {
                kind: PrimitiveType::String,
                ..
            }))
        ));
    }

    /// Ensures parameter `$ref` can be resolved from root-level `parameters` section.
    #[test]
    fn resolves_root_level_parameter_ref() {
        let parsed = parse_json(
            r##"{
              "paths": {
                "/users/{id}": {
                  "get": {
                    "parameters": [
                      { "$ref": "#/parameters/user_id" }
                    ],
                    "responses": {
                      "200": {
                        "content": {
                          "application/json": {
                            "schema": { "type": "string" }
                          }
                        }
                      }
                    }
                  }
                }
              },
              "parameters": {
                "user_id": {
                  "name": "id",
                  "in": "path",
                  "required": true,
                  "schema": { "type": "string" }
                }
              }
            }"##,
        );

        assert!(parsed.issues.is_empty());
        let req = parsed.requests.first().expect("one request");
        let params = req.params.as_ref().expect("params should be parsed");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "id");
        assert_eq!(params[0].location.as_deref(), Some("path"));
    }

    /// Ensures parameter diagnostics include missing-name errors.
    #[test]
    fn reports_parameter_without_name() {
        let parsed = parse_json(
            r##"{
              "paths": {
                "/broken-param": {
                  "get": {
                    "parameters": [
                      { "in": "query", "schema": { "type": "string" } }
                    ],
                    "responses": {
                      "200": {
                        "content": {
                          "application/json": { "schema": { "type": "string" } }
                        }
                      }
                    }
                  }
                }
              }
            }"##,
        );

        assert!(parsed
            .issues
            .iter()
            .any(|i| i.stage == "parameters" && i.detail.contains("parameter name is missing")));
    }

    /// Ensures parameter diagnostics include missing schema/type errors.
    #[test]
    fn reports_parameter_without_schema_or_type() {
        let parsed = parse_json(
            r##"{
              "paths": {
                "/no-schema": {
                  "get": {
                    "parameters": [
                      { "name": "q", "in": "query" }
                    ],
                    "responses": {
                      "200": {
                        "content": {
                          "application/json": { "schema": { "type": "string" } }
                        }
                      }
                    }
                  }
                }
              }
            }"##,
        );

        assert!(parsed.issues.iter().any(|i| {
            i.stage == "parameters" && i.detail.contains("parameter 'q' has no schema/type")
        }));
    }

    /// Ensures invalid status code keys are reported.
    #[test]
    fn reports_non_numeric_status_code() {
        let parsed = parse_json(
            r##"{
              "paths": {
                "/default-status": {
                  "get": {
                    "responses": {
                      "default": {
                        "content": {
                          "application/json": { "schema": { "type": "string" } }
                        }
                      }
                    }
                  }
                }
              }
            }"##,
        );

        assert!(parsed.issues.iter().any(|i| {
            i.stage == "response.status" && i.detail.contains("status code is not a valid u16")
        }));
    }

    /// Ensures invalid response references are reported in response.ref stage.
    #[test]
    fn reports_invalid_schema_ref() {
        let parsed = parse_json(
            r##"{
              "paths": {
                "/invalid-ref": {
                  "get": {
                    "responses": {
                      "200": {
                        "content": {
                          "application/json": { "schema": { "$ref": "#/" } }
                        }
                      }
                    }
                  }
                }
              }
            }"##,
        );

        assert!(parsed
            .issues
            .iter()
            .any(|i| i.stage == "response.ref" && i.detail.contains("invalid $ref")));
    }

    /// Ensures empty combinator lists are downgraded to unknown with explicit code.
    #[test]
    fn maps_empty_anyof_to_unknown_with_code() {
        let parsed = parse_json(
            r##"{
              "paths": {
                "/empty-anyof": {
                  "get": {
                    "responses": {
                      "200": {
                        "content": {
                          "application/json": { "schema": { "anyOf": [] } }
                        }
                      }
                    }
                  }
                }
              }
            }"##,
        );

        assert!(parsed
            .issues
            .iter()
            .any(|i| i.code == Some("unknown_anyof_unparseable")));
        let req = parsed.requests.first().expect("one request");
        let r200 = req
            .responses
            .as_ref()
            .expect("responses")
            .get(&200)
            .expect("200");
        assert!(matches!(r200.schema_type, Some(SchemaType::Unknown)));
    }

    /// Ensures arrays without items are downgraded to unknown with explicit code.
    #[test]
    fn maps_array_without_items_to_unknown_with_code() {
        let parsed = parse_json(
            r##"{
              "paths": {
                "/array-no-items": {
                  "get": {
                    "responses": {
                      "200": {
                        "content": {
                          "application/json": { "schema": { "type": "array" } }
                        }
                      }
                    }
                  }
                }
              }
            }"##,
        );

        assert!(parsed
            .issues
            .iter()
            .any(|i| i.code == Some("unknown_array_items_unparseable")));
    }

    /// Ensures object properties with null schema nodes emit schema diagnostics.
    #[test]
    fn reports_object_property_without_schema() {
        let parsed = parse_json(
            r##"{
              "paths": {
                "/null-property": {
                  "get": {
                    "responses": {
                      "200": {
                        "content": {
                          "application/json": {
                            "schema": {
                              "type": "object",
                              "properties": {
                                "id": null
                              }
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }
            }"##,
        );

        assert!(parsed
            .issues
            .iter()
            .any(|i| i.stage == "schema" && i.detail.contains("without a schema")));
    }

    /// Ensures unsupported schema types are downgraded to unknown with explicit code.
    #[test]
    fn maps_unsupported_type_to_unknown_with_code() {
        let parsed = parse_json(
            r##"{
              "paths": {
                "/null-type": {
                  "get": {
                    "responses": {
                      "200": {
                        "content": {
                          "application/json": { "schema": { "type": "null" } }
                        }
                      }
                    }
                  }
                }
              }
            }"##,
        );

        assert!(parsed
            .issues
            .iter()
            .any(|i| i.code == Some("unknown_schema_type_not_supported")));
        let req = parsed.requests.first().expect("one request");
        let r200 = req
            .responses
            .as_ref()
            .expect("responses")
            .get(&200)
            .expect("200");
        assert!(matches!(r200.schema_type, Some(SchemaType::Unknown)));
    }

    /// Ensures Swagger 2.0 `type: file` is parsed as a binary string primitive.
    #[test]
    fn maps_file_type_to_binary_string() {
        let parsed = parse_json(
            r##"{
              "paths": {
                "/upload": {
                  "post": {
                    "responses": {
                      "200": {
                        "content": {
                          "application/json": { "schema": { "type": "file" } }
                        }
                      }
                    }
                  }
                }
              }
            }"##,
        );

        assert!(!parsed
            .issues
            .iter()
            .any(|i| i.code == Some("unknown_schema_type_not_supported")));

        let req = parsed.requests.first().expect("one request");
        let r200 = req
            .responses
            .as_ref()
            .expect("responses")
            .get(&200)
            .expect("200");

        let Some(SchemaType::Primitive(primitive)) = r200.schema_type.as_ref() else {
            panic!("expected primitive schema");
        };

        assert!(matches!(primitive.kind, PrimitiveType::String));
        assert_eq!(primitive.format.as_deref(), Some("binary"));
    }
}
