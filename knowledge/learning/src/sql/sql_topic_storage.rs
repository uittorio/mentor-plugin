use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use eyre::OptionExt;
use libsql::{Row, params};

use crate::category::Category;
use crate::sql::sql_storage::SqlConnection;
use crate::topic::{Topic, TopicCategories};
use crate::topic_storage::TopicStorage;

pub struct SqlTopicStorage(pub Arc<SqlConnection>);

impl SqlTopicStorage {
    fn map(&self, row: &Row) -> eyre::Result<Topic> {
        let categories_raw: String = row.get(5)?;
        let categories = categories_raw
            .split(",")
            .filter(|v| !v.is_empty())
            .map(|v| Category {
                name: v.to_string(),
            })
            .collect::<Vec<Category>>();

        let reviewed_at = DateTime::from_timestamp_secs(row.get::<i64>(4)?)
            .ok_or_eyre("Expecting valid epoc for reviewed_at")?;

        Ok(Topic {
            name: row.get(0)?,
            repetitions: row.get::<i64>(1)? as u32,
            interval: row.get::<i64>(2)? as u32,
            ease_factor: row.get::<f64>(3)? as f32,
            reviewed_at: reviewed_at,
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
impl TopicStorage for SqlTopicStorage {
    async fn get_overdue(&self, now: DateTime<Utc>) -> eyre::Result<Vec<Topic>> {
        let statement = self
            .0
            .connection
            .prepare(
                "
            SELECT t.name, t.repetitions, t.interval_days, t.ease_factor, t.reviewed_at, t.categories
            FROM topics t
            WHERE ((?1) - t.reviewed_at) / 86400 >= t.interval_days
            ORDER BY t.reviewed_at ASC
            ",
            )
            .await?;

        let now_seconds = now.timestamp();

        let mut rows = statement.query([now_seconds]).await?;
        let mut topics = Vec::new();
        while let Some(row) = rows.next().await? {
            topics.push(self.map(&row)?);
        }
        Ok(topics)
    }

    async fn get_all(&self) -> eyre::Result<Vec<Topic>> {
        let statement = self
            .0
            .connection
            .prepare(
                "
            SELECT t.name, t.repetitions, t.interval_days, t.ease_factor, t.reviewed_at, t.categories
            FROM topics t
            ORDER BY t.reviewed_at DESC
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
            .connection
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
        self.0.connection
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
                    topic.reviewed_at.timestamp(),
                    categories
                ],
            )
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Days;

    use super::*;
    use crate::topic::Topic;

    #[tokio::test]
    async fn upsert_and_get() {
        let conn = SqlConnection::new_in_memory().await.unwrap();
        let storage = SqlTopicStorage(Arc::new(conn));

        storage
            .upsert(&Topic::new(
                "test",
                DateTime::from_timestamp_secs(1000).unwrap(),
            ))
            .await
            .unwrap();

        let inserted = storage.get("test").await.unwrap().unwrap();

        assert_eq!(inserted.name, "test");
    }

    #[tokio::test]
    async fn get_all() {
        let conn = SqlConnection::new_in_memory().await.unwrap();
        let storage = SqlTopicStorage(Arc::new(conn));

        storage
            .upsert(&Topic::new(
                "test 1",
                DateTime::from_timestamp_secs(1000).unwrap(),
            ))
            .await
            .unwrap();
        storage
            .upsert(&Topic::new(
                "test 2",
                DateTime::from_timestamp_secs(1000).unwrap(),
            ))
            .await
            .unwrap();
        storage
            .upsert(&Topic::new(
                "test 3",
                DateTime::from_timestamp_secs(1000).unwrap(),
            ))
            .await
            .unwrap();
        let topics = storage.get_all().await.unwrap();
        assert_eq!(topics.len(), 3);
    }

    #[tokio::test]
    async fn update_topic() {
        let conn = SqlConnection::new_in_memory().await.unwrap();
        let storage = SqlTopicStorage(Arc::new(conn));

        let mut topic = Topic::new("test 1", DateTime::from_timestamp_secs(1200).unwrap());
        storage.upsert(&topic).await.unwrap();
        topic = storage.get("test 1").await.unwrap().unwrap();
        assert_eq!(topic.ease_factor, 2.5);
        assert_eq!(topic.repetitions, 0);
        assert_eq!(topic.interval, 1);

        topic = topic.update_quality(5, DateTime::from_timestamp_secs(2000).unwrap());
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
        let conn = SqlConnection::new_in_memory().await.unwrap();
        let storage = SqlTopicStorage(Arc::new(conn));
        let today_seconds = Utc::now();

        let two_days_ago = today_seconds.checked_sub_days(Days::new(2)).unwrap();

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
