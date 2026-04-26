use std::{fs, path::Path, sync::Arc};

use crate::file_storage::file_storage_folder;
use libsql::{Builder, Connection, Database};
use serde::Deserialize;

pub struct SqlConnection {
    pub connection: Connection,
    _database: Option<Arc<Database>>,
}

impl SqlConnection {
    pub async fn new() -> eyre::Result<SqlConnection> {
        let config = config()?;
        let local_path = db_path()?;

        match config {
            Some(config) => {
                let database = Arc::new(
                    Builder::new_remote_replica(local_path, config.turso.url, config.turso.token)
                        .build()
                        .await?,
                );
                let connection = database.connect()?;
                database.sync().await?;
                Ok(SqlConnection {
                    connection,
                    _database: Some(database),
                })
            }
            None => {
                let database = Builder::new_local(local_path).build().await?;
                let connection = database.connect()?;
                Ok(SqlConnection {
                    connection,
                    _database: None,
                })
            }
        }
    }

    #[cfg(test)]
    pub async fn new_inmemory() -> eyre::Result<SqlConnection> {
        let database = Builder::new_local(":memory:").build().await?;
        let connection = database.connect()?;
        Ok(SqlConnection {
            connection,
            _database: None,
        })
    }
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
