use async_trait::async_trait;
use tokio::io::{self, AsyncWriteExt};

#[async_trait]
pub trait OutputSink {
    async fn emit(&mut self, text: &str) -> anyhow::Result<()>;
    async fn emit_inline(&mut self, text: &str) -> anyhow::Result<()>;
}

pub struct StdoutOutput;

impl StdoutOutput {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl OutputSink for StdoutOutput {
    async fn emit(&mut self, text: &str) -> anyhow::Result<()> {
        let mut stdout = io::stdout();
        stdout.write_all(text.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
        Ok(())
    }

    async fn emit_inline(&mut self, text: &str) -> anyhow::Result<()> {
        let mut stdout = io::stdout();
        stdout.write_all(text.as_bytes()).await?;
        stdout.flush().await?;
        Ok(())
    }
}
