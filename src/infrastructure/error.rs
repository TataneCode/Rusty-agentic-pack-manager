use thiserror::Error;

#[derive(Debug, Error)]
pub enum InfrastructureError {
    #[error("packs directory not found: {0}")]
    PacksDirNotFound(String),
    #[error("io error on {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
}
