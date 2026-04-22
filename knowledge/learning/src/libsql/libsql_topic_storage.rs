use std::sync::Arc;

use async_trait::async_trait;
use libsql::{Connection, params};

use crate::category::Category;
use crate::topic::{Topic, TopicCategories};
use crate::topic_storage::TopicStorage;

pub struct LibsqlTopicStorage(Arc<Connection>);

impl LibsqlTopicStorage {
    pub async fn init(connection: Arc<Connection>) -> eyre::Result<Self> {
        let storage = LibsqlTopicStorage(connection);
        storage.create_tables().await?;
        Ok(storage)
    }

    #[cfg(test)]
    pub async fn init_inmemory() -> eyre::Result<Self> {
        let database = libsql::Builder::new_local(":memory:").build().await?;
        let connection = Arc::new(database.connect()?);
        let storage = LibsqlTopicStorage(connection);
        storage.create_tables().await?;
        Ok(storage)
    }

    async fn create_tables(&self) -> eyre::Result<()> {
        self.0
            .execute_batch(
                "PRAGMA foreign_keys = ON;
            BEGIN;
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

    fn map(&self, row: &libsql::Row) -> eyre::Result<Topic> {
        let categories_raw: String = row.get(5)?;
        let categories = categories_raw
            .split(",")
            .filter(|v| !v.is_empty())
            .map(|v| Category {
                name: v.to_string(),
            })
            .collect::<Vec<Category>>();

        Ok(Topic {
            name: row.get(0)?,
            repetitions: row.get::<i64>(1)? as u32,
            interval: row.get::<i64>(2)? as u32,
            ease_factor: row.get::<f64>(3)? as f32,
            reviewed_at: row.get::<i64>(4)? as u64,
            categories: TopicCategories(categories),
        })
    }

    fn categories_to_string(categories: &TopicCategories) -> String {
        categories
            .0
            .iter()
            .map(|v| v.name.as_str())
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[async_trait]
impl TopicStorage for LibsqlTopicStorage {
    async fn get_overdue(&self, now: u64) -> eyre::Result<Vec<Topic>> {
        let statement = self
            .0
            .prepare(
                "
            SELECT t.name, t.repetitions, t.interval_days, t.ease_factor, t.reviewed_at, t.categories
            FROM topics t
            WHERE ((?1) - t.reviewed_at) / 86400 >= t.interval_days
            ORDER BY t.reviewed_at ASC
            ",
            )
            .await?;

        let mut rows = statement.query([now as i64]).await?;
        let mut topics = Vec::new();
        while let Some(row) = rows.next().await? {
            topics.push(self.map(&row)?);
        }
        Ok(topics)
    }

    async fn get_all(&self) -> eyre::Result<Vec<Topic>> {
        let statement = self
            .0
            .prepare(
                "
            SELECT t.name, t.repetitions, t.interval_days, t.ease_factor, t.reviewed_at, t.categories
            FROM topics t
            ",
            )
            .await?;

        let mut rows = statement.query(()).await?;
        let mut topics = Vec::new();
        while let Some(row) = rows.next().await? {
            topics.push(self.map(&row)?);
        }
        Ok(topics)
    }

    async fn get(&self, topic: &str) -> eyre::Result<Option<Topic>> {
        let statement = self
            .0
            .prepare(
                "
            SELECT t.name, t.repetitions, t.interval_days, t.ease_factor, t.reviewed_at, t.categories
            FROM topics t
            WHERE t.name = (?1)
            ",
            )
            .await?;

        let mut rows = statement.query([topic]).await?;
        match rows.next().await? {
            Some(row) => Ok(Some(self.map(&row)?)),
            None => Ok(None),
        }
    }

    async fn upsert(&self, topic: &Topic) -> eyre::Result<()> {
        let categories = Self::categories_to_string(&topic.categories);
        self.0
            .execute(
                "
            INSERT INTO topics (name, repetitions, interval_days, ease_factor, reviewed_at, categories)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(name) DO UPDATE SET
                repetitions = excluded.repetitions,
                interval_days = excluded.interval_days,
                ease_factor = excluded.ease_factor,
                reviewed_at = excluded.reviewed_at,
                categories = excluded.categories
            ",
                params![
                    topic.name.clone(),
                    topic.repetitions as i64,
                    topic.interval as i64,
                    topic.ease_factor as f64,
                    topic.reviewed_at as i64,
                    categories
                ],
            )
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topic::Topic;

    #[tokio::test]
    async fn upsert_and_get() {
        let storage = LibsqlTopicStorage::init_inmemory().await.unwrap();

        storage.upsert(&Topic::new("test", 1000)).await.unwrap();

        let inserted = storage.get("test").await.unwrap().unwrap();

        assert_eq!(inserted.name, "test");
    }

    #[tokio::test]
    async fn get_all() {
        let storage = LibsqlTopicStorage::init_inmemory().await.unwrap();

        storage.upsert(&Topic::new("test 1", 1000)).await.unwrap();
        storage.upsert(&Topic::new("test 2", 1000)).await.unwrap();
        storage.upsert(&Topic::new("test 3", 1000)).await.unwrap();
        let topics = storage.get_all().await.unwrap();
        assert_eq!(topics.len(), 3);
    }

    #[tokio::test]
    async fn update_topic() {
        let storage = LibsqlTopicStorage::init_inmemory().await.unwrap();

        let mut topic = Topic::new("test 1", 1200);
        storage.upsert(&topic).await.unwrap();
        topic = storage.get("test 1").await.unwrap().unwrap();
        assert_eq!(topic.ease_factor, 2.5);
        assert_eq!(topic.repetitions, 0);
        assert_eq!(topic.interval, 1);

        topic = topic.update_quality(5, 2000);
        storage.upsert(&topic).await.unwrap();
        topic = storage.get("test 1").await.unwrap().unwrap();
        assert_eq!(topic.ease_factor, 2.6);
        assert_eq!(topic.repetitions, 1);
        assert_eq!(topic.interval, 6);

        let topics = storage.get_all().await.unwrap();
        assert_eq!(topics.len(), 1);
    }

    #[tokio::test]
    async fn get_overdue() {
        let storage = LibsqlTopicStorage::init_inmemory().await.unwrap();
        let today_seconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|t| t.as_secs())
            .unwrap();

        let two_days_ago = today_seconds - (2 * 86400);

        let mut overdue_topic = Topic::new("test 1", two_days_ago);
        overdue_topic = overdue_topic.update_quality(2, two_days_ago);
        storage.upsert(&overdue_topic).await.unwrap();

        let mut not_overdue_topic = Topic::new("test 2", two_days_ago);
        not_overdue_topic = not_overdue_topic.update_quality(5, two_days_ago);
        storage.upsert(&not_overdue_topic).await.unwrap();

        let topics = storage.get_overdue(today_seconds).await.unwrap();
        assert_eq!(topics.len(), 1);
        assert_eq!(topics.first().unwrap().name, "test 1");
    }
}
