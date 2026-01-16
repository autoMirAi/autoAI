use crate::config::OllamaConfig;
use crate::error::{AppError, Result};
use crate::llm::ollama::{OllamaClient, OllamaStream};

pub struct Agent {
    ollama: OllamaClient,
}

impl Agent {
    pub fn new(cfg: &OllamaConfig) -> Result<Self> {
        tracing::info!("Initializing agent with model: {}", cfg.model_name);

        let ollama = OllamaClient::new(
            &cfg.base_url,
            &cfg.model_name,
            cfg.timeout_secs,
            cfg.max_retries,
        )?;

        Ok(Self { ollama })
    }

    pub async fn health_check(&self) -> Result<()> {
        self.ollama.health_check().await
    }

    pub async fn handle_input(&mut self, text: String) -> Result<OllamaStream> {
        if text.trim().is_empty() {
            return Err(AppError::InvalidInput("Input cannot be empty".to_string()));
        }

        if text.len() > 10000 {
            return Err(AppError::InvalidInput(
                "Input too long (max 10000 characters)".to_string(),
            ));
        }

        tracing::info!("Processing input: {} chars", text.len());

        self.ollama.chat_stream_with_retry(text).await
    }
}
