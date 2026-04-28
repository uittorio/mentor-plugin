use libsql::Connection;

pub async fn migration_0(connection: &Connection) -> eyre::Result<()> {
    connection
        .execute_batch(
            "BEGIN;
        CREATE TABLE IF NOT EXISTS sessions (
          id TEXT PRIMARY KEY,
          name TEXT NOT NULL UNIQUE COLLATE NOCASE,
          created_at INTEGER NOT NULL DEFAULT (unixepoch()),
          modified_at INTEGER NOT NULL DEFAULT (unixepoch()),
          content TEXT NULL
        );
        CREATE TABLE IF NOT EXISTS topics (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          name TEXT NOT NULL UNIQUE COLLATE NOCASE,
          created_at INTEGER NOT NULL DEFAULT (unixepoch()),
          ease_factor REAL NOT NULL DEFAULT 2.5,
          interval_days INTEGER NOT NULL DEFAULT 0,
          repetitions INTEGER NOT NULL DEFAULT 0,
          reviewed_at INTEGER NOT NULL,
          categories TEXT NOT NULL
        );
        COMMIT;
        ",
        )
        .await?;

    Ok(())
}
