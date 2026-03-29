use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, schemars::JsonSchema, Deserialize)]
pub struct TopicsParams {
    pub search: String,
}
