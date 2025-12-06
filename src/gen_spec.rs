use std::collections::HashMap;

#[derive(Debug)]
pub struct Schema {
    pub ref_: Option<String>,
    pub map_: Option<HashMap<String, String>>,
}

#[derive(Debug)]
pub struct Request {
    pub path: String,
    pub type_: String,
    pub params: Option<Schema>,
    pub body: Option<Schema>,
    pub responses: Option<HashMap<u16, Option<Schema>>>,
}
