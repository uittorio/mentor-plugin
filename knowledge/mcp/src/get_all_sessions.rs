use crate::deserialisers::deserialize_usize;
use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, schemars::JsonSchema, Deserialize)]
pub struct GetAllSessionsParams {
    #[serde(deserialize_with = "deserialize_usize")]
    pub limit: usize,
}

#[derive(serde::Serialize)]
pub struct SessionResult {
    pub name: String,
    pub id: String,
}
