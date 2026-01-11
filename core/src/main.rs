mod agent;
mod io;
mod llm;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let input = io::input::StdinInput::new();
    let output = io::output::StdoutOutput::new();
    let agent = agent::agent::Agent::new();

    agent::runtime::run(input, output, agent).await
}
