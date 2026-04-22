use std::{path::Path, sync::Arc};

use libsql::{Builder, Connection};

use crate::file_storage::file_storage_folder;

pub fn db_path() -> eyre::Result<String> {
    let folder = file_storage_folder();

    std::fs::create_dir_all(&folder)?;

    Ok(Path::new(&folder)
        .join("knowledge.db")
        .to_str()
        .unwrap()
        .to_string())
}

pub async fn connection() -> eyre::Result<Arc<Connection>> {
    let path = db_path()?;
    let database = Builder::new_local(path).build().await?;
    let connection = database.connect()?;
    return Ok(Arc::new(connection));
}
