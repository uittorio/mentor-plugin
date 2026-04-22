use rmcp::schemars;

#[derive(serde::Serialize)]
pub struct UpdateSessionResult {
    pub session_name: String,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct UpdateSessionParams {
    pub session_id: String,
    pub content: String,
}
