use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model_name: String,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    #[serde(default = "default_max_retries")]
    #[allow(dead_code)]
    pub max_retries: u32,
}

fn default_timeout() -> u64 {
    30
}

fn default_max_retries() -> u32 {
    3
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub ollama: OllamaConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        Self::load_from_path("config/config.toml")
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        tracing::info!("Loading configuration from: {}", path.display());

        let cfg = config::Config::builder()
            .add_source(config::File::from(path))
            .add_source(config::Environment::with_prefix("APP").separator("__"))
            .build()
            .with_context(|| format!("Failed to build config from {}", path.display()))?;

        let cfg: AppConfig = cfg
            .try_deserialize()
            .context("Failed to deserialize config")?;

        cfg.validate().context("Config validation failed")?;

        tracing::info!("Configuration loaded successfully");
        Ok(cfg)
    }

    fn validate(&self) -> Result<()> {
        if self.ollama.base_url.is_empty() {
            anyhow::bail!("ollama.base_url cannot be empty");
        }

        if !self.ollama.base_url.starts_with("http://")
            && !self.ollama.base_url.starts_with("https://")
        {
            anyhow::bail!("ollama.base_url must start with http:// or https://");
        }

        if self.ollama.model_name.is_empty() {
            anyhow::bail!("ollama.model_name cannot be empty");
        }

        if self.ollama.timeout_secs == 0 {
            anyhow::bail!("ollama.timeout_secs must be greater than 0");
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn load_or_default() -> Self {
        match Self::load() {
            Ok(cfg) => cfg,
            Err(e) => {
                tracing::warn!("Failed to load config: {}, using defaults", e);
                Self::default()
            }
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ollama: OllamaConfig {
                base_url: "http://localhost:11434".to_string(),
                model_name: "llama3.1:8b".to_string(),
                timeout_secs: 30,
                max_retries: 3,
            },
        }
    }
}
