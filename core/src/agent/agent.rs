use crate::llm::ollama::{OllamaClient, OllamaStream};
use crate::config::OllamaConfig;

pub struct Agent {
    ollama: OllamaClient,
}

impl Agent {
    pub fn new(cfg: &OllamaConfig) -> Self {
        Self {
            ollama: OllamaClient::new(&cfg.base_url, &cfg.model_name),
        }
    }

    pub async fn handle_input(
        &mut self,
        text: String,
    ) -> anyhow::Result<OllamaStream> {
        self.ollama.chat_stream(text).await
    }
}
