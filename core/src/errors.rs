use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize, Clone)]
pub enum StateError {
    #[error("Update to state failed.")]
    Update,
    #[error("Corrupted state.")]
    ErroneousState,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DBError(pub String);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Error {
    StateError(StateError),
    Unknown,
    DBError(DBError),
}

impl Error {
    pub fn default() -> Self {
        Error::Unknown
    }
}
