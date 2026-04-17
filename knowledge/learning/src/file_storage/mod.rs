use std::path::Path;

use crate::session::Session;

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

pub fn session_file_path(session: &Session) -> std::io::Result<String> {
    let folder = file_storage_folder();

    let file_name = session_file_name(&session.name);
    std::fs::create_dir_all(&folder)?;

    Ok(Path::new(&folder)
        .join(file_name)
        .to_str()
        .unwrap()
        .to_string())
}

pub fn session_file_name(session_name: &str) -> String {
    session_name.replace(" ", "_") + ".md"
}

impl Session {
    pub fn file_path(&self) -> std::io::Result<String> {
        session_file_path(&self)
    }
}
