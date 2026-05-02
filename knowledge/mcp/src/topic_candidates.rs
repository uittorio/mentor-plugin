use rmcp::schemars;
use serde::{Deserialize, Serialize};

use crate::deserialisers::deserialize_usize;

#[derive(Debug, schemars::JsonSchema, Deserialize)]
pub struct TopicCandidatesParams {
    #[serde(deserialize_with = "deserialize_usize")]
    pub limit: usize,
}
#[derive(Serialize)]
pub struct TopicCandidate {
    pub name: String,
    pub days_since_last_review: i64,
}
