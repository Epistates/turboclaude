//! Example: Error Handling and Resilience
//!
//! This example demonstrates:
//! - Handling connection errors
//! - Automatic retry with backoff
//! - Graceful degradation
//! - Error recovery patterns
//! - Builder pattern for custom query configuration
//!
//! The Agent SDK automatically handles:
//! - Subprocess crashes (auto-restart)
//! - Network timeouts (retry with exponential backoff)
//! - Protocol errors (validation and recovery)
//!
//! This example shows how to handle errors in your application code.
//!
//! Run with: cargo run --example error_handling

use std::time::Duration;
use turboclaudeagent::ClaudeAgentClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Step 1: Create a client with custom timeout
    let config = ClaudeAgentClient::builder()
        .api_key(
            std::env::var("CLAUDE_API_KEY").expect("CLAUDE_API_KEY environment variable not set"),
        )
        .model("claude-3-5-sonnet-20241022")
        .build()?;
    let client = ClaudeAgentClient::new(config);

    // Step 2: Create a session
    let session = client.create_session().await?;

    // Step 3: Send a query with error handling
    println!("Attempting to send query with error handling...\n");

    match session.query_str("What is 2 + 2?").await {
        Ok(response) => {
            println!("✅ Query succeeded!");
            for content_block in &response.message.content {
                match content_block {
                    turboclaude_protocol::ContentBlock::Text { text } => {
                        println!("Response: {}", text);
                    }
                    _ => println!("(non-text content)"),
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Query failed: {}", e);
            eprintln!("\nHandling error gracefully...");

            // You could implement retry logic here
            println!("Would retry with exponential backoff in production");

            // Or fallback to cached response
            println!("Using cached response or default value");
        }
    }

    // Step 4: Example of handling specific error types
    println!("\n\nDemonstrating error classification:");
    match send_query_with_retry(&session, "Test query", 3).await {
        Ok(_response) => println!("✅ Query succeeded after retry"),
        Err(e) => eprintln!("❌ All retries failed: {}", e),
    }

    // Step 5: Demonstrate builder pattern for custom configuration
    println!("\n\nDemonstrating query builder with chained configuration:");
    match session
        .query_str("Explain error handling")
        .max_tokens(8000) // Override default max_tokens
        .system_prompt("Be concise and focus on best practices")
        .await
    {
        Ok(response) => {
            println!("✅ Query with custom config succeeded!");
            for content_block in &response.message.content {
                match content_block {
                    turboclaude_protocol::ContentBlock::Text { text } => {
                        println!("Response: {}", text);
                    }
                    _ => println!("(non-text content)"),
                }
            }
        }
        Err(e) => eprintln!("❌ Query failed: {}", e),
    }

    Ok(())
}

/// Helper function: Send a query with retry logic
///
/// This demonstrates a common pattern for resilient applications:
/// - Attempt the query
/// - If it fails, retry with a delay
/// - Give up after max_retries
async fn send_query_with_retry(
    session: &turboclaudeagent::AgentSession,
    query: &str,
    max_retries: usize,
) -> anyhow::Result<turboclaude_protocol::QueryResponse> {
    let mut retries = 0;
    let mut delay = Duration::from_millis(100);

    loop {
        match session.query_str(query).await {
            Ok(response) => {
                if retries > 0 {
                    println!("✅ Query succeeded after {} retries", retries);
                }
                return Ok(response);
            }
            Err(e) => {
                retries += 1;
                if retries >= max_retries {
                    return Err(e.into());
                }

                eprintln!(
                    "⚠️  Query attempt {} failed: {}. Retrying in {:?}...",
                    retries, e, delay
                );

                tokio::time::sleep(delay).await;

                // Exponential backoff: double the delay, cap at 5 seconds
                delay = std::cmp::min(delay * 2, Duration::from_secs(5));
            }
        }
    }
}
