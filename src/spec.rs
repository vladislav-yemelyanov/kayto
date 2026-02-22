use serde::Deserialize;
use std::collections::BTreeMap;
use strum::Display;
use strum_macros::EnumString;

/// HTTP method variants used as keys in the parsed path map.
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Deserialize, Debug, EnumString, Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum MethodVariant {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
}

/// OpenAPI primitive and container schema types.
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

/// Raw OpenAPI schema node as it appears in the source document.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    #[serde(rename = "$ref")]
    pub reference: Option<String>,
    #[serde(rename = "type")]
    pub type_name: Option<SchemaType>,
    #[serde(rename = "oneOf")]
    pub one_of: Option<Vec<Schema>>,
    #[serde(rename = "anyOf")]
    pub any_of: Option<Vec<Schema>>,
    #[serde(rename = "allOf")]
    pub all_of: Option<Vec<Schema>>,
    pub description: Option<String>,
    #[serde(rename = "default")]
    pub default_value: Option<serde_json::Value>,
    pub nullable: Option<bool>,
    pub format: Option<String>,
    pub required: Option<Vec<String>>,
    pub properties: Option<BTreeMap<String, Option<Schema>>>,
    #[serde(rename = "enum")]
    pub enum_variants: Option<Vec<serde_json::Value>>,
    pub items: Option<Box<Schema>>,
}

/// Raw OpenAPI method parameter node (inline or via `$ref`).
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MethodParams {
    #[serde(rename = "$ref")]
    pub reference: Option<String>,
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

/// Media-type payload wrapper that may contain a schema.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub schema: Option<Schema>,
}

/// `content` section with dynamic media-type keys.
#[derive(Deserialize, Debug)]
pub struct ResponseContent {
    #[serde(flatten)]
    pub media_types: BTreeMap<String, Content>,
}

/// Raw OpenAPI response object.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub content: Option<ResponseContent>,
}

/// Raw OpenAPI operation object.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Method {
    pub operation_id: Option<String>,
    pub parameters: Option<Vec<MethodParams>>,
    pub request_body: Option<Response>,
    pub responses: Option<BTreeMap<String, Option<Response>>>,
}

/// Path item methods indexed by method variant.
pub type PathMethods = Option<BTreeMap<MethodVariant, Method>>;

/// Top-level paths indexed by URL template.
pub type Paths = Option<BTreeMap<String, PathMethods>>;

/// Shared components section with reusable schemas/parameters.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Components {
    pub schemas: Option<BTreeMap<String, Option<Schema>>>,
    pub definitions: Option<BTreeMap<String, Option<Schema>>>,
    pub parameters: Option<BTreeMap<String, Option<MethodParams>>>,
}

/// Root OpenAPI document model used by the parser layer.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpenAPI {
    pub paths: Paths,
    pub components: Option<Components>,
    pub parameters: Option<BTreeMap<String, Option<MethodParams>>>,
}
