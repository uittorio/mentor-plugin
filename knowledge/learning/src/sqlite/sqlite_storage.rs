use std::path::Path;

use crate::{file_storage::file_storage_folder, storage_error::StorageError};

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
    let folder = file_storage_folder();

    std::fs::create_dir_all(&folder)?;

    Ok(Path::new(&folder)
        .join("knowledge.db")
        .to_str()
        .unwrap()
        .to_string())
}
