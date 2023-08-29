use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppError(pub String);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProofError(pub String);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Error {
    ProofError(ProofError),
    Unknown,
    AppError(AppError),
}

impl Error {
    fn default() -> Self {
        Error::Unknown
    }
}
