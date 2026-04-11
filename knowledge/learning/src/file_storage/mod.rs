use std::path::Path;

pub fn file_storage_folder() -> String {
    std::env::var("AGENT_MENTOR_STORAGE_FOLDER").unwrap_or_else(|_| {
        let home = std::env::var("HOME").expect("HOME not set");

        format!("{}/.local/share/agent-mentor", home)
    })
}

pub fn session_file_storage(session_name: &str) -> std::io::Result<String> {
    let folder = file_storage_folder();

    std::fs::create_dir_all(&folder)?;
    let without_spaces = session_name.replace(" ", "_");

    Ok(Path::new(&folder)
        .join(without_spaces + ".md")
        .to_str()
        .unwrap()
        .to_string())
}
