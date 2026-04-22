use async_trait::async_trait;

use crate::topic::Topic;

#[async_trait]
pub trait TopicStorage {
    async fn get_overdue(&self, now: u64) -> eyre::Result<Vec<Topic>>;
    async fn get_all(&self) -> eyre::Result<Vec<Topic>>;
    async fn get(&self, topic: &str) -> eyre::Result<Option<Topic>>;
    async fn upsert(&self, topic: &Topic) -> eyre::Result<()>;
}
