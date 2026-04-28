use libsql::Connection;

use crate::sql::migrations::migration_0::migration_0;

const SCHEMA_VERSION: u32 = 1;

pub async fn run_migrations(connection: &Connection) -> eyre::Result<()> {
    match get_version(connection).await? {
        ..1 => {
            migration_0(connection).await?;
        }
        _ => {}
    }

    set_version(connection).await?;
    Ok(())
}

pub async fn get_version(connection: &Connection) -> eyre::Result<u32> {
    let mut rows = connection.query("PRAGMA user_version", ()).await?;

    let row = rows.next().await?.unwrap();
    Ok(row.get::<u32>(0)?)
}

pub async fn set_version(connection: &Connection) -> eyre::Result<()> {
    connection
        .execute(&format!("PRAGMA user_version = {SCHEMA_VERSION}"), ())
        .await?;
    Ok(())
}
