use rmcp::schemars;

use crate::topic::QuestionDepth;

#[derive(Debug, schemars::JsonSchema, serde::Deserialize)]
pub struct TopicDepthParams {
    pub topic: String,
}

#[derive(serde::Serialize)]
pub struct TopicDepthResult {
    pub name: String,
    pub question_depth: QuestionDepth,
}
