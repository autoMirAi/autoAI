use crate::error::{AppError, Result};
use async_trait::async_trait;
use tokio::io::{self, AsyncBufReadExt};

#[async_trait]
pub trait InputSource {
    async fn next(&mut self) -> Result<Option<String>>;
}

pub struct StdinInput {
    reader: io::BufReader<io::Stdin>,
}

impl StdinInput {
    pub fn new() -> Self {
        tracing::debug!("Initializing stdin input");
        Self {
            reader: io::BufReader::new(io::stdin()),
        }
    }
}

#[async_trait]
impl InputSource for StdinInput {
    async fn next(&mut self) -> Result<Option<String>> {
        let mut line = String::new();

        let n = self
            .reader
            .read_line(&mut line)
            .await
            .map_err(|e| AppError::Io(e))?;

        if n == 0 {
            tracing::debug!("Reached EOF");
            Ok(None)
        } else {
            let trimmed = line.trim().to_string();

            if trimmed.is_empty() {
                tracing::trace!("Skipping empty line");
                return self.next().await;
            }

            tracing::trace!("Read input: {} chars", trimmed.len());
            Ok(Some(trimmed))
        }
    }
}

impl Default for StdinInput {
    fn default() -> Self {
        Self::new()
    }
}
