use crate::tools::deserialisers::deserialize_usize;
use rmcp::schemars;
#[derive(Debug, schemars::JsonSchema, serde::Deserialize)]
pub struct ReviewTopicParams {
    pub topic: String,
    #[serde(deserialize_with = "deserialize_usize")]
    pub quality: usize,
}

#[derive(serde::Serialize)]
pub struct ReviewTopicResult {
    pub topic: String,
    pub next_review_in_days: u32,
}
