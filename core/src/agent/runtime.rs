use crate::io::{input::InputSource, output::OutputSink};
use crate::agent::agent::Agent;
use futures_util::StreamExt;

pub async fn run(
    mut input: impl InputSource,
    mut output: impl OutputSink,
    mut agent: Agent,
) -> anyhow::Result<()> {
    output.emit("Agent ready. Type something. Ctrl+D to exit.").await?;

    while let Some(text) = input.next().await? {
        let mut stream = agent.handle_input(text).await?;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            output.emit_inline(&chunk).await?;
        }

        output.emit("\n").await?;
    }

    Ok(())
}
