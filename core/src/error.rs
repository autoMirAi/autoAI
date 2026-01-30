use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Input error: {0}")]
    Input(String),

    #[error("Output error: {0}")]
    Output(String),

    #[error("Audio error: {0}")]
    Audio(String),

    #[error("Speech recognition error: {0]")]
    SpeechRecognition(String),

    #[error("No audio device found")]
    NoAudioDevice,

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Timeout after {seconds} seconds")]
    Timeout { seconds: u64 },

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Stream ended unexpectedly")]
    StreamEnded,

    #[error("Retry limit exceeded: {attempts} attempts")]
    RetryExhausted { attempts: u32 },

    #[error("Operation cancelled")]
    Cancelled,
}

impl AppError {
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    pub fn llm(msg: impl Into<String>) -> Self {
        Self::Llm(msg.into())
    }

    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    pub fn service_unvailable(msg: impl Into<String>) -> Self {
        Self::ServiceUnavailable(msg.into())
    }

    pub fn audio(msg: impl Into<String>) -> Self {
        Self::Audio(msg.into())
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Http(_) | Self::Timeout { .. } | Self::ServiceUnavailable(_)
        )
    }
}

impl From<config::ConfigError> for AppError {
    fn from(err: config::ConfigError) -> Self {
        AppError::Config(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
