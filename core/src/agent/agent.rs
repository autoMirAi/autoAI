use crate::config::OllamaConfig;
use crate::error::{AppError, Result};
use crate::llm::{LlmProvider, OllamaClient, ResponseStream};

pub struct Agent {
    llm: Box<dyn LlmProvider>,
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

        Ok(Self::with_provider(Box::new(ollama)))
    }

    pub fn with_provider(llm: Box<dyn LlmProvider>) -> Self {
        Self { llm }
    }

    pub async fn health_check(&self) -> Result<()> {
        self.llm.health_check().await
    }

    pub async fn process(&self, text: &str) -> Result<ResponseStream> {
        self.validate_input(text)?;
        tracing::info!("Processing input: {} chars", text.len());
        self.llm.chat(text).await
    }

    fn validate_input(&self, text: &str) -> Result<()> {
        const MAX_INPUT_LENGTH: usize = 10000;

        if text.trim().is_empty() {
            return Err(AppError::invalid_input("Input cannot be empty"));
        }

        if text.len() > MAX_INPUT_LENGTH {
            return Err(AppError::invalid_input(format!(
                "Input too long (max {} characters)",
                MAX_INPUT_LENGTH
            )));
        }

        Ok(())
    }
}
