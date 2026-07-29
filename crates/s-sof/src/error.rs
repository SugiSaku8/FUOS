use thiserror::Error;

/// Result type used by s-sof.
pub type Result<T> = std::result::Result<T, SofError>;

/// Errors produced by SOF operations.
#[derive(Debug, Error)]
pub enum SofError {
    /// Standard IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Error from SIO layer.
    #[error("SIO error: {0}")]
    Sio(#[from] s_io::SioError),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Invalid SOF format.
    #[error("Invalid SOF format")]
    InvalidFormat,

    /// Custom message.
    #[error("{0}")]
    Message(String),
}
