use crate::parser::{Request, SchemaType};
use std::collections::{BTreeMap, BTreeSet};

/// Prepared registry data used by the Dart renderer.
pub struct ModelData {
    pub definitions: BTreeMap<String, SchemaType>,
    pub names: BTreeSet<String>,
}

/// Collects model definitions and referenced model names from parsed requests.
pub fn prepare_model_data(
    requests: &[Request],
    models: &BTreeMap<String, SchemaType>,
) -> ModelData {
    let mut model_definitions: BTreeMap<String, SchemaType> = models.clone();
    let mut model_names: BTreeSet<String> = BTreeSet::new();

    for req in requests {
        add_models_from_request(req, &mut model_definitions, &mut model_names);
    }

    for model_name in model_definitions.keys() {
        model_names.insert(model_name.to_string());
    }

    ModelData {
        definitions: model_definitions,
        names: model_names,
    }
}

/// Processes all schema-carrying parts of a request and updates model registries.
fn add_models_from_request(
    req: &Request,
    model_definitions: &mut BTreeMap<String, SchemaType>,
    model_names: &mut BTreeSet<String>,
) {
    add_models_from_params(req, model_names);
    add_models_from_body(req, model_definitions, model_names);
    add_models_from_responses(req, model_definitions, model_names);
}

/// Collects model references from operation parameters.
fn add_models_from_params(req: &Request, model_names: &mut BTreeSet<String>) {
    if let Some(params) = &req.params {
        for param in params {
            if let Some(schema_type) = &param.schema_type {
                collect_ref_names(schema_type, model_names);
            }
        }
    }
}

/// Collects model definitions and references from request body schema.
fn add_models_from_body(
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
            collect_ref_names(schema_type, model_names);
        }
    }
}

/// Collects model definitions and references from response schemas.
fn add_models_from_responses(
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
                collect_ref_names(schema_type, model_names);
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
fn collect_ref_names(schema: &SchemaType, names: &mut BTreeSet<String>) {
    match schema {
        SchemaType::Ref(name) => {
            names.insert(name.clone());
        }
        SchemaType::Array(inner) => collect_ref_names(inner, names),
        SchemaType::Object(obj) => {
            for value in obj.properties.values() {
                collect_ref_names(value, names);
            }
        }
        SchemaType::OneOf(variants)
        | SchemaType::AnyOf(variants)
        | SchemaType::AllOf(variants) => {
            for variant in variants {
                collect_ref_names(variant, names);
            }
        }
        SchemaType::Primitive(_) | SchemaType::Unknown => {}
    }
}
