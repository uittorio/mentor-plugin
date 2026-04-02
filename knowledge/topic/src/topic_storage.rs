use std::error::Error;

use crate::Topic;

pub trait TopicStorage {
    fn get_overdue(&self, now: u64) -> Result<Vec<Topic>, Box<dyn Error>>;
    fn get_all(&self) -> Result<Vec<Topic>, Box<dyn Error>>;
    fn get(&self, topic: &str) -> Result<Option<Topic>, Box<dyn Error>>;
    fn upsert(&self, topic: &Topic) -> Result<(), Box<dyn Error>>;
}
