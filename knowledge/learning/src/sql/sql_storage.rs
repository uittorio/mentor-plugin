use std::{fs, path::Path};

use crate::{file_storage::file_storage_folder, sql::migrations::run::run_migrations};
use libsql::{Builder, Connection};
use serde::Deserialize;

#[derive(Clone)]
pub enum ConnectionType {
    Remote,
    Local,
}

pub struct SqlConnection {
    pub connection: Connection,
    pub connection_type: ConnectionType,
}

impl SqlConnection {
    pub async fn new() -> eyre::Result<SqlConnection> {
        let config = config()?;
        let local_path = db_path()?;

        let sql_connection = match config {
            Some(config) => SqlConnection::remote(local_path, config).await?,
            None => SqlConnection::local(local_path).await?,
        };

        Ok(sql_connection)
    }

    async fn remote(local_path: String, config: SyncConfig) -> eyre::Result<SqlConnection> {
        let database =
            Builder::new_remote_replica(&local_path, config.turso.url, config.turso.token)
                .build()
                .await;

        match database {
            Ok(database) => {
                let connection = database.connect()?;

                database.sync().await?;
                run_migrations(&connection).await?;
                Ok(SqlConnection {
                    connection,
                    connection_type: ConnectionType::Remote,
                })
            }
            Err(_) => Ok(SqlConnection::local(local_path).await?),
        }
    }

    async fn local(local_path: String) -> eyre::Result<SqlConnection> {
        let database = Builder::new_local(local_path).build().await?;
        let connection = database.connect()?;
        run_migrations(&connection).await?;
        Ok(SqlConnection {
            connection,
            connection_type: ConnectionType::Local,
        })
    }

    #[cfg(test)]
    pub async fn new_in_memory() -> eyre::Result<SqlConnection> {
        let database = Builder::new_local(":memory:").build().await?;
        let connection = database.connect()?;
        run_migrations(&connection).await?;
        Ok(SqlConnection {
            connection,
            connection_type: ConnectionType::Local,
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
