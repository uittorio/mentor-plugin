use std::sync::Mutex;

use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::sqlite::sqlite_storage::db_path;
use crate::storage_error::StorageError;
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
              reviewed_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS categories (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              name TEXT NOT NULL UNIQUE COLLATE NOCASE
            );
            CREATE TABLE IF NOT EXISTS topic_categories (
              topic_id INTEGER NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
              category_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
              PRIMARY KEY (topic_id, category_id)
            );
            COMMIT;
            ",
        )
    }

    fn map_row_to_topic(&self, row: &Row) -> Result<Topic, rusqlite::Error> {
        let categories_str: Option<String> = row.get(5)?;
        let categories = categories_str
            .map(|s| s.split('|').map(String::from).collect())
            .unwrap_or_default();
        Ok(Topic {
            name: row.get(0)?,
            repetitions: row.get(1)?,
            interval: row.get(2)?,
            ease_factor: row.get(3)?,
            reviewed_at: row.get::<_, i64>(4)? as u64,
            categories,
        })
    }
}

impl TopicStorage for SqliteTopicStorage {
    fn get_overdue(&self, now: u64) -> Result<Vec<Topic>, StorageError> {
        let conn = self.0.lock().unwrap();
        let mut statement = conn.prepare(
            "
            SELECT t.name, t.repetitions, t.interval_days, t.ease_factor, t.reviewed_at,
                   GROUP_CONCAT(c.name, '|') as categories
            FROM topics t
            LEFT JOIN topic_categories tc ON tc.topic_id = t.id
            LEFT JOIN categories c ON c.id = tc.category_id
            WHERE ((?1) - t.reviewed_at) / 86400 >= t.interval_days
            GROUP BY t.id
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
            SELECT t.name, t.repetitions, t.interval_days, t.ease_factor, t.reviewed_at,
                   GROUP_CONCAT(c.name, '|') as categories
            FROM topics t
            LEFT JOIN topic_categories tc ON tc.topic_id = t.id
            LEFT JOIN categories c ON c.id = tc.category_id
            GROUP BY t.id
            ORDER BY t.name COLLATE NOCASE
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
            SELECT t.name, t.repetitions, t.interval_days, t.ease_factor, t.reviewed_at,
                   GROUP_CONCAT(c.name, '|') as categories
            FROM topics t
            LEFT JOIN topic_categories tc ON tc.topic_id = t.id
            LEFT JOIN categories c ON c.id = tc.category_id
            WHERE t.name = (?1)
            GROUP BY t.id
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

    fn set_topic_categories(&self, topic_name: &str, categories: &[String]) -> Result<(), StorageError> {
        let conn = self.0.lock().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        let topic_id: i64 = conn.query_row(
            "SELECT id FROM topics WHERE name = ?1 COLLATE NOCASE",
            [topic_name],
            |row| row.get(0),
        )?;

        conn.execute(
            "DELETE FROM topic_categories WHERE topic_id = ?1",
            [topic_id],
        )?;

        for cat in categories {
            let cat = cat.trim();
            if cat.is_empty() {
                continue;
            }
            conn.execute(
                "INSERT OR IGNORE INTO categories (name) VALUES (?1)",
                [cat],
            )?;
            let cat_id: i64 = conn.query_row(
                "SELECT id FROM categories WHERE name = ?1 COLLATE NOCASE",
                [cat],
                |row| row.get(0),
            )?;
            conn.execute(
                "INSERT OR IGNORE INTO topic_categories (topic_id, category_id) VALUES (?1, ?2)",
                rusqlite::params![topic_id, cat_id],
            )?;
        }

        Ok(())
    }

    fn get_categories(&self) -> Result<Vec<String>, StorageError> {
        let conn = self.0.lock().unwrap();
        let mut statement = conn.prepare(
            "SELECT name FROM categories ORDER BY name COLLATE NOCASE",
        )?;
        let cats = statement
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<String>, rusqlite::Error>>()?;
        Ok(cats)
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
