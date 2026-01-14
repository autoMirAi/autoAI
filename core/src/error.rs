use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Stream ended unexpectedly")]
    StreamEnded,

    #[error("Operation cancelled")]
    Cancelled,

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Unknown(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
