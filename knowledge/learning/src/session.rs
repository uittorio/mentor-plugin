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
    pub modified_at: u64,
}
