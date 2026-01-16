mod agent;
mod config;
mod error;
mod io;
mod llm;

use error::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    init_logging()?;

    tracing::info!("Starting AI Chat application");

    let cfg = config::AppConfig::load()?;

    tracing::debug!("Configuration: {:#?}", cfg);

    let input = io::input::StdinInput::new();
    let output = io::output::StdoutOutput::new();

    let agent = agent::agent::Agent::new(&cfg.ollama)?;

    let result = agent::runtime::run(input, output, agent).await;

    match result {
        Ok(_) => {
            tracing::info!("Application exited normally");
            Ok(())
        }
        Err(e) => {
            tracing::error!("Application error: {}", e);
            eprintln!("\n❌ Fatal error: {}", e);
            std::process::exit(1);
        }
    }
}

fn init_logging() -> Result<()> {
    let env_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info,core=debug".to_string());

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| env_filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}
