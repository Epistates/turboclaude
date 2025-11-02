//! Simple Query Example
//!
//! This is the most basic example of using the TurboClaude Agent SDK.
//! It demonstrates:
//! - Creating a client
//! - Creating a session
//! - Sending a query
//! - Receiving a response
//!
//! Run with: cargo run --example simple_query

use turboclaudeagent::ClaudeAgentClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Step 1: Create a client
    let config = ClaudeAgentClient::builder()
        .api_key(
            std::env::var("CLAUDE_API_KEY").expect("CLAUDE_API_KEY environment variable not set"),
        )
        .model("claude-3-5-sonnet-20241022")
        .build()?;
    let client = ClaudeAgentClient::new(config);

    // Step 2: Create a session with default configuration
    let session = client.create_session().await?;

    // Step 3: Send a simple query
    println!("Sending query to Claude...");
    let response = session
        .query_str("What is 2 + 2? Respond with just the answer.")
        .await?;

    // Step 4: Print the response
    println!("\nResponse from Claude:");
    for content_block in &response.message.content {
        match content_block {
            turboclaude_protocol::ContentBlock::Text { text } => {
                println!("{}", text);
            }
            _ => println!("(non-text content)"),
        }
    }

    // Step 5: Print usage statistics
    println!(
        "\nUsage: {} input tokens, {} output tokens",
        response.message.usage.input_tokens, response.message.usage.output_tokens
    );

    Ok(())
}
