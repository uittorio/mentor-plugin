use rmcp::schemars;

#[derive(schemars::JsonSchema, serde::Deserialize)]
pub struct ListAllTopicsParams {}

#[derive(serde::Serialize)]
pub struct TopicEntry {
    pub name: String,
    pub categories: Vec<String>,
}
