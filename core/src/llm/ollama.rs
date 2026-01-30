use crate::error::{AppError, Result};
use crate::llm::{LlmProvider, ResponseStream, StreamChunk};
use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
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
        Self::validate_config(base_url, model_name)?;

        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            model_name: model_name.to_string(),
            max_retries,
        })
    }

    fn validate_config(base_url: &str, model_name: &str) -> Result<()> {
        if base_url.is_empty() {
            return Err(AppError::invalid_input("base url can not be empty"));
        }
        if model_name.is_empty() {
            return Err(AppError::invalid_input("model_name can not be empty"));
        }
        Ok(())
    }

    pub async fn chat_stream_with_retry(&self, prompt: &str) -> Result<ResponseStream> {
        let mut last_error = None;

        for attempt in 1..=self.max_retries {
            match self.send_chat_request(prompt).await {
                Ok(stream) => {
                    if attempt > 1 {
                        tracing::info!("Request succeeded on attempt {}", attempt);
                    }
                    return Ok(stream);
                }
                Err(e) if e.is_retryable() && attempt < self.max_retries => {
                    tracing::warn!("Attempt {}/{} failed: {}", attempt, self.max_retries, e);
                    last_error = Some(e);

                    let delay = Duration::from_millis(100 * 2_u64.pow(attempt - 1));
                    tracing::debug!("Retrying after {:?}", delay);
                    tokio::time::sleep(delay).await;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Err(last_error.unwrap_or(AppError::RetryExhausted {
            attempts: self.max_retries,
        }))
    }

    async fn send_chat_request(&self, prompt: &str) -> Result<ResponseStream> {
        tracing::debug!("Sending prompt to Ollama (length: {})", prompt.len());

        let url = format!("{}/api/generate", self.base_url);
        let request = GenerateRequest {
            model: self.model_name.clone(),
            prompt: prompt.to_string(),
            stream: true,
        };

        let resp = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| AppError::llm(format!("API error: {}", e)))?;

        let stream = resp.bytes_stream().map(|item| {
            let bytes = item.map_err(|e| AppError::Http(e))?;

            let response: GenerateResponse =
                serde_json::from_slice(&bytes).map_err(|e| AppError::Json(e))?;

            if let Some(error) = response.error {
                return Err(AppError::Llm(error));
            }

            Ok(StreamChunk {
                text: response.response,
                done: response.done,
            })
        });

        Ok(Box::pin(stream))
    }
}

#[async_trait]
impl LlmProvider for OllamaClient {
    async fn health_check(&self) -> Result<()> {
        tracing::debug!("Cehck Ollama service health: {}", self.base_url);

        self.client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map_err(|e| AppError::service_unvailable(format!("connect failed: {}", e)))?
            .error_for_status()
            .map_err(|e| AppError::service_unvailable(format!("health check failed: {}", e)))?;

        tracing::info!("Ollama service normal");
        Ok(())
    }

    async fn chat(&self, prompt: &str) -> Result<ResponseStream> {
        if prompt.trim().is_empty() {
            return Err(AppError::invalid_input("Prompt can not be empty"));
        }

        self.chat_stream_with_retry(prompt).await
    }

    fn name(&self) -> &str {
        "ollama"
    }
}
