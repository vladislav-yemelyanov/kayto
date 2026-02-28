use crate::parser::{self, PrimitiveType, Request, SchemaType};
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};

use super::utils;

/// Converts parser IR schema nodes into TypeScript type expressions.
pub fn schema_to_ts(schema: &SchemaType) -> String {
    match schema {
        SchemaType::Primitive(p) => primitive_to_ts(p),
        SchemaType::Array(inner) => format!("Array<{}>", schema_to_ts(inner)),
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
                    (name.clone(), schema_to_ts(value), optional)
                })
                .collect();
            props.sort_by(|a, b| a.0.cmp(&b.0));

            let fields: Vec<String> = props
                .into_iter()
                .map(|(name, ty, optional)| {
                    let optional_suffix = if optional { "?" } else { "" };
                    format!("{}{}: {}", utils::ts_quote(&name), optional_suffix, ty)
                })
                .collect();

            utils::type_object(fields)
        }
        SchemaType::Ref(name) => format!("Schemas[{}]", utils::ts_quote(name)),
        SchemaType::OneOf(variants) | SchemaType::AnyOf(variants) => {
            if variants.is_empty() {
                "unknown".to_string()
            } else {
                variants
                    .iter()
                    .map(schema_to_ts)
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
                    .map(schema_to_ts)
                    .map(|v| format!("({v})"))
                    .collect::<Vec<_>>()
                    .join(" & ")
            }
        }
        SchemaType::Unknown => "unknown".to_string(),
    }
}

/// Converts operation parameters into a typed `params` object grouped by location.
pub fn params_to_ts(req: &Request) -> Option<String> {
    let params = req.params.as_ref()?;
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
                let optional_suffix = if param.required == Some(true) { "" } else { "?" };
                let ty = param
                    .schema_type
                    .as_ref()
                    .map(schema_to_ts)
                    .unwrap_or_else(|| "unknown".to_string());
                format!("{}{}: {}", utils::ts_quote(&param.name), optional_suffix, ty)
            })
            .collect();

        location_fields.push(format!(
            "{}: {}",
            utils::ts_quote(&location),
            utils::type_object(param_fields)
        ));
    }

    Some(utils::type_object(location_fields))
}

/// Converts operation responses into a typed status-code map.
pub fn responses_to_ts(req: &Request) -> Option<String> {
    let responses = req.responses.as_ref()?;
    if responses.is_empty() {
        return None;
    }

    let fields: Vec<String> = responses
        .into_iter()
        .map(|(status, parsed_response)| format!("{status}: {}", parsed_response_to_ts(parsed_response)))
        .collect();

    Some(utils::type_object(fields))
}

/// Converts a parsed response node into a TS type, preferring named schemas.
pub fn parsed_response_to_ts(parsed_response: &parser::ParsedResponse) -> String {
    if let Some(schema_name) = &parsed_response.schema_name {
        return format!("Schemas[{}]", utils::ts_quote(schema_name));
    }

    if let Some(schema_type) = parsed_response.schema_type.as_ref() {
        return schema_to_ts(schema_type);
    }

    "never".to_string()
}

/// Maps primitive schema metadata (including enum/nullable) to TS type syntax.
fn primitive_to_ts(primitive: &parser::Primitive) -> String {
    let mut base = if let Some(enum_values) = &primitive.enum_values {
        if enum_values.is_empty() {
            primitive_kind_to_ts(&primitive.kind).to_string()
        } else {
            let literals: Vec<String> = enum_values.iter().map(value_to_ts_literal).collect();
            literals.join(" | ")
        }
    } else {
        primitive_kind_to_ts(&primitive.kind).to_string()
    };

    if primitive.nullable == Some(true) {
        base.push_str(" | null");
    }

    base
}

/// Maps primitive kinds to the closest TypeScript scalar type.
fn primitive_kind_to_ts(kind: &PrimitiveType) -> &'static str {
    match kind {
        PrimitiveType::String => "string",
        PrimitiveType::Integer => "number",
        PrimitiveType::Number => "number",
        PrimitiveType::Boolean => "boolean",
    }
}

/// Converts JSON enum/default values to TS literal expressions.
fn value_to_ts_literal(value: &Value) -> String {
    match value {
        Value::String(s) => utils::ts_quote(s),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        _ => "unknown".to_string(),
    }
}

