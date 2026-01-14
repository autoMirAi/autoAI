use anyhow::{Context, Result};
use async_trait::async_trait;
use tokio::io::{self, AsyncWriteExt};

#[async_trait]
pub trait OutputSink {
    async fn emit(&mut self, text: &str) -> Result<()>;
    async fn emit_inline(&mut self, text: &str) -> Result<()>;
    async fn emit_error(&mut self, error: &str) -> Result<()>;
}

pub struct StdoutOutput {
    buffer: Vec<u8>,
}

impl StdoutOutput {
    pub fn new() -> Self {
        tracing::debug!("Initializing stdout output");
        Self {
            buffer: Vec::with_capacity(4096),
        }
    }

    async fn flush_buffer(&mut self) -> Result<()> {
        if !self.buffer.is_empty() {
            let mut stdout = io::stdout();
            stdout
                .write_all(&self.buffer)
                .await
                .context("Failed to write to stdout")?;
            stdout.flush().await.context("Failed to flush stdout")?;
            self.buffer.clear();
        }
        Ok(())
    }
}

#[async_trait]
impl OutputSink for StdoutOutput {
    async fn emit(&mut self, text: &str) -> Result<()> {
        self.buffer.extend_from_slice(text.as_bytes());
        self.buffer.push(b'\n');
        self.flush_buffer().await
    }

    async fn emit_inline(&mut self, text: &str) -> Result<()> {
        self.buffer.extend_from_slice(text.as_bytes());

        if self.buffer.len() >= 1024 {
            self.flush_buffer().await?;
        }

        Ok(())
    }

    async fn emit_error(&mut self, error: &str) -> Result<()> {
        let mut stderr = io::stderr();
        stderr
            .write_all(b"Error: ")
            .await
            .context("Failed to write to stderr")?;
        stderr
            .write_all(error.as_bytes())
            .await
            .context("Failed to write to stderr")?;
        stderr
            .write_all(b"\n")
            .await
            .context("Failed to write to stderr")?;
        stderr.flush().await.context("failed to flush stderr")?;
        Ok(())
    }
}

impl Default for StdoutOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for StdoutOutput {
    fn drop(&mut self) {
        if !self.buffer.is_empty() {
            tracing::warn!(
                "Output buffer not flushed, {} bytes lost",
                self.buffer.len()
            );
        }
    }
}
