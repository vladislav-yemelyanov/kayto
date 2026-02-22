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
    pub description: Option<String>,
    #[serde(rename = "default")]
    pub default_value: Option<serde_json::Value>,
    pub nullable: Option<bool>,
    pub format: Option<String>,
    pub required: Option<Vec<String>>,
    pub properties: Option<HashMap<String, Option<Schema>>>,
    #[serde(rename = "enum")]
    pub enum_variants: Option<Vec<serde_json::Value>>,
    pub items: Option<Box<Schema>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MethodParams {
    pub name: Option<String>,
    #[serde(rename = "in")]
    pub location: Option<String>,
    pub description: Option<String>,
    pub required: Option<bool>,
    pub schema: Option<Schema>,
    #[serde(rename = "type")]
    pub type_name: Option<SchemaType>,
    #[serde(rename = "default")]
    pub default_value: Option<serde_json::Value>,
    pub nullable: Option<bool>,
    pub format: Option<String>,
    #[serde(rename = "enum")]
    pub enum_variants: Option<Vec<serde_json::Value>>,
    pub items: Option<Box<Schema>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub schema: Option<Schema>,
}

#[derive(Deserialize, Debug)]
pub struct ResponseContent {
    #[serde(flatten)]
    pub media_types: HashMap<String, Content>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub content: Option<ResponseContent>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Method {
    pub operation_id: Option<String>,
    pub parameters: Option<Vec<MethodParams>>,
    pub request_body: Option<Response>,
    pub responses: Option<HashMap<String, Option<Response>>>,
}

pub type PathMethods = Option<HashMap<MethodVariant, Method>>;

pub type Paths = Option<HashMap<String, PathMethods>>;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Components {
    pub schemas: Option<HashMap<String, Option<Schema>>>,
    pub definitions: Option<HashMap<String, Option<Schema>>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpenAPI {
    pub paths: Paths,
    pub components: Option<Components>,
}
