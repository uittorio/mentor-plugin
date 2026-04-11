use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct StorageError {
    pub message: String,
    pub source: Option<Box<dyn Error + Send + Sync>>,
}

impl Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(s) = &self.source {
            return write!(f, "{} {}", self.message, s);
        }

        write!(f, "{}", self.message)
    }
}

impl Error for StorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as &dyn Error)
    }
}
