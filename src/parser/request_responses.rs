use crate::spec;
use std::collections::BTreeMap;

use super::reference_resolution::{get_schema_by_ref, get_schema_name_by_ref};
use super::{issue, try_parse_schema, ParseCtx, ParseIssue, ParsedResponse};

/// Selects the most suitable schema from response content media types.
fn pick_content_schema(content: &spec::ResponseContent) -> Option<&spec::Schema> {
    if content.media_types.is_empty() {
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

    None
}

/// Creates an empty parsed response marker for body-less responses.
fn empty_parsed_response() -> ParsedResponse {
    ParsedResponse {
        schema_type: None,
        schema_name: None,
    }
}

/// Parses a single response object into IR response payload metadata.
pub(crate) fn try_parse_response(
    openapi: &spec::OpenAPI,
    response: &Option<spec::Response>,
    issues: &mut Vec<ParseIssue>,
    ctx: ParseCtx<'_>,
) -> Option<ParsedResponse> {
    let Some(response) = response.as_ref() else {
        return None;
    };

    let Some(content) = response.content.as_ref() else {
        return Some(empty_parsed_response());
    };

    let Some(schema) = pick_content_schema(content) else {
        return Some(empty_parsed_response());
    };

    match &schema.reference {
        None => Some(ParsedResponse {
            schema_type: try_parse_schema(schema, issues, ctx, "response body", "$"),
            schema_name: None,
        }),
        Some(reference) => {
            let Some(schema_name) = get_schema_name_by_ref(reference) else {
                issue(
                    issues,
                    "response.ref",
                    ctx,
                    format!("invalid $ref: '{reference}'"),
                );
                return Some(empty_parsed_response());
            };

            let Some(schema) = get_schema_by_ref(openapi, reference) else {
                issue(
                    issues,
                    "response.ref",
                    ctx,
                    format!("schema not found by $ref: '{reference}'"),
                );
                return Some(empty_parsed_response());
            };

            let schema_type = try_parse_schema(&schema, issues, ctx, "response body", "$");

            Some(ParsedResponse {
                schema_type,
                schema_name: Some(schema_name.to_string()),
            })
        }
    }
}

/// Parses and normalizes all operation responses keyed by numeric status code.
pub(crate) fn try_parse_responses(
    openapi: &spec::OpenAPI,
    method: &spec::Method,
    issues: &mut Vec<ParseIssue>,
    ctx: ParseCtx<'_>,
) -> Option<BTreeMap<u16, ParsedResponse>> {
    let Some(responses) = &method.responses else {
        issue(issues, "response", ctx, "method has no responses");
        return None;
    };

    let mut map: BTreeMap<u16, ParsedResponse> = BTreeMap::new();

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
        let parsed_response =
            try_parse_response(openapi, response, issues, status_ctx).unwrap_or_else(empty_parsed_response);

        map.insert(u, parsed_response);
    }

    Some(map)
}
