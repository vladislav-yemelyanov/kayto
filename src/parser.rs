use crate::spec;
use std::collections::HashMap;

fn get_schema_name_by_ref(reference: &str) -> Option<&str> {
    reference.split("/").last()
}

fn get_schema_by_ref<'a>(openapi: &'a spec::OpenAPI, reference: &str) -> Option<&'a spec::Schema> {
    let name = get_schema_name_by_ref(reference)?;
    let components = &openapi.components.as_ref()?;

    let schema1 = components.schemas.get(name); // v3

    if let Some(schema1) = schema1 {
        return schema1.as_ref();
    }

    let schema2 = components.definitions.as_ref()?.get(name); // v2

    if let Some(schema2) = schema2 {
        return schema2.as_ref();
    }

    return None;
}

fn try_parse_schema(schema: &spec::Schema, root: bool) -> Option<()> {
    let type_name = schema.type_name.as_ref()?;

    match type_name {
        spec::SchemaType::OBJECT => {
            if let Some(properties) = &schema.properties {
                for (key, value) in properties {
                    let schema = value.as_ref()?;

                    println!(
                        "key: {:?}, value: {:?}",
                        key,
                        &schema.type_name.as_ref()?.to_string()
                    );
                    try_parse_schema(&schema, false);
                }
            }
        }
        // _ => println!("{}", type_name.to_string()),
        _ => {
            if root {
                println!("{}", type_name.to_string());
            }
        }
    }

    Some(())
}

fn try_parse_response(openapi: &spec::OpenAPI, response: &Option<spec::Response>) -> Option<()> {
    let schema = &response
        .as_ref()?
        .content
        .as_ref()?
        .json
        .as_ref()?
        .schema
        .as_ref()?;

    match &schema.reference {
        Some(reference) => {
            let schema_name = get_schema_name_by_ref(&reference)?;
            println!("Response Reference: {}", &schema_name);

            let schema = get_schema_by_ref(&openapi, &reference)?;

            println!("Reference Schema:");
            try_parse_schema(&schema, true)?
        }
        None => {
            println!("Response Type:");
            try_parse_schema(schema, true)?
        }
    };

    Some(())
}

fn try_parse_responses(openapi: &spec::OpenAPI, method: &spec::Method) {
    if let Some(responses) = &method.responses {
        for (status_code, response) in responses {
            println!("Status Code: {}", status_code);

            try_parse_response(&openapi, &response);
        }
    }
}

fn try_parse_parameters(method: &spec::Method) {
    if let Some(params) = &method.parameters {
        for param in params {
            if let Some(schema) = &param.schema {
                try_parse_schema(schema, true);
            }
        }
    }
}

fn try_parse_methods(
    openapi: &spec::OpenAPI,
    pathname: &str,
    methods: &Option<HashMap<spec::MethodVariant, spec::Method>>,
) -> Option<()> {
    println!("Pathname: {}", pathname);

    if let Some(methods) = &methods {
        for (variant, method) in methods {
            println!("Variant: {:?}", variant);

            try_parse_responses(&openapi, &method);
            try_parse_parameters(&method);

            println!("---------------------");
        }
    }

    Some(())
}

pub fn parse_openapi(openapi: &spec::OpenAPI) {
    if let Some(paths) = &openapi.paths {
        for (pathname, methods) in paths {
            try_parse_methods(openapi, pathname, methods);
        }
    }
}
