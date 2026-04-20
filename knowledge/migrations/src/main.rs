use std::path::Path;

use learning::sqlite::sqlite_storage::db_path;
use rusqlite::{Connection, params};

fn main() {
    session_file_path_to_file_name();
    make_file_name_required();
    add_categories_to_topics();
}

// 0.0.29 onwards
fn session_file_path_to_file_name() {
    let path = db_path().unwrap();
    let connection = Connection::open(path).unwrap();

    connection
        .execute(
            "
        ALTER TABLE sessions ADD COLUMN file_name TEXT;

        ",
            [],
        )
        .unwrap();

    let mut statement = connection
        .prepare(
            "
        SELECT id, name, created_at, modified_at, file_name, file_path from sessions
        WHERE file_name is NULL
        ",
        )
        .unwrap();

    let sessions = statement
        .query_map([], map_sessions)
        .unwrap()
        .collect::<Result<Vec<SessionWithAllHistoricalFields>, rusqlite::Error>>()
        .unwrap();

    for ele in sessions.iter() {
        let file_name = ele
            .file_path
            .as_ref()
            .map(|f| {
                return Path::new(f.as_str())
                    .file_name()
                    .and_then(|p| p.to_str())
                    .unwrap();
            })
            .unwrap();

        connection
            .execute(
                "UPDATE sessions SET file_name = ?1 WHERE id = ?2",
                params![file_name, ele.id],
            )
            .unwrap();
    }

    connection
        .execute(
            "
        ALTER TABLE sessions DROP COLUMN file_path;

        ",
            [],
        )
        .unwrap();
}

// 0.0.30 onwards
fn make_file_name_required() {
    let path = db_path().unwrap();
    let connection = Connection::open(path).unwrap();

    connection
        .execute_batch(
            "BEGIN;
        CREATE TABLE IF NOT EXISTS sessions_new (
          id TEXT PRIMARY KEY,
          name TEXT NOT NULL UNIQUE COLLATE NOCASE,
          created_at INTEGER NOT NULL DEFAULT (unixepoch()),
          modified_at INTEGER NOT NULL DEFAULT (unixepoch()),
          file_name TEXT NOT NULL
        );

        INSERT INTO sessions_new SELECT * FROM sessions;

        DROP TABLE sessions;

        ALTER TABLE sessions_new RENAME TO sessions;
        COMMIT;
        ",
        )
        .unwrap();
}

// 0.0.32 onwards
fn add_categories_to_topics() {
    let path = db_path().unwrap();
    let connection = Connection::open(path).unwrap();

    connection
        .execute(
            "
        ALTER TABLE topics ADD COLUMN categories TEXT NOT NULL DEFAULT '';
        ",
            [],
        )
        .unwrap();
}

fn map_sessions(row: &rusqlite::Row<'_>) -> rusqlite::Result<SessionWithAllHistoricalFields> {
    Ok(SessionWithAllHistoricalFields {
        id: row.get(0)?,
        name: row.get(1)?,
        created_at: row.get::<_, i64>(2)? as u64,
        modified_at: row.get::<_, i64>(3)? as u64,
        file_name: row.get(4)?,
        file_path: row.get(5)?,
    })
}

pub struct SessionWithAllHistoricalFields {
    pub id: String,
    pub name: String,
    pub created_at: u64,
    pub file_name: Option<String>,
    pub file_path: Option<String>,
    pub modified_at: u64,
}
