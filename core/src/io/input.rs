use crate::error::Result;
use async_trait::async_trait;
use tokio::io::{self, AsyncBufReadExt};

#[async_trait]
pub trait InputSource: Send {
    async fn next(&mut self) -> Result<Option<String>>;
}

pub struct TextInput {
    reader: io::BufReader<io::Stdin>,
}

impl TextInput {
    pub fn new() -> Self {
        tracing::debug!("Initializing stdin input");
        Self {
            reader: io::BufReader::new(io::stdin()),
        }
    }
}

#[async_trait]
impl InputSource for TextInput {
    async fn next(&mut self) -> Result<Option<String>> {
        let mut line = String::new();

        let byte_read = self.reader.read_line(&mut line).await?;

        if byte_read == 0 {
            tracing::debug!("Reached EOF");
            return Ok(None);
        }

        let trimmed = line.trim().to_string();

        if trimmed.is_empty() {
            tracing::trace!("Skipping empty line");
            return self.next().await;
        }

        tracing::trace!("Read input: {} chars", trimmed.len());
        Ok(Some(trimmed))
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}
