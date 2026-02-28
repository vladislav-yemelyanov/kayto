use crate::parser::{self, PrimitiveType, SchemaType};
use std::collections::BTreeMap;

/// Converts parser IR schema nodes into Dart type expressions.
pub fn schema_to_dart(schema: &SchemaType, identifiers: &BTreeMap<String, String>) -> String {
    match schema {
        SchemaType::Primitive(p) => primitive_to_dart(p),
        SchemaType::Array(inner) => {
            let inner_type = schema_to_dart(inner, identifiers);
            format!("List<{inner_type}>")
        }
        SchemaType::Object(_) => "Map<String, Object?>".to_string(),
        SchemaType::Ref(name) => identifiers
            .get(name)
            .cloned()
            .unwrap_or_else(|| "Object?".to_string()),
        SchemaType::OneOf(_) | SchemaType::AnyOf(_) | SchemaType::AllOf(_) => "Object?".to_string(),
        SchemaType::Unknown => "Object?".to_string(),
    }
}

/// Converts a parsed response node into a Dart type expression string.
pub fn parsed_response_to_dart_type(
    parsed_response: &parser::ParsedResponse,
    identifiers: &BTreeMap<String, String>,
) -> String {
    if let Some(schema_name) = &parsed_response.schema_name {
        return identifiers
            .get(schema_name)
            .cloned()
            .unwrap_or_else(|| "Object?".to_string());
    }

    if let Some(schema_type) = parsed_response.schema_type.as_ref() {
        return schema_to_dart(schema_type, identifiers);
    }

    "Never".to_string()
}

/// Maps primitive schema metadata to the closest Dart scalar type.
fn primitive_to_dart(primitive: &parser::Primitive) -> String {
    let mut base = primitive_kind_to_dart(&primitive.kind).to_string();

    if primitive.nullable == Some(true) && !base.ends_with('?') {
        base.push('?');
    }

    base
}

/// Maps primitive kinds to Dart scalar types.
fn primitive_kind_to_dart(kind: &PrimitiveType) -> &'static str {
    match kind {
        PrimitiveType::String => "String",
        PrimitiveType::Integer => "int",
        PrimitiveType::Number => "double",
        PrimitiveType::Boolean => "bool",
    }
}

