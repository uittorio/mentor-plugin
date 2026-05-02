use std::sync::Arc;

use async_trait::async_trait;
use chrono::DateTime;
use eyre::OptionExt;
use libsql::Row;
use libsql::params;

use crate::session::{Session, SessionId};
use crate::session_storage::SessionStorage;
use crate::sql::sql_storage::SqlConnection;

pub struct SqlSessionStorage(pub Arc<SqlConnection>);

impl SqlSessionStorage {
    fn map(&self, row: &Row) -> eyre::Result<Session> {
        use uuid::Uuid;

        let id_raw: String = row.get(0)?;
        let uuid = Uuid::parse_str(&id_raw)?;

        let created_at = DateTime::from_timestamp_secs(row.get::<i64>(2)?)
            .ok_or_eyre("Expecting valid epoc for created_at")?;

        let modified_at = DateTime::from_timestamp_secs(row.get::<i64>(3)?)
            .ok_or_eyre("Expecting valid epoc for modified_at")?;

        Ok(Session {
            id: SessionId(uuid),
            name: row.get(1)?,
            created_at: created_at,
            modified_at: modified_at,
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
                    session.created_at.timestamp(),
                    session.modified_at.timestamp(),
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
                    session.modified_at.timestamp(),
                    session.content.clone(),
                    session.id.0.to_string()
                ],
            )
            .await?;

        Ok(())
    }

    async fn get(&self, session_id: &SessionId) -> eyre::Result<Option<Session>> {
        let statement = self
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
        let statement = self
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
    use chrono::Utc;

    use crate::session::Session;

    use super::*;

    #[tokio::test]
    async fn create() {
        let connection = SqlConnection::new_in_memory().await.unwrap();
        let storage = SqlSessionStorage(Arc::new(connection));

        let session_id = SessionId::new();
        storage
            .create(&Session {
                id: session_id.clone(),
                name: "session name".to_string(),
                created_at: date_time(1775764375),
                modified_at: date_time(1775764371),
                content: Some("content".to_string()),
            })
            .await
            .unwrap();

        let inserted = storage.get(&session_id).await.unwrap().unwrap();

        assert_eq!(inserted.name, "session name");
        assert_eq!(inserted.created_at, date_time(1775764375));
        assert_eq!(inserted.modified_at, date_time(1775764371));
    }

    #[tokio::test]
    async fn update() {
        let connection = SqlConnection::new_in_memory().await.unwrap();
        let storage = SqlSessionStorage(Arc::new(connection));

        let session_id = SessionId::new();

        let session = Session {
            id: session_id.clone(),
            name: "session name".to_string(),
            created_at: date_time(1775764375),
            modified_at: date_time(1775764371),
            content: Some("content".to_string()),
        };

        storage.create(&session).await.unwrap();

        let update = session.update_content(&"new content".to_string(), date_time(1775764999));

        storage.update(&update).await.unwrap();

        let inserted = storage.get(&session_id).await.unwrap().unwrap();

        assert_eq!(inserted.content, Some("new content".to_string()));
        assert_eq!(inserted.modified_at, date_time(1775764999));
    }

    #[tokio::test]
    async fn get_all() {
        let connection = SqlConnection::new_in_memory().await.unwrap();
        let storage = SqlSessionStorage(Arc::new(connection));

        storage
            .create(&Session {
                id: SessionId::new(),
                name: "session name 1".to_string(),
                created_at: date_time(1775764375),
                modified_at: date_time(1775764371),
                content: Some("content".to_string()),
            })
            .await
            .unwrap();

        storage
            .create(&Session {
                id: SessionId::new(),
                name: "session name 2".to_string(),
                created_at: date_time(1775764375),
                modified_at: date_time(1775764371),
                content: Some("content".to_string()),
            })
            .await
            .unwrap();

        let sessions = storage.get_all().await.unwrap();

        assert_eq!(sessions.len(), 2);
    }

    fn date_time(seconds: i64) -> DateTime<Utc> {
        DateTime::from_timestamp_secs(seconds).unwrap()
    }
}
