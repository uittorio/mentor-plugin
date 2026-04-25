use std::{fs, path::Path, sync::Arc, time::Duration};

use crate::file_storage::file_storage_folder;
use serde::Deserialize;
use tokio::time::sleep;
use turso::{Connection, sync::Builder};

pub struct SqlConnection {
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

pub async fn sql_connection() -> eyre::Result<Arc<SqlConnection>> {
    let config = config()?;
    let local_path = db_path()?;

    match config {
        Some(config) => {
            let database = Arc::new(
                Builder::new_remote(&local_path)
                    .with_remote_url(config.turso.url)
                    .with_auth_token(config.turso.token)
                    .bootstrap_if_empty(true)
                    .build()
                    .await?,
            );

            let push_db = database.clone();
            tokio::spawn(async move {
                loop {
                    sleep(Duration::from_secs(5)).await;
                    push_db.push().await.ok();
                }
            });

            let connect: Connection = database.connect().await?;
            let connection = Arc::new(connect);
            database.pull().await?;

            Ok(Arc::new(SqlConnection { connection }))
        }
        None => {
            let database = turso::Builder::new_local(&local_path).build().await?;
            let connection = Arc::new(database.connect()?);
            Ok(Arc::new(SqlConnection { connection }))
        }
    }
}
