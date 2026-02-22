use crate::spec;
use std::collections::BTreeSet;

use super::{issue, ParseCtx, ParseIssue};

/// Extracts component key name from a `$ref` path.
pub(crate) fn get_schema_name_by_ref(reference: &str) -> Option<&str> {
    reference
        .split('/')
        .last()
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
}

/// Resolves schema `$ref` against components schemas/definitions.
pub(crate) fn get_schema_by_ref(openapi: &spec::OpenAPI, reference: &str) -> Option<spec::Schema> {
    let name = get_schema_name_by_ref(reference)?;
    let components = &openapi.components.as_ref()?;

    if let Some(schemas) = components.schemas.as_ref() {
        if let Some(schema_v3) = schemas.get(name) {
            return schema_v3.clone();
        }
    }

    let schema_v2 = components.definitions.as_ref()?.get(name);
    schema_v2.cloned().flatten()
}

/// Resolves parameter `$ref` from reusable parameter sections.
fn get_parameter_by_ref(openapi: &spec::OpenAPI, reference: &str) -> Option<spec::MethodParams> {
    let name = get_schema_name_by_ref(reference)?;

    if let Some(components) = openapi.components.as_ref() {
        if let Some(parameters) = components.parameters.as_ref() {
            if let Some(param) = parameters.get(name) {
                return param.clone();
            }
        }
    }

    if let Some(parameters) = openapi.parameters.as_ref() {
        if let Some(param) = parameters.get(name) {
            return param.clone();
        }
    }

    None
}

/// Resolves chained parameter references with cycle detection.
pub(crate) fn resolve_parameter_ref(
    openapi: &spec::OpenAPI,
    parameter: &spec::MethodParams,
    issues: &mut Vec<ParseIssue>,
    ctx: ParseCtx<'_>,
) -> Option<spec::MethodParams> {
    let mut visited = BTreeSet::new();
    resolve_parameter_ref_recursive(openapi, parameter.clone(), issues, ctx, &mut visited)
}

/// Recursive implementation for resolving parameter `$ref` chains.
fn resolve_parameter_ref_recursive(
    openapi: &spec::OpenAPI,
    parameter: spec::MethodParams,
    issues: &mut Vec<ParseIssue>,
    ctx: ParseCtx<'_>,
    visited: &mut BTreeSet<String>,
) -> Option<spec::MethodParams> {
    let Some(reference) = parameter.reference.clone() else {
        return Some(parameter);
    };

    if !visited.insert(reference.clone()) {
        issue(
            issues,
            "parameters.ref",
            ctx,
            format!("cyclic parameter $ref detected: '{reference}'"),
        );
        return None;
    }

    let Some(next_parameter) = get_parameter_by_ref(openapi, &reference) else {
        issue(
            issues,
            "parameters.ref",
            ctx,
            format!("parameter not found by $ref: '{reference}'"),
        );
        return None;
    };

    resolve_parameter_ref_recursive(openapi, next_parameter, issues, ctx, visited)
}
