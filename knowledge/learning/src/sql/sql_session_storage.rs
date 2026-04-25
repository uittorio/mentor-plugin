use std::sync::Arc;

use async_trait::async_trait;
use turso::{Row, params};
use uuid::Uuid;

use crate::session::{Session, SessionId};
use crate::session_storage::SessionStorage;
use crate::sql::sql_storage::SqlConnection;

pub struct SqlSessionStorage(Arc<SqlConnection>);

impl SqlSessionStorage {
    pub async fn init(conn: Arc<SqlConnection>) -> eyre::Result<Self> {
        let storage = SqlSessionStorage(conn);
        storage.create_tables().await?;
        Ok(storage)
    }

    #[cfg(test)]
    pub async fn init_inmemory() -> eyre::Result<Self> {
        use turso::Builder;

        let database = Arc::new(Builder::new_local(":memory:").build().await?);
        let connection = Arc::new(database.connect()?);
        let conn = Arc::new(SqlConnection { connection });
        let storage = SqlSessionStorage(conn);
        storage.create_tables().await?;
        Ok(storage)
    }

    async fn create_tables(&self) -> eyre::Result<()> {
        self.0
            .connection
            .execute_batch(
                "BEGIN;
            CREATE TABLE IF NOT EXISTS sessions (
              id TEXT PRIMARY KEY,
              name TEXT NOT NULL UNIQUE COLLATE NOCASE,
              created_at INTEGER NOT NULL DEFAULT (unixepoch()),
              modified_at INTEGER NOT NULL DEFAULT (unixepoch()),
              content TEXT NULL
            );
            COMMIT;
            ",
            )
            .await?;
        Ok(())
    }

    fn map(&self, row: &Row) -> eyre::Result<Session> {
        let id_raw: String = row.get(0)?;
        let uuid = Uuid::parse_str(&id_raw)?;
        Ok(Session {
            id: SessionId(uuid),
            name: row.get(1)?,
            created_at: row.get::<i64>(2)? as u64,
            modified_at: row.get::<i64>(3)? as u64,
            content: row.get(4)?,
        })
    }
}

#[async_trait]
impl SessionStorage for SqlSessionStorage {
    async fn create(&self, session: &Session) -> eyre::Result<()> {
        self.0
            .connection
            .execute(
                "
            INSERT INTO sessions (id, name, created_at, modified_at, content)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ",
                params![
                    session.id.0.to_string(),
                    session.name.clone(),
                    session.created_at as i64,
                    session.modified_at as i64,
                    session.content.clone()
                ],
            )
            .await?;
        Ok(())
    }

    async fn update(&self, session: &Session) -> eyre::Result<()> {
        self.0
            .connection
            .execute(
                "
            UPDATE sessions
            SET modified_at = ?1, content = ?2
            WHERE id = ?3
            ",
                params![
                    session.modified_at as i64,
                    session.content.clone(),
                    session.id.0.to_string()
                ],
            )
            .await?;
        Ok(())
    }

    async fn get(&self, session_id: &SessionId) -> eyre::Result<Option<Session>> {
        let mut statement = self
            .0
            .connection
            .prepare(
                "
            SELECT s.id, s.name, s.created_at, s.modified_at, s.content
            FROM sessions s
            WHERE s.id = (?1)
            ",
            )
            .await?;

        let mut rows = statement.query([session_id.0.to_string()]).await?;
        match rows.next().await? {
            Some(row) => Ok(Some(self.map(&row)?)),
            None => Ok(None),
        }
    }

    async fn get_all(&self) -> eyre::Result<Vec<Session>> {
        let mut statement = self
            .0
            .connection
            .prepare(
                "
            SELECT s.id, s.name, s.created_at, s.modified_at, s.content
            FROM sessions s
            ORDER by s.modified_at DESC
            ",
            )
            .await?;

        let mut rows = statement.query(()).await?;
        let mut sessions = Vec::new();
        while let Some(row) = rows.next().await? {
            sessions.push(self.map(&row)?);
        }
        Ok(sessions)
    }
}

#[cfg(test)]
mod tests {
    use crate::session::Session;

    use super::*;

    #[tokio::test]
    async fn create() {
        let storage = SqlSessionStorage::init_inmemory().await.unwrap();

        let session_id = SessionId::new();
        storage
            .create(&Session {
                id: session_id.clone(),
                name: "session name".to_string(),
                created_at: 1775764375,
                modified_at: 1775764371,
                content: Some("content".to_string()),
            })
            .await
            .unwrap();

        let inserted = storage.get(&session_id).await.unwrap().unwrap();

        assert_eq!(inserted.name, "session name");
        assert_eq!(inserted.created_at, 1775764375);
        assert_eq!(inserted.modified_at, 1775764371);
    }

    #[tokio::test]
    async fn update() {
        let storage = SqlSessionStorage::init_inmemory().await.unwrap();

        let session_id = SessionId::new();

        let session = Session {
            id: session_id.clone(),
            name: "session name".to_string(),
            created_at: 1775764375,
            modified_at: 1775764371,
            content: Some("content".to_string()),
        };

        storage.create(&session).await.unwrap();

        let update = session.update_content(&"new content".to_string(), 1775764999);

        storage.update(&update).await.unwrap();

        let inserted = storage.get(&session_id).await.unwrap().unwrap();

        assert_eq!(inserted.content, Some("new content".to_string()));
        assert_eq!(inserted.modified_at, 1775764999);
    }

    #[tokio::test]
    async fn get_all() {
        let storage = SqlSessionStorage::init_inmemory().await.unwrap();

        storage
            .create(&Session {
                id: SessionId::new(),
                name: "session name 1".to_string(),
                created_at: 1775764375,
                modified_at: 1775764371,
                content: Some("content".to_string()),
            })
            .await
            .unwrap();

        storage
            .create(&Session {
                id: SessionId::new(),
                name: "session name 2".to_string(),
                created_at: 1775764375,
                modified_at: 1775764371,
                content: Some("content".to_string()),
            })
            .await
            .unwrap();

        let sessions = storage.get_all().await.unwrap();

        assert_eq!(sessions.len(), 2);
    }
}
