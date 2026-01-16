use crate::error::{AppError, Result};
use futures_util::Stream;
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;

#[derive(Debug, Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct GenerateResponse {
    #[serde(default)]
    response: String,
    #[serde(default)]
    done: bool,
    #[serde(default)]
    error: Option<String>,
}

pub struct OllamaClient {
    client: Client,
    base_url: String,
    model_name: String,
    max_retries: u32,
}

impl OllamaClient {
    pub fn new(
        base_url: &str,
        model_name: &str,
        timeout_secs: u64,
        max_retries: u32,
    ) -> Result<Self> {
        if base_url.is_empty() {
            return Err(AppError::InvalidInput(
                "base_url cannot be empty".to_string(),
            ));
        }
        if model_name.is_empty() {
            return Err(AppError::InvalidInput(
                "model_name cannot be empty".to_string(),
            ));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| AppError::Http(e))?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            model_name: model_name.to_string(),
            max_retries,
        })
    }

    pub async fn health_check(&self) -> Result<()> {
        tracing::debug!("Checking Ollama health at {}", self.base_url);

        self.client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map_err(|e| AppError::ServiceUnavailable(format!("Cannot connect: {}", e)))?
            .error_for_status()
            .map_err(|e| AppError::ServiceUnavailable(format!("Health check failed: {}", e)))?;

        tracing::info!("Ollama service is healthy");
        Ok(())
    }

    pub async fn chat_stream(&self, prompt: String) -> Result<OllamaStream> {
        if prompt.trim().is_empty() {
            return Err(AppError::InvalidInput("Prompt cannot be empty".to_string()));
        }

        tracing::debug!("Sending prompt to Ollama (length: {})", prompt.len());

        let url = format!("{}/api/generate", self.base_url);
        let request = GenerateRequest {
            model: self.model_name.clone(),
            prompt,
            stream: true,
        };

        let resp = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Http(e))?
            .error_for_status()
            .map_err(|e| AppError::Llm(format!("API error: {}", e)))?;

        let stream = resp.bytes_stream().map(|item| {
            let bytes = item.map_err(|e| AppError::Http(e))?;

            let response: GenerateResponse =
                serde_json::from_slice(&bytes).map_err(|e| AppError::Json(e))?;

            if let Some(error) = response.error {
                return Err(AppError::Llm(error));
            }

            Ok(OllamaChunk {
                text: response.response,
                done: response.done,
            })
        });

        Ok(Box::pin(stream))
    }

    pub async fn chat_stream_with_retry(&self, prompt: String) -> Result<OllamaStream> {
        let mut last_error = None;

        for attempt in 1..=self.max_retries {
            match self.chat_stream(prompt.clone()).await {
                Ok(stream) => {
                    if attempt > 1 {
                        tracing::info!("Request succeeded on attempt {}", attempt);
                    }
                    return Ok(stream);
                }
                Err(e) => {
                    tracing::warn!("Attempt {}/{} failed: {}", attempt, self.max_retries, e);
                    last_error = Some(e);

                    if attempt < self.max_retries {
                        let delay = Duration::from_millis(100 * 2_u64.pow(attempt - 1));
                        tracing::debug!("Retrying after {:?}", delay);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or(AppError::RetryLimitExceeded {
            attempts: self.max_retries,
        }))
    }
}

#[derive(Debug)]
pub struct OllamaChunk {
    pub text: String,
    pub done: bool,
}

pub type OllamaStream = Pin<Box<dyn Stream<Item = Result<OllamaChunk>> + Send>>;
