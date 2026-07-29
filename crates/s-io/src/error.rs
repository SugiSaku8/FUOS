use thiserror::Error;

#[derive(Debug, Error)]
pub enum SioError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid SIO format")]
    InvalidFormat,

    #[error("Dataset not found: {0}")]
    DatasetNotFound(String),
}
