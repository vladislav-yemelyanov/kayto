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

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a primitive schema IR node for concise conversion tests.
    fn primitive(kind: PrimitiveType) -> SchemaType {
        SchemaType::Primitive(parser::Primitive {
            kind,
            enum_values: None,
            description: None,
            default_value: None,
            nullable: None,
            format: None,
        })
    }

    /// Verifies combinators are currently rendered to `Object?` in Dart output.
    #[test]
    fn renders_object_for_any_of() {
        let schema = SchemaType::AnyOf(vec![
            primitive(PrimitiveType::String),
            primitive(PrimitiveType::Integer),
        ]);
        let identifiers = BTreeMap::new();
        assert_eq!(schema_to_dart(&schema, &identifiers), "Object?");
    }

    /// Verifies arrays preserve element type in Dart (`List<T>`).
    #[test]
    fn renders_list_for_array_schema() {
        let schema = SchemaType::Array(Box::new(primitive(PrimitiveType::Boolean)));
        let identifiers = BTreeMap::new();
        assert_eq!(schema_to_dart(&schema, &identifiers), "List<bool>");
    }

    /// Verifies unknown schemas are explicitly represented as `Object?`.
    #[test]
    fn renders_object_for_unknown_schema_type() {
        let identifiers = BTreeMap::new();
        assert_eq!(schema_to_dart(&SchemaType::Unknown, &identifiers), "Object?");
    }
}
