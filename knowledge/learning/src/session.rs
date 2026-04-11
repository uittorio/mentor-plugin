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
    pub file_path: String,
    pub modified_at: u64,
}

impl Session {
    pub fn new(name: &str, file_path: &str, created_at: u64) -> Self {
        Session {
            id: SessionId::new(),
            name: name.to_string(),
            file_path: file_path.to_string(),
            created_at,
            modified_at: created_at,
        }
    }
}
