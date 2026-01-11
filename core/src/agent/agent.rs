use crate::llm::ollama::{OllamaClient, OllamaStream};

pub struct Agent {
    ollama: OllamaClient,
}

impl Agent {
    pub fn new() -> Self {
        Self {
            ollama: OllamaClient::new("http://localhost:11434", "llama3.1:8b"),
        }
    }

    pub async fn handle_input(
        &mut self,
        text: String,
    ) -> anyhow::Result<OllamaStream> {
        self.ollama.chat_stream(text).await
    }
}
