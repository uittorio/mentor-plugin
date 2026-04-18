use crate::{storage_error::StorageError, topic::Topic};

pub trait TopicStorage {
    fn get_overdue(&self, now: u64) -> Result<Vec<Topic>, StorageError>;
    fn get_all(&self) -> Result<Vec<Topic>, StorageError>;
    fn get(&self, topic: &str) -> Result<Option<Topic>, StorageError>;
    fn upsert(&self, topic: &Topic) -> Result<(), StorageError>;
    fn set_topic_categories(&self, topic: &str, categories: &[String]) -> Result<(), StorageError>;
    fn get_categories(&self) -> Result<Vec<String>, StorageError>;
}
