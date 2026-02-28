use crate::parser::{Request, SchemaType};
use std::collections::{BTreeMap, BTreeSet};

/// Collected registry used by Dart schema generation.
pub struct ModelCatalog {
    pub definitions: BTreeMap<String, SchemaType>,
    pub names: BTreeSet<String>,
}

/// Collects model definitions and referenced model names from parsed requests.
pub fn build_model_catalog(
    requests: &[Request],
    models: &BTreeMap<String, SchemaType>,
) -> ModelCatalog {
    let mut model_definitions: BTreeMap<String, SchemaType> = models.clone();
    let mut model_names: BTreeSet<String> = BTreeSet::new();

    for req in requests {
        collect_request_models(req, &mut model_definitions, &mut model_names);
    }

    for model_name in model_definitions.keys() {
        model_names.insert(model_name.to_string());
    }

    ModelCatalog {
        definitions: model_definitions,
        names: model_names,
    }
}

/// Processes all schema-carrying parts of a request and updates model registries.
fn collect_request_models(
    req: &Request,
    model_definitions: &mut BTreeMap<String, SchemaType>,
    model_names: &mut BTreeSet<String>,
) {
    collect_param_models(req, model_names);
    collect_body_models(req, model_definitions, model_names);
    collect_response_models(req, model_definitions, model_names);
}

/// Collects model references from operation parameters.
fn collect_param_models(req: &Request, model_names: &mut BTreeSet<String>) {
    if let Some(params) = &req.params {
        for param in params {
            if let Some(schema_type) = &param.schema_type {
                collect_schema_refs(schema_type, model_names);
            }
        }
    }
}

/// Collects model definitions and references from request body schema.
fn collect_body_models(
    req: &Request,
    model_definitions: &mut BTreeMap<String, SchemaType>,
    model_names: &mut BTreeSet<String>,
) {
    if let Some(body) = &req.body {
        register_named_schema(
            body.schema_name.as_ref(),
            body.schema_type.as_ref(),
            model_definitions,
            model_names,
        );

        if let Some(schema_type) = &body.schema_type {
            collect_schema_refs(schema_type, model_names);
        }
    }
}

/// Collects model definitions and references from response schemas.
fn collect_response_models(
    req: &Request,
    model_definitions: &mut BTreeMap<String, SchemaType>,
    model_names: &mut BTreeSet<String>,
) {
    if let Some(responses) = &req.responses {
        for response in responses.values() {
            register_named_schema(
                response.schema_name.as_ref(),
                response.schema_type.as_ref(),
                model_definitions,
                model_names,
            );

            if let Some(schema_type) = &response.schema_type {
                collect_schema_refs(schema_type, model_names);
            }
        }
    }
}

/// Registers a named schema definition when both name and schema are available.
fn register_named_schema(
    schema_name: Option<&String>,
    schema_type: Option<&SchemaType>,
    model_definitions: &mut BTreeMap<String, SchemaType>,
    model_names: &mut BTreeSet<String>,
) {
    let Some(name) = schema_name else {
        return;
    };

    model_names.insert(name.clone());

    if let Some(schema_type) = schema_type {
        model_definitions
            .entry(name.clone())
            .or_insert_with(|| schema_type.clone());
    }
}

/// Traverses a schema tree and collects all `$ref` model names.
fn collect_schema_refs(schema: &SchemaType, names: &mut BTreeSet<String>) {
    match schema {
        SchemaType::Ref(name) => {
            names.insert(name.clone());
        }
        SchemaType::Array(inner) => collect_schema_refs(inner, names),
        SchemaType::Object(obj) => {
            for value in obj.properties.values() {
                collect_schema_refs(value, names);
            }
        }
        SchemaType::OneOf(variants) | SchemaType::AnyOf(variants) | SchemaType::AllOf(variants) => {
            for variant in variants {
                collect_schema_refs(variant, names);
            }
        }
        SchemaType::Primitive(_) | SchemaType::Unknown => {}
    }
}
