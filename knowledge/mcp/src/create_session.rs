#[derive(serde::Serialize)]
pub struct CreateSessionResult {
    pub session_id: String,
    pub session_file_path: String,
}
