use std::{fs, path::Path, sync::Arc};

use libsql::{Builder, Connection, Database};
use serde::Deserialize;

use crate::file_storage::file_storage_folder;

pub struct LibsqlConnection {
    pub database: Arc<Database>,
    pub connection: Arc<Connection>,
}

pub fn db_path() -> eyre::Result<String> {
    let folder = file_storage_folder();

    std::fs::create_dir_all(&folder)?;

    Ok(Path::new(&folder)
        .join("knowledge.db")
        .to_str()
        .unwrap()
        .to_string())
}

#[derive(Deserialize)]
pub struct TursoConfig {
    url: String,
    token: String,
}

#[derive(Deserialize)]
pub struct SyncConfig {
    turso: TursoConfig,
}

pub fn config() -> eyre::Result<Option<SyncConfig>> {
    let folder = file_storage_folder();
    let config_path = Path::new(&folder)
        .join("sync.toml")
        .to_str()
        .unwrap()
        .to_string();

    let sync = match fs::read_to_string(config_path) {
        Ok(file_content) => {
            let config: SyncConfig = toml::from_str(&file_content)?;
            Some(config)
        }
        Err(_) => None,
    };

    return Ok(sync);
}

pub async fn libsql_connection() -> eyre::Result<Arc<LibsqlConnection>> {
    let config = config()?;
    let local_path = db_path()?;

    match config {
        Some(config) => {
            let database = Arc::new(
                Builder::new_remote_replica(local_path, config.turso.url, config.turso.token)
                    .build()
                    .await?,
            );

            let connection = Arc::new(database.connect()?);
            database.sync().await?;

            Ok(Arc::new(LibsqlConnection {
                database,
                connection,
            }))
        }
        None => {
            let database = Arc::new(Builder::new_local(local_path).build().await?);
            let connection = Arc::new(database.connect()?);
            Ok(Arc::new(LibsqlConnection {
                database,
                connection,
            }))
        }
    }
}
