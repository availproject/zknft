use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum StateError {
    #[error("Update to state failed.")]
    Update,
    #[error("Corrupted state.")]
    ErroneousState,
}

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum Error {
    #[error("State error: {0}.")]
    StateError(#[from] StateError),
    #[error("Unknown error")]
    Unknown,
}

impl Error {
    fn default() -> Self {
        Error::Unknown
    }
}
