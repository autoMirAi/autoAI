use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),

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

    #[allow(dead_code)]
    #[error("Stream ended unexpectedly")]
    StreamEnded,

    #[allow(dead_code)]
    #[error("Operation cancelled")]
    Cancelled,

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[allow(dead_code)]
    #[error("Timeout after {0} seconds")]
    Timeout(u64),

    #[error("Retry limit exceeded: {attempts} attempts")]
    RetryLimitExceeded { attempts: u32 },
}

impl From<config::ConfigError> for AppError {
    fn from(err: config::ConfigError) -> Self {
        AppError::Config(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
