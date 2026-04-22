use std::str::FromStr;
use std::sync::Mutex;

use rusqlite::types::{FromSql, FromSqlError, ToSqlOutput};
use rusqlite::{Connection, OptionalExtension, ToSql, params};
use uuid::Uuid;

use crate::session::{Session, SessionId};
use crate::session_storage::SessionStorage;
use crate::sqlite::sqlite_storage::db_path;
use crate::storage_error::StorageError;

pub struct SqliteSessionStorage(Mutex<Connection>);

impl SqliteSessionStorage {
    #[cfg(test)]
    pub fn init_inmemory() -> Result<Self, StorageError> {
        let connection = Connection::open_in_memory()?;
        let storage = SqliteSessionStorage(Mutex::new(connection));
        storage.create_tables()?;
        Ok(storage)
    }

    pub fn init() -> Result<Self, StorageError> {
        let path = db_path()?;
        let connection = Connection::open(path)?;
        let storage = SqliteSessionStorage(Mutex::new(connection));
        storage.create_tables()?;
        Ok(storage)
    }

    fn create_tables(&self) -> rusqlite::Result<()> {
        let conn = self.0.lock().unwrap();
        conn.execute_batch(
            "BEGIN;
            CREATE TABLE IF NOT EXISTS sessions (
              id TEXT PRIMARY KEY,
              name TEXT NOT NULL UNIQUE COLLATE NOCASE,
              created_at INTEGER NOT NULL DEFAULT (unixepoch()),
              modified_at INTEGER NOT NULL DEFAULT (unixepoch()),
              file_name TEXT NOT NULL
            );
            COMMIT;
            ",
        )
    }

    fn map(&self, row: &rusqlite::Row<'_>) -> rusqlite::Result<Session> {
        Ok(Session {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get::<_, i64>(2)? as u64,
            modified_at: row.get::<_, i64>(3)? as u64,
            file_name: row.get(4)?,
        })
    }
}

impl FromSql for SessionId {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let uuid_raw = value.as_str()?;
        let uuid = Uuid::from_str(uuid_raw).map_err(|e| FromSqlError::Other(Box::new(e)))?;
        Ok(SessionId(uuid))
    }
}

impl ToSql for SessionId {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        let value = self.0.to_string();
        rusqlite::Result::Ok(ToSqlOutput::from(value))
    }
}

impl From<uuid::Error> for StorageError {
    fn from(value: uuid::Error) -> Self {
        StorageError {
            message: "uuid error".to_string(),
            source: Some(Box::new(value)),
        }
    }
}

impl SessionStorage for SqliteSessionStorage {
    fn create(&self, session: &Session) -> Result<(), StorageError> {
        let conn = self.0.lock().unwrap();

        conn.execute(
            "
            INSERT INTO sessions (id, name, created_at, modified_at, file_name)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ",
            params![
                session.id,
                session.name,
                session.created_at as i64,
                session.modified_at as i64,
                session.file_name,
            ],
        )?;

        Ok(())
    }

    fn get(&self, session_id: &SessionId) -> Result<Option<Session>, StorageError> {
        let conn = self.0.lock().unwrap();
        let mut statement = conn.prepare(
            "
            SELECT s.id, s.name, s.created_at, s.modified_at, s.file_name
            FROM sessions s
            WHERE s.id = (?1)
            ",
        )?;

        let session = statement
            .query_one::<Session, _, _>([session_id], |row| self.map(row))
            .optional()?;

        Ok(session)
    }

    fn get_all(&self) -> Result<Vec<Session>, StorageError> {
        let conn = self.0.lock().unwrap();
        let mut statement = conn.prepare(
            "
            SELECT s.id, s.name, s.created_at, s.modified_at, s.file_name
            FROM sessions s
            ORDER by s.modified_at DESC
            ",
        )?;

        let sessions = statement
            .query_map([], |row| self.map(row))?
            .collect::<Result<Vec<Session>, rusqlite::Error>>()?;

        Ok(sessions)
    }
}

#[cfg(test)]
mod tests {
    use crate::session::Session;

    use super::*;

    #[test]
    fn create() {
        let storage = SqliteSessionStorage::init_inmemory().unwrap();

        let session_id = SessionId::new();
        storage
            .create(&Session {
                id: session_id.clone(),
                name: "session name".to_string(),
                created_at: 1775764375,
                modified_at: 1775764371,
                file_name: Some("path.md".to_string()),
            })
            .unwrap();

        let inserted = storage.get(&session_id).unwrap().unwrap();

        assert_eq!(inserted.name, "session name");
        assert_eq!(inserted.created_at, 1775764375);
        assert_eq!(inserted.modified_at, 1775764371);
    }

    #[test]
    fn get_all() {
        let storage = SqliteSessionStorage::init_inmemory().unwrap();

        storage
            .create(&Session {
                id: SessionId::new(),
                name: "session name 1".to_string(),
                created_at: 1775764375,
                modified_at: 1775764371,
                file_name: Some("path1.md".to_string()),
            })
            .unwrap();

        storage
            .create(&Session {
                id: SessionId::new(),
                name: "session name 2".to_string(),
                created_at: 1775764375,
                modified_at: 1775764371,
                file_name: Some("path2.md".to_string()),
            })
            .unwrap();

        let sessions = storage.get_all().unwrap();

        assert_eq!(sessions.len(), 2);
    }
}
