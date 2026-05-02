use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::topic::Topic;

#[async_trait]
pub trait TopicStorage {
    async fn get_overdue(&self, now: DateTime<Utc>) -> eyre::Result<Vec<Topic>>;
    async fn get_all(&self) -> eyre::Result<Vec<Topic>>;
    async fn get(&self, topic: &str) -> eyre::Result<Option<Topic>>;
    async fn upsert(&self, topic: &Topic) -> eyre::Result<()>;
}
