use crate::agent::agent::Agent;
use crate::error::Result;
use crate::io::{input::InputSource, output::OutputSink};
use futures_util::StreamExt;
use tokio::signal;

pub async fn run(
    mut input: impl InputSource,
    mut output: impl OutputSink,
    mut agent: Agent,
) -> Result<()> {
    tracing::info!("Performing health check...");
    if let Err(e) = agent.health_check().await {
        output
            .emit_error(&format!("Health check failed: {}", e))
            .await?;
        output.emit("Continuing anyway...").await?;
    }

    output
        .emit("🤖 Agent ready. Type your message and press Enter. Ctrl+D or Ctrl+C to exit.")
        .await?;
    output.emit("").await?;

    let ctrl_c = async {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        tracing::info!("Received Ctrl+C signal");
    };

    tokio::pin!(ctrl_c);

    loop {
        tokio::select! {
            _ = &mut ctrl_c => {
                output.emit("\n👋 Goodbye!").await?;
                break;
            }

            result = input.next() => {
                match result {
                    Ok(Some(text)) => {
                        if let Err(e) = process_input(&mut output, &mut agent, text).await {
                            tracing::error!("Error processing input: {}", e);
                            output.emit_error(&format!("{}", e)).await?;
                            output.emit("").await?;
                        }
                    }
                    Ok(None) => {
                        tracing::info!("Reached EOF");
                        output.emit("\n👋 Goodbye!").await?;
                        break;
                    }
                    Err(e) => {
                        tracing::error!("Input error: {}", e);
                        output.emit_error(&format!("Input error: {}", e)).await?;
                        break;
                    }
                }
            }
        }
    }

    tracing::info!("Runtime shutting down");
    Ok(())
}

async fn process_input(
    output: &mut impl OutputSink,
    agent: &mut Agent,
    text: String,
) -> Result<()> {
    output.emit(&format!("You: {}", text)).await?;
    output.emit("").await?;
    output.emit("Assistant: ").await?;

    let mut stream = agent.handle_input(text).await?;

    let mut total_chars = 0;

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;

        if !chunk.text.is_empty() {
            output.emit_inline(&chunk.text).await?;
            total_chars += chunk.text.len();
        }

        if chunk.done {
            tracing::debug!("Stream completed, total chars: {}", total_chars);
            break;
        }
    }

    output.emit("\n").await?;
    output.emit("").await?;

    Ok(())
}
