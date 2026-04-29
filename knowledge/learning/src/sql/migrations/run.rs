use libsql::{Connection, params};

use crate::sql::migrations::migration_0::migration_0;

const SCHEMA_VERSION: u32 = 1;

pub async fn run_migrations(connection: &Connection) -> eyre::Result<()> {
    create_migration_table(connection).await?;

    match get_version(connection).await? {
        ..1 => {
            migration_0(connection).await?;
            add_version(connection).await?;
        }
        _ => {}
    }
    Ok(())
}

pub async fn get_version(connection: &Connection) -> eyre::Result<u32> {
    let mut rows = connection
        .query(
            "SELECT version FROM migrations ORDER BY version DESC LIMIT 1",
            (),
        )
        .await?;

    if let Some(row) = rows.next().await? {
        return Ok(row.get::<u32>(0)?);
    }

    Ok(0)
}

pub async fn add_version(connection: &Connection) -> eyre::Result<()> {
    connection
        .execute(
            &format!("INSERT INTO migrations (version) VALUES (?1);"),
            params![SCHEMA_VERSION],
        )
        .await?;
    Ok(())
}

pub async fn create_migration_table(connection: &Connection) -> eyre::Result<()> {
    connection
        .execute(
            "CREATE TABLE IF NOT EXISTS migrations (version INTEGER NOT NULL)",
            (),
        )
        .await?;

    Ok(())
}
