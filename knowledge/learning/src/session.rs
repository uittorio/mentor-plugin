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
    pub created_at: u64,
    pub file_name: Option<String>,
    pub modified_at: u64,
    pub content: Option<String>,
}

impl Session {
    pub fn new(name: &str, file_name: &str, created_at: u64) -> Self {
        Session {
            id: SessionId::new(),
            name: name.to_string(),
            file_name: Some(file_name.to_string()),
            created_at,
            modified_at: created_at,
            content: None,
        }
    }

    pub fn update_content(&self, content: &String, now: u64) -> Session {
        Session {
            id: self.id.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            file_name: self.file_name.clone(),
            modified_at: now,
            content: Some(content.to_string()),
        }
    }
}
