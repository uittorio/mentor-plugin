use std::{path::Path, sync::Mutex};

use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::{
    topic::Topic,
    topic_storage::{TopicStorage, TopicStorageError},
};

pub struct SqliteTopicStorage(Mutex<Connection>);

fn db_path() -> std::io::Result<String> {
    let folder = std::env::var("AGENT_MENTOR_DB_FOLDER").unwrap_or_else(|_| {
        let home = std::env::var("HOME").expect("HOME not set");

        format!("{}/.local/share/agent-mentor", home)
    });

    std::fs::create_dir_all(&folder)?;

    Ok(Path::new(&folder)
        .join("knowledge.db")
        .to_str()
        .unwrap()
        .to_string())
}

impl SqliteTopicStorage {
    #[cfg(test)]
    pub fn init_inmemory() -> Result<Self, TopicStorageError> {
        let connection = Connection::open_in_memory()?;
        let storage = SqliteTopicStorage(Mutex::new(connection));
        storage.create_tables()?;
        Ok(storage)
    }

    pub fn init() -> Result<Self, TopicStorageError> {
        let path = db_path()?;
        let connection = Connection::open(path)?;
        let storage = SqliteTopicStorage(Mutex::new(connection));
        storage.create_tables()?;

        Ok(storage)
    }

    fn create_tables(&self) -> rusqlite::Result<()> {
        let conn = self.0.lock().unwrap();
        conn.execute_batch(
            "BEGIN;
            CREATE TABLE IF NOT EXISTS topics (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              name TEXT NOT NULL UNIQUE COLLATE NOCASE,
              created_at INTEGER NOT NULL DEFAULT (unixepoch()),
              ease_factor REAL NOT NULL DEFAULT 2.5,
              interval_days INTEGER NOT NULL DEFAULT 0,
              repetitions INTEGER NOT NULL DEFAULT 0,
              reviewed_at INTEGER NOT NULL
            );
            COMMIT;
            ",
        )
    }

    fn map_row_to_topic(&self, row: &Row) -> Result<Topic, rusqlite::Error> {
        Ok(Topic {
            name: row.get(0)?,
            repetitions: row.get(1)?,
            interval: row.get(2)?,
            ease_factor: row.get(3)?,
            reviewed_at: row.get::<_, i64>(4)? as u64,
        })
    }
}

impl TopicStorage for SqliteTopicStorage {
    fn get_overdue(&self, now: u64) -> Result<Vec<Topic>, TopicStorageError> {
        let conn = self.0.lock().unwrap();
        let mut statement = conn.prepare(
            "
            SELECT t.name, t.repetitions, t.interval_days, t.ease_factor, t.reviewed_at
            FROM topics t
            WHERE ((?1) - t.reviewed_at) / 86400 >= t.interval_days
            ORDER BY t.reviewed_at ASC
            ",
        )?;

        let topics = statement
            .query_map([now as i64], |row| self.map_row_to_topic(row))?
            .collect::<Result<Vec<Topic>, rusqlite::Error>>()?;

        Ok(topics)
    }

    fn get_all(&self) -> Result<Vec<Topic>, TopicStorageError> {
        let conn = self.0.lock().unwrap();
        let mut statement = conn.prepare(
            "
            SELECT t.name, t.repetitions, t.interval_days, t.ease_factor, t.reviewed_at
            FROM topics t
            ",
        )?;

        let topics = statement
            .query_map([], |row| self.map_row_to_topic(row))?
            .collect::<Result<Vec<Topic>, rusqlite::Error>>()?;

        Ok(topics)
    }

    fn get(&self, topic: &str) -> Result<Option<Topic>, TopicStorageError> {
        let conn = self.0.lock().unwrap();
        let mut statement = conn.prepare(
            "
            SELECT t.name, t.repetitions, t.interval_days, t.ease_factor, t.reviewed_at
            FROM topics t
            WHERE t.name = (?1)
            ",
        )?;

        let topic = statement
            .query_one::<Topic, _, _>([topic], |row| self.map_row_to_topic(row))
            .optional()?;

        Ok(topic)
    }

    fn upsert(&self, topic: &Topic) -> Result<(), TopicStorageError> {
        let conn = self.0.lock().unwrap();

        conn.execute(
            "
            INSERT INTO topics (name, repetitions, interval_days, ease_factor, reviewed_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(name) DO UPDATE SET
                repetitions = excluded.repetitions,
                interval_days = excluded.interval_days,
                ease_factor = excluded.ease_factor,
                reviewed_at = excluded.reviewed_at
            ",
            params![
                topic.name,
                topic.repetitions,
                topic.interval,
                topic.ease_factor,
                topic.reviewed_at as i64,
            ],
        )?;

        Ok(())
    }
}

impl From<std::io::Error> for TopicStorageError {
    fn from(value: std::io::Error) -> Self {
        TopicStorageError {
            message: "TopicStorageError stdio error".to_string(),
            source: Some(Box::new(value)),
        }
    }
}
impl From<rusqlite::Error> for TopicStorageError {
    fn from(value: rusqlite::Error) -> Self {
        TopicStorageError {
            message: "TopicStorageError sqlite error".to_string(),
            source: Some(Box::new(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn upsert_and_get() {
        let storage = SqliteTopicStorage::init_inmemory().unwrap();

        storage.upsert(&Topic::new("test", 1000)).unwrap();

        let inserted = storage.get("test").unwrap().unwrap();

        assert_eq!(inserted.name, "test")
    }

    #[test]
    fn table_not_recreated() {
        let storage = SqliteTopicStorage::init_inmemory().unwrap();

        assert!(storage.create_tables().is_ok());
    }

    #[test]
    fn get_all() {
        let storage = SqliteTopicStorage::init_inmemory().unwrap();

        storage.upsert(&Topic::new("test 1", 1000)).unwrap();
        storage.upsert(&Topic::new("test 2", 1000)).unwrap();
        storage.upsert(&Topic::new("test 3", 1000)).unwrap();
        let topics = storage.get_all().unwrap();
        assert_eq!(topics.len(), 3);
    }

    #[test]
    fn update_topic() {
        let storage = SqliteTopicStorage::init_inmemory().unwrap();

        let mut topic = Topic::new("test 1", 1200);
        storage.upsert(&topic).unwrap();
        topic = storage.get("test 1").unwrap().unwrap();
        assert_eq!(topic.ease_factor, 2.5);
        assert_eq!(topic.repetitions, 0);
        assert_eq!(topic.interval, 1);

        topic = topic.update_quality(5, 2000);
        storage.upsert(&topic).unwrap();
        topic = storage.get("test 1").unwrap().unwrap();
        assert_eq!(topic.ease_factor, 2.6);
        assert_eq!(topic.repetitions, 1);
        assert_eq!(topic.interval, 6);

        let topics = storage.get_all().unwrap();
        assert_eq!(topics.len(), 1);
    }

    #[test]
    fn get_overdue() {
        let storage = SqliteTopicStorage::init_inmemory().unwrap();
        let today = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        let two_days_ago = today.as_secs() - (2 * 86400);

        let mut overdue_topic = Topic::new("test 1", two_days_ago);
        overdue_topic = overdue_topic.update_quality(2, two_days_ago);
        storage.upsert(&overdue_topic).unwrap();

        let mut not_overdue_topic = Topic::new("test 2", two_days_ago);
        not_overdue_topic = not_overdue_topic.update_quality(5, two_days_ago);
        storage.upsert(&not_overdue_topic).unwrap();

        let topics = storage.get_overdue(today.as_secs()).unwrap();
        assert_eq!(topics.len(), 1);
        assert_eq!(topics.first().unwrap().name, "test 1");
    }
}
