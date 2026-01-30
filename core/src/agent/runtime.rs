use crate::agent::agent::Agent;
use crate::error::Result;
use crate::io::{InputSource, OutputSink};
use futures_util::StreamExt;
use tokio::signal;

pub async fn run(
    mut input: impl InputSource,
    mut output: impl OutputSink,
    agent: Agent,
) -> Result<()> {
    perform_health_check(&agent, &mut output).await?;

    output
        .emit("🤖 Agent ready. Type your message and press Enter. Ctrl+D or Ctrl+C to exit.")
        .await?;
    output.emit("").await?;

    run_main_loop(&mut input, &mut output, &agent).await
}

async fn perform_health_check(agent: &Agent, output: &mut impl OutputSink) -> Result<()> {
    tracing::info!("Performing health check...");

    if let Err(e) = agent.health_check().await {
        output
            .emit_error(&format!("Health check failed: {}", e))
            .await?;
        output.emit("Continuing anyway...").await?;
    }

    Ok(())
}

async fn run_main_loop(
    input: &mut impl InputSource,
    output: &mut impl OutputSink,
    agent: &Agent,
) -> Result<()> {
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
                        if let Err(e) = process_user_input(output, agent, &text).await {
                            tracing::error!("Error processing input: {}", e);
                            output.emit_error(&e.to_string()).await?;
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

    output.flush().await?;
    tracing::info!("Runtime shutting down");
    Ok(())
}

async fn process_user_input(output: &mut impl OutputSink, agent: &Agent, text: &str) -> Result<()> {
    output.emit(&format!("You: {}", text)).await?;
    output.emit("").await?;
    output.emit("Assistant: ").await?;

    let mut stream = agent.process(text).await?;
    let mut total_chars = 0;

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;

        if !chunk.text.is_empty() {
            output.emit_chunk(&chunk.text).await?;
            total_chars += chunk.text.len();
        }

        if chunk.done {
            tracing::debug!("Stream completed, total chars: {}", total_chars);
            break;
        }
    }

    output.flush().await?;
    output.emit("\n").await?;
    output.emit("").await?;

    Ok(())
}
