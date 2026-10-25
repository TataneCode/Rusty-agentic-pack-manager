use thiserror::Error;

use crate::domain::pack::error::DomainError;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("domain error: {0}")]
    Domain(#[from] DomainError),
    #[error("pack not found: {0}")]
    PackNotFound(String),
    #[error("file not found: {0}")]
    FileNotFound(String),
    #[error("not a directory: {0}")]
    NotADirectory(String),
    #[error("invalid pack manifest {path}: {message}")]
    InvalidManifest { path: String, message: String },
    #[error("destination already exists: {0}")]
    DestinationExists(String),
    #[error("io error on {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
}
