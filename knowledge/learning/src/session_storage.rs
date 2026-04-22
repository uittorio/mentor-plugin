use async_trait::async_trait;

use crate::session::{Session, SessionId};

#[async_trait]
pub trait SessionStorage {
    async fn create(&self, session: &Session) -> eyre::Result<()>;
    async fn update(&self, session: &Session) -> eyre::Result<()>;
    async fn get(&self, session_id: &SessionId) -> eyre::Result<Option<Session>>;
    async fn get_all(&self) -> eyre::Result<Vec<Session>>;
}
