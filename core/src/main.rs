mod agent;
mod io;
mod llm;
mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = config::AppConfig::load()?;
    let input = io::input::StdinInput::new();
    let output = io::output::StdoutOutput::new();
    let agent = agent::agent::Agent::new(&cfg.ollama);

    agent::runtime::run(input, output, agent).await
}
