use crate::parser::{self, PrimitiveType, Request, SchemaType};
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};

use super::format;

/// Converts parser IR schema nodes into TypeScript type expressions.
pub fn schema_type(schema: &SchemaType) -> String {
    match schema {
        SchemaType::Primitive(p) => primitive_type(p),
        SchemaType::Array(inner) => format!("Array<{}>", schema_type(inner)),
        SchemaType::Object(obj) => {
            let required: HashSet<String> = obj
                .required
                .as_ref()
                .map(|v| v.iter().cloned().collect())
                .unwrap_or_default();

            let mut props: Vec<(String, String, bool)> = obj
                .properties
                .iter()
                .map(|(name, value)| {
                    let optional = !required.contains(name);
                    (name.clone(), schema_type(value), optional)
                })
                .collect();
            props.sort_by(|a, b| a.0.cmp(&b.0));

            let fields: Vec<String> = props
                .into_iter()
                .map(|(name, ty, optional)| {
                    let optional_suffix = if optional { "?" } else { "" };
                    format!("{}{}: {}", format::quote(&name), optional_suffix, ty)
                })
                .collect();

            format::object_type(fields)
        }
        SchemaType::Ref(name) => format!("Schemas[{}]", format::quote(name)),
        SchemaType::OneOf(variants) | SchemaType::AnyOf(variants) => {
            if variants.is_empty() {
                "unknown".to_string()
            } else {
                variants
                    .iter()
                    .map(schema_type)
                    .map(|v| format!("({v})"))
                    .collect::<Vec<_>>()
                    .join(" | ")
            }
        }
        SchemaType::AllOf(variants) => {
            if variants.is_empty() {
                "unknown".to_string()
            } else {
                variants
                    .iter()
                    .map(schema_type)
                    .map(|v| format!("({v})"))
                    .collect::<Vec<_>>()
                    .join(" & ")
            }
        }
        SchemaType::Unknown => "unknown".to_string(),
    }
}

/// Converts operation parameters into a typed `params` object grouped by location.
pub fn params_type(req: &Request) -> Option<String> {
    let params = match req.params.as_ref() {
        Some(params) => params,
        None => return None,
    };
    if params.is_empty() {
        return None;
    }

    let mut by_location: BTreeMap<String, Vec<&parser::ParsedParameter>> = BTreeMap::new();
    for param in params {
        let location = param
            .location
            .as_deref()
            .unwrap_or("other")
            .to_ascii_lowercase();
        by_location.entry(location).or_default().push(param);
    }

    let mut location_fields = Vec::new();
    for (location, mut fields) in by_location {
        fields.sort_by(|a, b| a.name.cmp(&b.name));

        let param_fields: Vec<String> = fields
            .into_iter()
            .map(|param| {
                let optional_suffix = if param.required == Some(true) {
                    ""
                } else {
                    "?"
                };
                let ty = param
                    .schema_type
                    .as_ref()
                    .map(schema_type)
                    .unwrap_or_else(|| "unknown".to_string());
                format!("{}{}: {}", format::quote(&param.name), optional_suffix, ty)
            })
            .collect();

        location_fields.push(format!(
            "{}: {}",
            format::quote(&location),
            format::object_type(param_fields)
        ));
    }

    Some(format::object_type(location_fields))
}

/// Converts operation responses into a typed status-code map.
pub fn responses_type(req: &Request) -> Option<String> {
    let responses = match req.responses.as_ref() {
        Some(responses) => responses,
        None => return None,
    };
    if responses.is_empty() {
        return None;
    }

    let fields: Vec<String> = responses
        .into_iter()
        .map(|(status, parsed_response)| format!("{status}: {}", response_type(parsed_response)))
        .collect();

    Some(format::object_type(fields))
}

/// Converts a parsed response node into a TS type, preferring named schemas.
pub fn response_type(parsed_response: &parser::ParsedResponse) -> String {
    if let Some(schema_name) = &parsed_response.schema_name {
        return format!("Schemas[{}]", format::quote(schema_name));
    }

    if let Some(schema) = parsed_response.schema_type.as_ref() {
        return schema_type(schema);
    }

    "never".to_string()
}

/// Maps primitive schema metadata (including enum/nullable) to TS type syntax.
fn primitive_type(primitive: &parser::Primitive) -> String {
    let mut base = if let Some(enum_values) = &primitive.enum_values {
        if enum_values.is_empty() {
            primitive_scalar_type(&primitive.kind).to_string()
        } else {
            let literals: Vec<String> = enum_values.iter().map(value_literal).collect();
            literals.join(" | ")
        }
    } else {
        primitive_scalar_type(&primitive.kind).to_string()
    };

    if primitive.nullable == Some(true) {
        base.push_str(" | null");
    }

    base
}

/// Maps primitive kinds to the closest TypeScript scalar type.
fn primitive_scalar_type(kind: &PrimitiveType) -> &'static str {
    match kind {
        PrimitiveType::String => "string",
        PrimitiveType::Integer => "number",
        PrimitiveType::Number => "number",
        PrimitiveType::Boolean => "boolean",
    }
}

/// Converts JSON enum/default values to TS literal expressions.
fn value_literal(value: &Value) -> String {
    match value {
        Value::String(s) => format::quote(s),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        _ => "unknown".to_string(),
    }
}
