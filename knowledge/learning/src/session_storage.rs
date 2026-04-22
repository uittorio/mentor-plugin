use crate::{
    session::{Session, SessionId},
    storage_error::StorageError,
};

pub trait SessionStorage {
    fn create(&self, session: &Session) -> Result<(), StorageError>;
    fn update(&self, session: &Session) -> Result<(), StorageError>;
    fn get(&self, session: &SessionId) -> Result<Option<Session>, StorageError>;
    fn get_all(&self) -> Result<Vec<Session>, StorageError>;
}
