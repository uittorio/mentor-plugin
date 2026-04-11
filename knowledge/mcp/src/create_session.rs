use rmcp::schemars;

#[derive(serde::Serialize)]
pub struct CreateSessionResult {
    pub session_id: String,
    pub session_file_path: String,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct CreateSessionParams {
    pub name: String,
}
