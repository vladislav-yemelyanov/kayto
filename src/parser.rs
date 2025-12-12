use crate::{logger, spec, ts_codegen};
use std::collections::HashMap;

pub struct Parser {
    openapi: spec::OpenAPI,
    log: logger::Logger,
    reqs: Vec<Request>,
    refs: HashMap<String, SchemaType>,
}

#[derive(Debug, Clone)]
pub enum PrimitiveType {
    String,
    Integer,
    Number,
    Boolean,
}

#[derive(Debug, Clone)]
pub struct Primitive {
    pub kind: PrimitiveType,
    pub enum_values: Option<Vec<String>>,
    // TODO: add descripiton, default, nullable, format
}

#[derive(Debug, Clone)]
pub enum SchemaType {
    Primitive(Primitive),
    Array(Box<SchemaType>),
    Object(HashMap<String, SchemaType>),
    Ref(String),
}

#[derive(Debug)]
pub struct Request {
    pub path: String,
    pub method: String,
    pub params: Option<HashMap<String, SchemaType>>,
    pub body: Option<SchemaType>,
    pub responses: Option<HashMap<u16, SchemaType>>,
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

    fn try_parse_schema(&mut self, schema: &spec::Schema) -> Option<SchemaType> {
        let type_name = schema.type_name.as_ref();

        if let Some(reference) = &schema.reference {
            let schema_name = self.get_schema_name_by_ref(&reference)?;
            return Some(SchemaType::Ref(schema_name.to_string()));
        }

        match type_name? {
            spec::SchemaType::ARRAY => {
                if let Some(items) = &schema.items {
                    let schema_type = self.try_parse_schema(&items)?;

                    return Some(SchemaType::Array(Box::new(schema_type)));
                }
                return None;
            }
            spec::SchemaType::OBJECT => {
                if let Some(properties) = &schema.properties {
                    let mut s = HashMap::new();

                    for (key, value) in properties {
                        let schema = value.as_ref()?;

                        let t = self.try_parse_schema(&schema)?;

                        s.insert(key.to_string(), t);
                    }

                    return Some(SchemaType::Object(s));
                }
                return None;
            }
            spec::SchemaType::STRING => Some(SchemaType::Primitive(Primitive {
                kind: PrimitiveType::String,
                enum_values: None,
            })),
            spec::SchemaType::NUMBER => Some(SchemaType::Primitive(Primitive {
                kind: PrimitiveType::Number,
                enum_values: None,
            })),
            spec::SchemaType::INTEGER => Some(SchemaType::Primitive(Primitive {
                kind: PrimitiveType::Integer,
                enum_values: None,
            })),
            spec::SchemaType::BOOLEAN => Some(SchemaType::Primitive(Primitive {
                kind: PrimitiveType::Boolean,
                enum_values: None,
            })),
            _ => None,
        }
    }

    fn try_parse_response(&mut self, response: &Option<spec::Response>) -> Option<SchemaType> {
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
                self.log.field("ref", schema_name);

                let schema = self.get_schema_by_ref(&reference)?;

                let schema_type = self.try_parse_schema(&schema);

                if let Some(schema_type) = &schema_type {
                    self.refs
                        .insert(schema_name.to_string(), schema_type.clone());
                }

                schema_type
            }
            None => self.try_parse_schema(schema),
        }
    }

    fn log_schema_type(&mut self, key: Option<&str>, schema_type: &SchemaType) -> Option<()> {
        match schema_type {
            SchemaType::Primitive(p) => {
                let v = match p.kind {
                    PrimitiveType::Integer => "integer",
                    PrimitiveType::Number => "number",
                    PrimitiveType::String => "string",
                    PrimitiveType::Boolean => "boolean",
                };
                self.log.field(key?, v);
            }
            SchemaType::Object(map) => {
                for (k, v) in map {
                    self.log_schema_type(Some(k), v);
                }
            }
            _ => {
                //
            }
        }

        Some(())
    }

    fn try_parse_responses(&mut self, method: &spec::Method) -> Option<HashMap<u16, SchemaType>> {
        if let Some(responses) = &method.responses {
            let mut map: HashMap<u16, SchemaType> = HashMap::new();
            for (status_code, response) in responses {
                let u = status_code.parse::<u16>();

                if let Ok(u) = u {
                    self.log.status(u);

                    let schema_type = self.try_parse_response(&response);

                    if let Some(schema_type) = schema_type {
                        self.log_schema_type(None, &schema_type);

                        map.insert(u, schema_type);
                    }
                }
            }
            return Some(map);
        }

        None
    }

    fn try_parse_parameters(
        &mut self,
        method: &spec::Method,
    ) -> Option<HashMap<String, SchemaType>> {
        if let Some(params) = &method.parameters {
            let mut map: HashMap<String, SchemaType> = HashMap::new();

            for param in params {
                if let Some(schema) = &param.schema {
                    let schema_type = self.try_parse_schema(schema);

                    if let Some(schema_type) = schema_type {
                        let name = param.name.as_ref()?;

                        self.log_schema_type(Some(name.as_str()), &schema_type);

                        map.insert(name.to_string(), schema_type);
                    }
                }
            }

            Some(map);
        }

        return None;
    }

    fn try_parse_methods(
        &mut self,
        pathname: &str,
        methods: &Option<HashMap<spec::MethodVariant, spec::Method>>,
    ) -> Option<()> {
        if let Some(methods) = &methods {
            for (variant, method) in methods {
                self.log.method(&variant.to_string());
                self.log.increase_indent();

                self.log.params();
                self.log.increase_indent();
                let params = self.try_parse_parameters(&method);
                self.log.decrease_indent();

                self.log.body();
                self.log.increase_indent();
                let body = self.try_parse_response(&method.request_body);

                if let Some(body) = &body {
                    self.log_schema_type(None, &body);
                }

                self.log.decrease_indent();

                self.log.responses();
                self.log.increase_indent();
                let responses = self.try_parse_responses(&method);
                self.log.decrease_indent();

                // end method
                self.log.decrease_indent();

                let req = Request {
                    path: pathname.to_string(),
                    method: variant.to_string(),
                    params: params,
                    body: body,
                    responses: responses,
                };

                self.reqs.push(req);
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
            refs: HashMap::new(),
        }
    }

    pub fn parse(&mut self) {
        let paths = self.openapi.paths.take();

        if let Some(paths) = paths {
            for (pathname, methods) in paths {
                self.log.path(&pathname);
                self.log.increase_indent();
                self.try_parse_methods(&pathname, &methods);
                self.log.decrease_indent();
            }
        }
    }
}
