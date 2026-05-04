use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone)]
pub struct SessionId(pub Uuid);

impl SessionId {
    pub fn new() -> Self {
        SessionId(Uuid::new_v4())
    }
}

pub struct Session {
    pub id: SessionId,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub content: Option<String>,
}

impl Session {
    pub fn new(name: &str, created_at: DateTime<Utc>) -> Self {
        Session {
            id: SessionId::new(),
            name: name.to_string(),
            created_at,
            modified_at: created_at,
            content: None,
        }
    }

    pub fn update_content(self, content: &String, now: DateTime<Utc>) -> Session {
        Session {
            id: self.id.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            modified_at: now,
            content: Some(content.to_string()),
        }
    }
}
