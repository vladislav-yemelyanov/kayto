use crate::spec;
use std::collections::HashMap;

fn get_schema_name_by_ref(reference: &str) -> Option<&str> {
    reference.split("/").last()
}

fn get_schema_by_ref<'a>(openapi: &'a spec::OpenAPI, reference: &str) -> Option<&'a spec::Schema> {
    let name = get_schema_name_by_ref(reference)?;
    let components = &openapi.components.as_ref()?;

    let schema1 = components.schemas.get(name); // v3
    let schema2 = components.definitions.get(name); // v2

    if let Some(schema1) = schema1 {
        return schema1.as_ref();
    }

    if let Some(schema2) = schema2 {
        return schema2.as_ref();
    }

    return None;
}

fn try_parse_schema(schema: &spec::Schema) -> Option<Vec<String>> {
    let type_name = schema.type_name.as_ref()?;

    let mut fields: Vec<String> = vec![];

    fields.push(format!("type: {}", &type_name));

    match type_name {
        spec::SchemaType::OBJECT => {
            if let Some(properties) = &schema.properties {
                for (k, s) in properties {
                    let schema = s.as_ref()?;

                    fields.push(format!("key: {}", k));
                    let o_fields = try_parse_schema(&schema);

                    if let Some(mut o_fields) = o_fields {
                        fields.append(&mut o_fields);
                    }
                }
            }
        }
        spec::SchemaType::STRING => {}
        spec::SchemaType::NUMBER => {}
        spec::SchemaType::INTEGER => {}
        spec::SchemaType::BOOLEAN => {}
        spec::SchemaType::ARRAY => {}
        spec::SchemaType::NULL => {}
    }

    Some(fields)
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

    let fields = match &schema.reference {
        Some(reference) => {
            let schema_name = get_schema_name_by_ref(&reference)?;
            println!("Reference: {}", &schema_name);
            let schema = get_schema_by_ref(&openapi, &reference)?;

            try_parse_schema(&schema)?
        }
        None => try_parse_schema(schema)?,
    };

    for field in fields {
        println!("{}", field);
    }

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
                try_parse_schema(schema);
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
