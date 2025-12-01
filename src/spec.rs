use serde::Deserialize;
use std::collections::HashMap;
use strum::Display;
use strum_macros::EnumString;

#[derive(Hash, PartialEq, Eq, Deserialize, Debug, EnumString, Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum MethodVariant {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
}

#[derive(Hash, PartialEq, Eq, Deserialize, Debug, EnumString, Display, Clone)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum SchemaType {
    STRING,
    NUMBER,
    INTEGER,
    BOOLEAN,
    ARRAY,
    OBJECT,
    NULL,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    #[serde(rename = "$ref")]
    pub reference: Option<String>,
    #[serde(rename = "type")]
    pub type_name: Option<SchemaType>,
    pub properties: Option<HashMap<String, Option<Schema>>>,
    #[serde(rename = "enum")]
    pub enum_variants: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MethodParams {
    pub name: Option<String>,
    pub description: Option<String>,
    pub required: Option<bool>,
    pub schema: Option<Schema>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub schema: Option<Schema>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseContent {
    #[serde(rename = "application/json")]
    pub json: Option<Content>,
    #[serde(rename = "multipart/form-data")]
    pub form_data: Option<Content>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub content: Option<ResponseContent>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Method {
    pub operations_id: Option<String>,
    pub parameters: Option<Vec<MethodParams>>,
    pub responses: Option<HashMap<String, Option<Response>>>,
}

pub type Paths = Option<HashMap<String, Option<HashMap<MethodVariant, Method>>>>;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Components {
    pub schemas: HashMap<String, Option<Schema>>,
    pub definitions: HashMap<String, Option<Schema>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpenAPI {
    pub paths: Paths,
    pub components: Option<Components>,
}
