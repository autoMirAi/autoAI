pub mod ollama;

use crate::error::Result;
use async_trait::async_trait;
use futures_util::Stream;
use std::pin::Pin;

pub use ollama::OllamaClient;

#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub text: String,
    pub done: bool,
}

pub type ResponseStream = Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn health_check(&self) -> Result<()>;

    async fn chat(&self, prompt: &str) -> Result<ResponseStream>;

    fn name(&self) -> &str;
}
