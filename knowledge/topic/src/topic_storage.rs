use std::{error::Error, fmt::Display};

use crate::Topic;

pub trait TopicStorage {
    fn get_overdue(&self, now: u64) -> Result<Vec<Topic>, TopicStorageError>;
    fn get_all(&self) -> Result<Vec<Topic>, TopicStorageError>;
    fn get(&self, topic: &str) -> Result<Option<Topic>, TopicStorageError>;
    fn upsert(&self, topic: &Topic) -> Result<(), TopicStorageError>;
}

#[derive(Debug)]
pub struct TopicStorageError {
    pub message: String,
    pub source: Option<Box<dyn Error + Send + Sync>>,
}

impl Display for TopicStorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for TopicStorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as &dyn Error)
    }
}
