use crate::{
    session::{Session, SessionId},
    storage_error::StorageError,
};

pub trait SessionStorage {
    fn create(&self, session: &Session) -> Result<(), StorageError>;
    fn get(&self, session: &SessionId) -> Result<Option<Session>, StorageError>;
}
