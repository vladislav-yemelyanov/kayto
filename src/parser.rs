use crate::{gen_spec, logger, spec};
use std::collections::HashMap;

pub struct Parser {
    openapi: spec::OpenAPI,
    log: logger::Logger,
    reqs: Vec<gen_spec::Request>,
}

impl Parser {
    fn get_schema_name_by_ref<'a>(&mut self, reference: &'a str) -> Option<&'a str> {
        reference.split("/").last()
    }

    fn get_schema_by_ref<'a>(&mut self, reference: &'a str) -> Option<spec::Schema> {
        let name = self.get_schema_name_by_ref(reference)?;
        let components = &self.openapi.components.as_ref()?;

        let schema1 = components.schemas.get(name); // v3

        if let Some(schema1) = schema1 {
            return schema1.clone();
        }

        let schema2 = components.definitions.as_ref()?.get(name); // v2

        if let Some(schema2) = schema2 {
            return schema2.clone();
        }

        return None;
    }

    fn try_parse_schema(
        &mut self,
        schema: &spec::Schema,
        req_schema: &mut Option<gen_spec::Schema>,
    ) -> Option<String> {
        let type_name = schema.type_name.as_ref()?;

        match type_name {
            spec::SchemaType::OBJECT => {
                if let Some(properties) = &schema.properties {
                    let mut map_: HashMap<String, String> = HashMap::new();
                    for (key, value) in properties {
                        let schema = value.as_ref()?;

                        let t = self.try_parse_schema(&schema, &mut None)?;

                        self.log.field(key.as_str(), &t.as_str());

                        map_.insert(key.to_string(), t);
                    }

                    *req_schema = Some(gen_spec::Schema {
                        ref_: None,
                        map_: Some(map_),
                    });
                }
                return None;
            }
            _ => {
                return Some(type_name.to_string());
            }
        }
    }

    fn try_parse_response(
        &mut self,
        response: &Option<spec::Response>,
        req_schema: &mut Option<gen_spec::Schema>,
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
                // TODO: add schema_name to log
                let schema_name = self.get_schema_name_by_ref(&reference)?;

                *req_schema = Some(gen_spec::Schema {
                    ref_: Some(schema_name.to_string()),
                    map_: None,
                });

                let schema = self.get_schema_by_ref(&reference)?;

                self.try_parse_schema(&schema, &mut None)?
            }
            None => self.try_parse_schema(schema, req_schema)?,
        };

        Some(())
    }

    fn try_parse_responses(
        &mut self,
        method: &spec::Method,
        r_responses: &mut Option<HashMap<u16, Option<gen_spec::Schema>>>,
    ) {
        if let Some(responses) = &method.responses {
            for (status_code, response) in responses {
                let u = status_code.parse::<u16>();

                if let Ok(u) = u {
                    let _ = &self.log.status(u);

                    let mut r_schema: Option<gen_spec::Schema> = None;
                    let _ = &self.try_parse_response(&response, &mut r_schema);

                    let mut r = HashMap::new();

                    r.insert(u, r_schema);
                    let _ = r_responses.insert(r);
                }
            }
        }
    }

    fn try_parse_parameters(
        &mut self,
        method: &spec::Method,
        req_schema: &mut Option<gen_spec::Schema>,
    ) -> Option<()> {
        if let Some(params) = &method.parameters {
            let mut map_: HashMap<String, String> = HashMap::new();

            for param in params {
                if let Some(schema) = &param.schema {
                    let type_ = self.try_parse_schema(schema, &mut None);

                    let name = param.name.as_ref()?;

                    let type_unwrapped = type_?;

                    let _ = &self.log.field(&name, &type_unwrapped.as_str());

                    map_.insert(name.to_string(), type_unwrapped);
                }
            }

            *req_schema = Some(gen_spec::Schema {
                ref_: None,
                map_: Some(map_),
            });
        }

        Some(())
    }

    fn try_parse_methods(
        &mut self,
        pathname: &str,
        methods: &Option<HashMap<spec::MethodVariant, spec::Method>>,
    ) -> Option<()> {
        if let Some(methods) = &methods {
            for (variant, method) in methods {
                let _ = &self.log.method(&variant.to_string());
                let _ = &self.log.increase_indent();

                let mut params: Option<gen_spec::Schema> = None;
                let mut body: Option<gen_spec::Schema> = None;
                let mut responses: Option<HashMap<u16, Option<gen_spec::Schema>>> = None;

                let _ = &self.log.params();
                let _ = &self.log.increase_indent();
                let _ = &self.try_parse_parameters(&method, &mut params);
                let _ = &self.log.decrease_indent();

                let _ = &self.log.body();
                let _ = &self.log.increase_indent();
                let _ = &self.try_parse_response(&method.requestBody, &mut body);
                let _ = &self.log.decrease_indent();

                let _ = &self.log.responses();
                let _ = &self.log.increase_indent();
                let _ = &self.try_parse_responses(&method, &mut responses);
                let _ = &self.log.decrease_indent();

                // end method
                let _ = &self.log.decrease_indent();

                let req = gen_spec::Request {
                    path: pathname.to_string(),
                    type_: variant.to_string(),
                    params: params,
                    body: body,
                    responses: responses,
                };

                let _ = &self.reqs.push(req);
            }
        }

        Some(())
    }

    pub fn new(openapi: spec::OpenAPI) -> Self {
        let log = logger::Logger::new();

        Self {
            openapi,
            log,
            reqs: vec![],
        }
    }

    pub fn parse(&mut self) {
        let paths = self.openapi.paths.take();

        if let Some(paths) = paths {
            for (pathname, methods) in paths {
                let _ = &self.log.path(&pathname);
                let _ = &self.log.increase_indent();
                // TODO: add &mut refs
                let _ = &self.try_parse_methods(&pathname, &methods);
                let _ = &self.log.decrease_indent();
            }
        }

        for r in &self.reqs {
            println!("req: {:?}", r);
        }
    }
}
