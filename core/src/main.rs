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

    let output = io::TextOutput::new();
    let agent = agent::Agent::new(&cfg.ollama)?;

    if let Some(ref voice_ref) = cfg.voice {
        tracing::info!("voic mode start!");
        let input = io::VoiceInput::new(voice_ref)?;
        run_with_input(input, output, agent).await
    } else {
        tracing::info!("text mode start!");
        let input = io::TextInput::new();
        run_with_input(input, output, agent).await
    }
}

async fn run_with_input(
    input: impl io::InputSource,
    output: impl io::OutputSink,
    agent: agent::Agent,
) -> Result<()> {
    match agent::run(input, output, agent).await {
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
    let default_filter = "info,core=debug";
    let env_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| default_filter.to_string());

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| env_filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}
