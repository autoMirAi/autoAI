use crate::error::Result;
use async_trait::async_trait;
use tokio::io::{self, AsyncWriteExt};

#[async_trait]
pub trait OutputSink: Send {
    async fn emit(&mut self, text: &str) -> Result<()>;
    async fn emit_chunk(&mut self, chunk: &str) -> Result<()>;
    async fn emit_error(&mut self, error: &str) -> Result<()>;
    async fn flush(&mut self) -> Result<()>;
}

pub struct TextOutput {
    buffer: Vec<u8>,
    buffer_capacity: usize,
}

impl TextOutput {
    const DEFAULT_BUFFER_SIZE: usize = 4096;
    const FLUSH_THRESHOLD: usize = 1024;

    pub fn new() -> Self {
        tracing::debug!("Initializing stdout output");
        Self {
            buffer: Vec::with_capacity(Self::DEFAULT_BUFFER_SIZE),
            buffer_capacity: Self::DEFAULT_BUFFER_SIZE,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            buffer_capacity: capacity,
        }
    }

    async fn flush_buffer(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        let mut stdout = io::stdout();
        stdout.write_all(&self.buffer).await?;
        stdout.flush().await?;
        self.buffer.clear();

        Ok(())
    }
}

#[async_trait]
impl OutputSink for TextOutput {
    async fn emit(&mut self, text: &str) -> Result<()> {
        self.buffer.extend_from_slice(text.as_bytes());
        self.buffer.push(b'\n');
        self.flush_buffer().await
    }

    async fn emit_chunk(&mut self, chunk: &str) -> Result<()> {
        self.buffer.extend_from_slice(chunk.as_bytes());

        if self.buffer.len() >= Self::FLUSH_THRESHOLD {
            self.flush_buffer().await?;
        }

        Ok(())
    }

    async fn emit_error(&mut self, error: &str) -> Result<()> {
        self.flush_buffer().await?;

        let mut stderr = io::stderr();
        stderr.write_all(b"\x1b[31mError: ").await?;
        stderr.write_all(error.as_bytes()).await?;
        stderr.write_all(b"\x1b[0m\n").await?;

        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        self.flush_buffer().await
    }
}

impl Default for TextOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TextOutput {
    fn drop(&mut self) {
        if !self.buffer.is_empty() {
            tracing::warn!(
                "Output buffer not flushed, {} bytes lost",
                self.buffer.len()
            );
        }
    }
}
