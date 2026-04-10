use std::path::Path;

use crate::storage_error::StorageError;

impl From<std::io::Error> for StorageError {
    fn from(value: std::io::Error) -> Self {
        StorageError {
            message: "Stdio error".to_string(),
            source: Some(Box::new(value)),
        }
    }
}

impl From<rusqlite::Error> for StorageError {
    fn from(value: rusqlite::Error) -> Self {
        StorageError {
            message: "Sqlite error".to_string(),
            source: Some(Box::new(value)),
        }
    }
}

pub fn db_path() -> std::io::Result<String> {
    let folder = std::env::var("AGENT_MENTOR_DB_FOLDER").unwrap_or_else(|_| {
        let home = std::env::var("HOME").expect("HOME not set");

        format!("{}/.local/share/agent-mentor", home)
    });

    std::fs::create_dir_all(&folder)?;

    Ok(Path::new(&folder)
        .join("knowledge.db")
        .to_str()
        .unwrap()
        .to_string())
}
