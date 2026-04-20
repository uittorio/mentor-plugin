#[derive(serde::Serialize)]
pub struct TopicResult {
    pub name: String,
    pub categories: Vec<String>,
}
