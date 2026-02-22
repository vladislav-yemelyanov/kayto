use crate::spec;

use super::reference_resolution::resolve_parameter_ref;
use super::{issue, try_parse_schema, ParseCtx, ParseIssue, ParsedParameter};

/// Builds a synthetic schema from parameter fields when inline `schema` is absent.
fn schema_from_parameter(param: &spec::MethodParams) -> Option<spec::Schema> {
    match param.schema.as_ref() {
        Some(schema) => Some(schema.clone()),
        None => match param.type_name.as_ref() {
            Some(type_name) => Some(spec::Schema {
                reference: None,
                type_name: Some(type_name.clone()),
                one_of: None,
                any_of: None,
                all_of: None,
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

/// Parses operation parameters into IR parameters with resolved schema types.
pub(crate) fn try_parse_parameters(
    openapi: &spec::OpenAPI,
    method: &spec::Method,
    issues: &mut Vec<ParseIssue>,
    ctx: ParseCtx<'_>,
) -> Option<Vec<ParsedParameter>> {
    if let Some(params) = &method.parameters {
        let mut parsed_params: Vec<ParsedParameter> = Vec::with_capacity(params.len());

        for param in params {
            let Some(param) = resolve_parameter_ref(openapi, param, issues, ctx) else {
                continue;
            };

            let Some(name) = param.name.as_ref() else {
                issue(issues, "parameters", ctx, "parameter name is missing");
                continue;
            };

            let schema = schema_from_parameter(&param);
            if schema.is_none() {
                issue(
                    issues,
                    "parameters",
                    ctx,
                    format!("parameter '{name}' has no schema/type"),
                );
            }

            let parameter_owner = format!("parameter '{name}'");
            let schema_type = schema.as_ref().and_then(|schema| {
                try_parse_schema(schema, issues, ctx, parameter_owner.as_str(), "$")
            });

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

    None
}
