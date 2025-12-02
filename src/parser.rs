use crate::{logger, spec};
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

fn try_parse_schema(schema: &spec::Schema, root: bool, log: &mut logger::Logger) -> Option<String> {
    let type_name = schema.type_name.as_ref()?;

    match type_name {
        spec::SchemaType::OBJECT => {
            if let Some(properties) = &schema.properties {
                for (key, value) in properties {
                    let schema = value.as_ref()?;

                    let t = try_parse_schema(&schema, false, log)?;
                    log.field(key.as_str(), &t.as_str());
                }
            }
            return None;
        }
        _ => {
            if root {
                // println!("{}", type_name.to_string());
            }
            return Some(type_name.to_string());
        }
    }
}

fn try_parse_response(
    openapi: &spec::OpenAPI,
    response: &Option<spec::Response>,
    log: &mut logger::Logger,
) -> Option<()> {
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

            let schema = get_schema_by_ref(&openapi, &reference)?;

            try_parse_schema(&schema, true, log)?
        }
        None => {
            // println!("Response Type:");
            try_parse_schema(schema, true, log)?
        }
    };

    Some(())
}

fn try_parse_responses(openapi: &spec::OpenAPI, method: &spec::Method, log: &mut logger::Logger) {
    if let Some(responses) = &method.responses {
        for (status_code, response) in responses {
            let u = status_code.parse::<u16>();

            if let Ok(u) = u {
                log.status(u, |log| {
                    try_parse_response(&openapi, &response, log);
                })
            }
        }
    }
}

fn try_parse_parameters(method: &spec::Method, log: &mut logger::Logger) -> Option<()> {
    if let Some(params) = &method.parameters {
        for param in params {
            if let Some(schema) = &param.schema {
                let t = try_parse_schema(schema, true, log);
                let name = param.name.as_ref()?;

                log.field(&name, &t?.as_str());
            }
        }
    }

    Some(())
}

fn try_parse_methods(
    openapi: &spec::OpenAPI,
    pathname: &str,
    methods: &Option<HashMap<spec::MethodVariant, spec::Method>>,
) -> Option<()> {
    let mut log = logger::Logger::new();

    log.path(pathname, |log| {
        if let Some(methods) = &methods {
            for (variant, method) in methods {
                log.method(&variant.to_string().as_str(), |log| {
                    log.params(|log| {
                        try_parse_parameters(&method, log);
                    });

                    log.body(|log| {
                        try_parse_response(&openapi, &method.requestBody, log);
                    });

                    log.response(|log| {
                        try_parse_responses(&openapi, &method, log);
                    });
                });
            }
        }
    });

    Some(())
}

pub fn parse_openapi(openapi: &spec::OpenAPI) {
    if let Some(paths) = &openapi.paths {
        for (pathname, methods) in paths {
            try_parse_methods(openapi, pathname, methods);
        }
    }
}
