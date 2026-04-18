use rmcp::schemars;

#[derive(schemars::JsonSchema, serde::Deserialize)]
pub struct SetTopicCategoriesParams {
    pub topic: String,
    pub categories: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct SetTopicCategoriesResult {
    pub topic: String,
    pub categories: Vec<String>,
}
