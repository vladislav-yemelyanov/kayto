use crate::spec;
use std::collections::BTreeMap;

use super::reference_resolution::get_schema_name_by_ref;
use super::{
    issue, issue_with_code, ObjectType, ParseCtx, ParseIssue, Primitive, PrimitiveType, SchemaType,
};

/// Produces a concise human-readable explanation for unsupported schema shapes.
fn explain_schema_failure(schema: &spec::Schema) -> String {
    if let Some(any_of) = schema.any_of.as_ref() {
        if !any_of.is_empty() {
            return format!(
                "it uses 'anyOf' with {} variants, but at least one variant could not be parsed",
                any_of.len()
            );
        }
    }

    if let Some(one_of) = schema.one_of.as_ref() {
        if !one_of.is_empty() {
            return format!(
                "it uses 'oneOf' with {} variants, but at least one variant could not be parsed",
                one_of.len()
            );
        }
    }

    if let Some(all_of) = schema.all_of.as_ref() {
        if !all_of.is_empty() {
            return format!(
                "it uses 'allOf' with {} variants, but at least one variant could not be parsed",
                all_of.len()
            );
        }
    }

    if matches!(schema.type_name, Some(spec::SchemaType::ARRAY)) {
        if let Some(items) = schema.items.as_ref() {
            return format!(
                "it is an array, but its 'items' schema cannot be parsed because {}",
                explain_schema_failure(items)
            );
        }
        return "it is an array but does not define 'items'".to_string();
    }

    if schema.type_name.is_none() && schema.reference.is_none() {
        return "it has neither '$ref' nor explicit 'type'".to_string();
    }

    if let Some(unsupported_type) = schema.type_name.as_ref() {
        return format!("its type '{unsupported_type:?}' is not supported yet");
    }

    "its structure is not supported by the current parser".to_string()
}

/// Parses `oneOf`/`anyOf`/`allOf` variant lists into IR schema variants.
fn parse_combinator_variants(
    kind: &'static str,
    variants: &[spec::Schema],
    issues: &mut Vec<ParseIssue>,
    ctx: ParseCtx<'_>,
    schema_owner: &str,
    schema_path: &str,
) -> Option<Vec<SchemaType>> {
    if variants.is_empty() {
        issue(
            issues,
            "schema",
            ctx,
            format!("{schema_owner} schema at '{schema_path}' has empty '{kind}' list"),
        );
        return None;
    }

    let mut parsed_variants: Vec<SchemaType> = Vec::with_capacity(variants.len());
    for (idx, variant) in variants.iter().enumerate() {
        let variant_path = format!("{schema_path}.{kind}[{idx}]");
        let Some(parsed_variant) =
            try_parse_schema(variant, issues, ctx, schema_owner, &variant_path)
        else {
            issue(
                issues,
                "schema",
                ctx,
                format!(
                    "{schema_owner} schema at '{variant_path}' could not be parsed as part of '{kind}'"
                ),
            );
            continue;
        };
        parsed_variants.push(parsed_variant);
    }

    if parsed_variants.is_empty() {
        issue(
            issues,
            "schema",
            ctx,
            format!(
                "{schema_owner} schema at '{schema_path}' has '{kind}', but none of its variants could be parsed"
            ),
        );
        return None;
    }

    Some(parsed_variants)
}

/// Parses an array schema and resolves its `items` type recursively.
fn parse_array_schema(
    schema: &spec::Schema,
    issues: &mut Vec<ParseIssue>,
    ctx: ParseCtx<'_>,
    schema_owner: &str,
    schema_path: &str,
) -> Option<SchemaType> {
    let Some(items) = schema.items.as_ref() else {
        issue(
            issues,
            "schema",
            ctx,
            format!("{schema_owner} schema at '{schema_path}' is an array but has no 'items'"),
        );
        return None;
    };

    let items_path = format!("{schema_path}.items");
    let parsed = try_parse_schema(items, issues, ctx, schema_owner, &items_path)?;
    Some(SchemaType::Array(Box::new(parsed)))
}

/// Parses an object schema and maps each property schema recursively.
fn parse_object_schema(
    schema: &spec::Schema,
    issues: &mut Vec<ParseIssue>,
    ctx: ParseCtx<'_>,
    schema_owner: &str,
    schema_path: &str,
) -> Option<SchemaType> {
    let Some(properties) = schema.properties.as_ref() else {
        return Some(SchemaType::Object(ObjectType {
            properties: BTreeMap::new(),
            required: schema.required.clone(),
        }));
    };

    let mut parsed_properties = BTreeMap::new();
    for (key, value) in properties {
        let Some(property_schema) = value.as_ref() else {
            issue(
                issues,
                "schema",
                ctx,
                format!(
                    "{schema_owner} schema has property '{key}' at '{schema_path}.properties.{key}' without a schema"
                ),
            );
            continue;
        };

        let property_path = format!("{schema_path}.properties.{key}");
        let Some(parsed_property) =
            try_parse_schema(property_schema, issues, ctx, schema_owner, &property_path)
        else {
            issue(
                issues,
                "schema",
                ctx,
                format!(
                    "{schema_owner} schema has unsupported property '{key}' at '{property_path}': {}",
                    explain_schema_failure(property_schema)
                ),
            );
            continue;
        };

        parsed_properties.insert(key.to_string(), parsed_property);
    }

    Some(SchemaType::Object(ObjectType {
        properties: parsed_properties,
        required: schema.required.clone(),
    }))
}

/// Converts a raw OpenAPI schema node into parser IR schema representation.
pub(crate) fn try_parse_schema(
    schema: &spec::Schema,
    issues: &mut Vec<ParseIssue>,
    ctx: ParseCtx<'_>,
    schema_owner: &str,
    schema_path: &str,
) -> Option<SchemaType> {
    if let Some(reference) = &schema.reference {
        let Some(schema_name) = get_schema_name_by_ref(reference) else {
            issue(
                issues,
                "schema.ref",
                ctx,
                format!(
                    "{schema_owner} schema at '{schema_path}' has invalid $ref: '{reference}'"
                ),
            );
            return None;
        };
        return Some(SchemaType::Ref(schema_name.to_string()));
    }

    let Some(type_name) = schema.type_name.as_ref() else {
        if let Some(any_of) = schema.any_of.as_ref() {
            let parsed = parse_combinator_variants(
                "anyOf",
                any_of,
                issues,
                ctx,
                schema_owner,
                schema_path,
            );
            if let Some(parsed) = parsed {
                return Some(SchemaType::AnyOf(parsed));
            }

            issue_with_code(
                issues,
                "schema",
                Some("unknown_anyof_unparseable"),
                ctx,
                format!(
                    "{schema_owner} schema at '{schema_path}' is mapped to 'unknown' because 'anyOf' variants could not be parsed"
                ),
            );
            return Some(SchemaType::Unknown);
        }

        if let Some(one_of) = schema.one_of.as_ref() {
            let parsed = parse_combinator_variants(
                "oneOf",
                one_of,
                issues,
                ctx,
                schema_owner,
                schema_path,
            );
            if let Some(parsed) = parsed {
                return Some(SchemaType::OneOf(parsed));
            }

            issue_with_code(
                issues,
                "schema",
                Some("unknown_oneof_unparseable"),
                ctx,
                format!(
                    "{schema_owner} schema at '{schema_path}' is mapped to 'unknown' because 'oneOf' variants could not be parsed"
                ),
            );
            return Some(SchemaType::Unknown);
        }

        if let Some(all_of) = schema.all_of.as_ref() {
            let parsed = parse_combinator_variants(
                "allOf",
                all_of,
                issues,
                ctx,
                schema_owner,
                schema_path,
            );
            if let Some(parsed) = parsed {
                return Some(SchemaType::AllOf(parsed));
            }

            issue_with_code(
                issues,
                "schema",
                Some("unknown_allof_unparseable"),
                ctx,
                format!(
                    "{schema_owner} schema at '{schema_path}' is mapped to 'unknown' because 'allOf' variants could not be parsed"
                ),
            );
            return Some(SchemaType::Unknown);
        }

        // OpenAPI nodes without `type`/`$ref` are treated as unknown to preserve parsing continuity.
        issue_with_code(
            issues,
            "schema",
            Some("unknown_schema_missing_type_and_ref"),
            ctx,
            format!(
                "{schema_owner} schema at '{schema_path}' is mapped to 'unknown' because it has neither '$ref' nor explicit 'type'"
            ),
        );
        return Some(SchemaType::Unknown);
    };

    /// Converts a primitive OpenAPI schema into primitive IR representation.
    fn to_primitive(kind: PrimitiveType, schema: &spec::Schema) -> SchemaType {
        SchemaType::Primitive(Primitive {
            kind,
            enum_values: schema.enum_variants.clone(),
            description: schema.description.clone(),
            default_value: schema.default_value.clone(),
            nullable: schema.nullable,
            format: schema.format.clone(),
        })
    }

    /// Swagger 2.0 `type: file` is represented as a binary string for generators.
    fn to_file_primitive(schema: &spec::Schema) -> SchemaType {
        SchemaType::Primitive(Primitive {
            kind: PrimitiveType::String,
            enum_values: schema.enum_variants.clone(),
            description: schema.description.clone(),
            default_value: schema.default_value.clone(),
            nullable: schema.nullable,
            format: schema
                .format
                .clone()
                .or_else(|| Some("binary".to_string())),
        })
    }

    match type_name {
        spec::SchemaType::ARRAY => {
            let parsed = parse_array_schema(schema, issues, ctx, schema_owner, schema_path);
            if parsed.is_some() {
                return parsed;
            }

            issue_with_code(
                issues,
                "schema",
                Some("unknown_array_items_unparseable"),
                ctx,
                format!(
                    "{schema_owner} schema at '{schema_path}' is mapped to 'unknown' because array 'items' could not be parsed"
                ),
            );
            Some(SchemaType::Unknown)
        }
        spec::SchemaType::OBJECT => parse_object_schema(schema, issues, ctx, schema_owner, schema_path),
        spec::SchemaType::STRING => Some(to_primitive(PrimitiveType::String, schema)),
        spec::SchemaType::NUMBER => Some(to_primitive(PrimitiveType::Number, schema)),
        spec::SchemaType::INTEGER => Some(to_primitive(PrimitiveType::Integer, schema)),
        spec::SchemaType::BOOLEAN => Some(to_primitive(PrimitiveType::Boolean, schema)),
        spec::SchemaType::FILE => Some(to_file_primitive(schema)),
        _ => {
            issue_with_code(
                issues,
                "schema",
                Some("unknown_schema_type_not_supported"),
                ctx,
                format!(
                    "{schema_owner} schema at '{schema_path}' is mapped to 'unknown' because its type is not supported yet: {}",
                    explain_schema_failure(schema)
                ),
            );
            Some(SchemaType::Unknown)
        }
    }
}
