use std::sync::Mutex;

use rusqlite::types::{FromSql, ToSqlOutput};
use rusqlite::{Connection, OptionalExtension, Row, ToSql, params};

use crate::category::Category;
use crate::sqlite::sqlite_storage::db_path;
use crate::storage_error::StorageError;
use crate::topic::TopicCategories;
use crate::{topic::Topic, topic_storage::TopicStorage};

pub struct SqliteTopicStorage(Mutex<Connection>);

impl SqliteTopicStorage {
    #[cfg(test)]
    pub fn init_inmemory() -> Result<Self, StorageError> {
        let connection = Connection::open_in_memory()?;
        let storage = SqliteTopicStorage(Mutex::new(connection));
        storage.create_tables()?;
        Ok(storage)
    }

    pub fn init() -> Result<Self, StorageError> {
        let path = db_path()?;
        let connection = Connection::open(path)?;
        let storage = SqliteTopicStorage(Mutex::new(connection));
        storage.create_tables()?;

        Ok(storage)
    }

    fn create_tables(&self) -> rusqlite::Result<()> {
        let conn = self.0.lock().unwrap();
        conn.execute_batch(
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
    }

    fn map_row_to_topic(&self, row: &Row) -> Result<Topic, rusqlite::Error> {
        Ok(Topic {
            name: row.get(0)?,
            repetitions: row.get(1)?,
            interval: row.get(2)?,
            ease_factor: row.get(3)?,
            reviewed_at: row.get::<_, i64>(4)? as u64,
            categories: row.get(5)?,
        })
    }
}

impl TopicStorage for SqliteTopicStorage {
    fn get_overdue(&self, now: u64) -> Result<Vec<Topic>, StorageError> {
        let conn = self.0.lock().unwrap();
        let mut statement = conn.prepare(
            "
            SELECT t.name, t.repetitions, t.interval_days, t.ease_factor, t.reviewed_at, t.categories
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

    fn get_all(&self) -> Result<Vec<Topic>, StorageError> {
        let conn = self.0.lock().unwrap();
        let mut statement = conn.prepare(
            "
            SELECT t.name, t.repetitions, t.interval_days, t.ease_factor, t.reviewed_at, t.categories
            FROM topics t
            ",
        )?;

        let topics = statement
            .query_map([], |row| self.map_row_to_topic(row))?
            .collect::<Result<Vec<Topic>, rusqlite::Error>>()?;

        Ok(topics)
    }

    fn get(&self, topic: &str) -> Result<Option<Topic>, StorageError> {
        let conn = self.0.lock().unwrap();
        let mut statement = conn.prepare(
            "
            SELECT t.name, t.repetitions, t.interval_days, t.ease_factor, t.reviewed_at, t.categories
            FROM topics t
            WHERE t.name = (?1)
            ",
        )?;

        let topic = statement
            .query_one::<Topic, _, _>([topic], |row| self.map_row_to_topic(row))
            .optional()?;

        Ok(topic)
    }

    fn upsert(&self, topic: &Topic) -> Result<(), StorageError> {
        let conn = self.0.lock().unwrap();

        conn.execute(
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
                topic.name,
                topic.repetitions,
                topic.interval,
                topic.ease_factor,
                topic.reviewed_at as i64,
                topic.categories
            ],
        )?;

        Ok(())
    }
}

impl FromSql for TopicCategories {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let values = value
            .as_str()?
            .split(",")
            .filter(|v| !v.is_empty())
            .map(|v| Category {
                name: v.to_string(),
            })
            .collect::<Vec<Category>>();

        Ok(TopicCategories(values))
    }
}

impl ToSql for TopicCategories {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        let value = &self
            .0
            .iter()
            .map(|v| v.name.to_string())
            .collect::<Vec<String>>()
            .join(",");
        rusqlite::Result::Ok(ToSqlOutput::from(value.clone()))
    }
}

#[cfg(test)]
mod tests {
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
        let today_seconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|t| t.as_secs())
            .unwrap();

        let two_days_ago = today_seconds - (2 * 86400);

        let mut overdue_topic = Topic::new("test 1", two_days_ago);
        overdue_topic = overdue_topic.update_quality(2, two_days_ago);
        storage.upsert(&overdue_topic).unwrap();

        let mut not_overdue_topic = Topic::new("test 2", two_days_ago);
        not_overdue_topic = not_overdue_topic.update_quality(5, two_days_ago);
        storage.upsert(&not_overdue_topic).unwrap();

        let topics = storage.get_overdue(today_seconds).unwrap();
        assert_eq!(topics.len(), 1);
        assert_eq!(topics.first().unwrap().name, "test 1");
    }
}
