pub fn file_storage_folder() -> String {
    std::env::var("AGENT_MENTOR_STORAGE_FOLDER").unwrap_or_else(|_| {
        let home = std::env::var("HOME").expect("HOME not set");

        format!("{}/.local/share/agent-mentor", home)
    })
}
