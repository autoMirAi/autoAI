use async_trait::async_trait;
use tokio::io::{self, AsyncBufReadExt};

#[async_trait]
pub trait InputSource {
    async fn next(&mut self) -> anyhow::Result<Option<String>>;
}

pub struct StdinInput {
    reader: io::BufReader<io::Stdin>,
}

impl StdinInput {
    pub fn new() -> Self {
        Self {
            reader: io::BufReader::new(io::stdin()),
        }
    }
}

#[async_trait]
impl InputSource for StdinInput {
    async fn next(&mut self) -> anyhow::Result<Option<String>> {
        let mut line = String::new();
        let n = self.reader.read_line(&mut line).await?;
        if n == 0 {
            Ok(None)
        } else {
            Ok(Some(line.trim().to_string()))
        }
    }
}
