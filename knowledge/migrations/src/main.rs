use std::{fs, path::Path, sync::Arc};

use learning::file_storage::file_storage_folder;
use learning::libsql::libsql_storage::connection;
use libsql::{Connection, params};

#[tokio::main]
async fn main() {
    let conn = connection().await.unwrap();
    session_file_path_to_file_name(&conn).await;
    make_file_name_required(&conn).await;
    add_categories_to_topics(&conn).await;
    add_content_to_sessions(&conn).await;
}

// 0.0.29 onwards
async fn session_file_path_to_file_name(conn: &Arc<Connection>) {
    conn.execute("ALTER TABLE sessions ADD COLUMN file_name TEXT;", ())
        .await
        .unwrap();

    let statement = conn
        .prepare(
            "SELECT id, name, created_at, modified_at, file_name, file_path from sessions
        WHERE file_name is NULL",
        )
        .await
        .unwrap();

    let mut rows = statement.query(()).await.unwrap();
    let mut sessions: Vec<SessionWithAllHistoricalFields> = Vec::new();
    while let Some(row) = rows.next().await.unwrap() {
        sessions.push(map_sessions(&row));
    }

    for ele in sessions.iter() {
        let file_name = ele
            .file_path
            .as_ref()
            .map(|f| {
                Path::new(f.as_str())
                    .file_name()
                    .and_then(|p| p.to_str())
                    .unwrap()
                    .to_string()
            })
            .unwrap();

        conn.execute(
            "UPDATE sessions SET file_name = ?1 WHERE id = ?2",
            params![file_name, ele.id.clone()],
        )
        .await
        .unwrap();
    }

    conn.execute("ALTER TABLE sessions DROP COLUMN file_path;", ())
        .await
        .unwrap();
}

// 0.0.30 onwards
async fn make_file_name_required(conn: &Arc<Connection>) {
    conn.execute_batch(
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
    .await
    .unwrap();
}

// 0.0.32 onwards
async fn add_categories_to_topics(conn: &Arc<Connection>) {
    conn.execute(
        "ALTER TABLE topics ADD COLUMN categories TEXT NOT NULL DEFAULT '';",
        (),
    )
    .await
    .unwrap();
}

// 0.0.34 onwards
async fn add_content_to_sessions(conn: &Arc<Connection>) {
    conn.execute("ALTER TABLE sessions ADD COLUMN content TEXT NULL;", ())
        .await
        .unwrap();

    let statement = conn
        .prepare(
            "SELECT id, name, created_at, modified_at, file_name, content from sessions
        WHERE content is NULL",
        )
        .await
        .unwrap();

    let mut rows = statement.query(()).await.unwrap();
    let mut sessions: Vec<SessionWithAllHistoricalFields> = Vec::new();
    while let Some(row) = rows.next().await.unwrap() {
        sessions.push(map_sessions(&row));
    }

    for ele in sessions.iter() {
        let folder = file_storage_folder();

        let file_name = ele.name.replace(" ", "_") + ".md";

        let path = Path::new(&folder)
            .join(file_name)
            .to_str()
            .unwrap()
            .to_string();

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => format!("No content found e: {}", e),
        };

        conn.execute(
            "UPDATE sessions SET content = ?1 WHERE id = ?2",
            params![content, ele.id.clone()],
        )
        .await
        .unwrap();
    }
}

fn map_sessions(row: &libsql::Row) -> SessionWithAllHistoricalFields {
    SessionWithAllHistoricalFields {
        id: row.get(0).unwrap(),
        name: row.get(1).unwrap(),
        created_at: row.get::<i64>(2).unwrap() as u64,
        modified_at: row.get::<i64>(3).unwrap() as u64,
        file_name: row.get(4).unwrap(),
        file_path: None,
        content: row.get(5).unwrap(),
    }
}

pub struct SessionWithAllHistoricalFields {
    pub id: String,
    pub name: String,
    pub created_at: u64,
    pub file_name: Option<String>,
    pub file_path: Option<String>,
    pub modified_at: u64,
    pub content: Option<String>,
}
