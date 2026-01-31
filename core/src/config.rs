use crate::error::{AppError, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model_name: String,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

fn default_timeout() -> u64 {
    30
}
fn default_max_retries() -> u32 {
    3
}

#[derive(Debug, Deserialize, Clone)]
pub struct VoiceConfig {
    pub model_path: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_silience_threshold")]
    pub silience_threshold_secs: f32,
    #[serde(default = "default_max_duration")]
    pub max_duration_secs: f32,
    #[serde(default)]
    pub translate: bool,
}

fn default_language() -> String {
    "auto".to_string()
}

fn default_silience_threshold() -> f32 {
    1.5
}

fn default_max_duration() -> f32 {
    30.0
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            model_path: "models/ggml-base.bin".to_string(),
            language: default_language(),
            silience_threshold_secs: default_silience_threshold(),
            max_duration_secs: default_max_duration(),
            translate: false,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub ollama: OllamaConfig,
    pub voice: Option<VoiceConfig>,
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
            .map_err(|e| AppError::Config(format!("Failed to build config: {}", e)))?;

        let cfg: AppConfig = cfg
            .try_deserialize()
            .map_err(|e| AppError::Config(format!("Failed to deserialize: {}", e)))?;

        cfg.validate()?;

        tracing::info!("Configuration loaded successfully");
        Ok(cfg)
    }

    fn validate(&self) -> Result<()> {
        if self.ollama.base_url.is_empty() {
            return Err(AppError::Config("base_url cannot be empty".to_string()));
        }

        if !self.ollama.base_url.starts_with("http://")
            && !self.ollama.base_url.starts_with("https://")
        {
            return Err(AppError::Config(
                "base_url must start with http:// or https://".to_string(),
            ));
        }

        if self.ollama.model_name.is_empty() {
            return Err(AppError::Config("model_name cannot be empty".to_string()));
        }

        if self.ollama.timeout_secs == 0 {
            return Err(AppError::Config(
                "timeout_secs must be greater than 0".to_string(),
            ));
        }

        if let Some(ref voice) = self.voice {
            if voice.model_path.is_empty() {
                return Err(AppError::Config(
                    "voice.model_path cannot be empty".to_string(),
                ));
            }
            if voice.silience_threshold_secs <= 0.0 {
                return Err(AppError::Config(
                    "voice.silence_threshold_secs must be positive".to_string(),
                ));
            }
            if voice.max_duration_secs <= 0.0 {
                return Err(AppError::Config(
                    "voice.max_duration_secs must be positive".to_string(),
                ));
            }
        }

        Ok(())
    }

    pub fn voice_enabled(&self) -> bool {
        self.voice.is_some()
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
            voice: None,
        }
    }
}
