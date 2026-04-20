use rmcp::schemars;
#[derive(Debug, schemars::JsonSchema, serde::Deserialize)]
pub struct UpdateTopicCategoriesParams {
    pub topic: String,
    pub categories: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct UpdateTopicCategoriesResult {
    pub topic: String,
    pub categories: Vec<String>,
}
