use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, schemars::JsonSchema, Deserialize)]
pub struct GetTopicsParams {
    pub search: String,
}
