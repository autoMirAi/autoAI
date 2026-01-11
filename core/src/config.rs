use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model_name: String,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub ollama: OllamaConfig,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let cfg = config::Config::builder()
            .add_source(config::File::with_name("config/config.toml"))
            .add_source(config::Environment::with_prefix("APP"))
            .build()?;
        let cfg: AppConfig = cfg.try_deserialize()?;
        Ok(cfg)
    }
}
