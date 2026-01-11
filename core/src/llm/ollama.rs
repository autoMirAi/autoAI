use anyhow::Result;
use futures_util::Stream;
use futures_util::StreamExt;
use reqwest::Client;
use serde_json::json;
use std::pin::Pin;

pub struct OllamaClient {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaClient {
    pub fn new(base_url: &str, model: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            model: model.to_string(),
        }
    }

    pub async fn chat_stream(&self, prompt: String) -> Result<OllamaStream> {
        let resp = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&json!({
                "model": self.model,
                "prompt": prompt,
                "stream": true
            }))
            .send()
            .await?
            .error_for_status()?;

        let stream = resp.bytes_stream().map(|item| {
            let bytes = item?;
            let text = String::from_utf8_lossy(&bytes).to_string();
            Ok(text)
        });

        Ok(Box::pin(stream))
    }
}

pub type OllamaStream =
    Pin<Box<dyn Stream<Item = Result<String>> + Send>>;
