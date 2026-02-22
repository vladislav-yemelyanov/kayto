use crate::spec;

use super::{try_parse_parameters, try_parse_response, try_parse_responses, ParseCtx, ParseIssue, Request};

/// Parses all operations for a single path into request IR entries.
pub(crate) fn parse_requests_for_path(
    openapi: &spec::OpenAPI,
    pathname: &str,
    methods: &spec::PathMethods,
    issues: &mut Vec<ParseIssue>,
) -> Result<Vec<Request>, String> {
    match methods {
        None => Err(format!("Methods not found: {}", pathname)),
        Some(methods) => {
            let mut reqs: Vec<Request> = vec![];

            for (variant, method) in methods {
                let method_name = variant.to_string();
                let method_ctx = ParseCtx::new(Some(pathname), Some(method_name.as_str()), None);

                let params = try_parse_parameters(openapi, method, issues, method_ctx);
                let body = try_parse_response(openapi, &method.request_body, issues, method_ctx)
                    .and_then(|body| {
                        if body.schema_type.is_none() && body.schema_name.is_none() {
                            None
                        } else {
                            Some(body)
                        }
                    });
                let responses = try_parse_responses(openapi, method, issues, method_ctx);

                reqs.push(Request {
                    path: pathname.to_string(),
                    method: method_name,
                    operation_id: method.operation_id.clone(),
                    params,
                    body,
                    responses,
                });
            }

            Ok(reqs)
        }
    }
}
