//! Example: Session Recovery and Reconnection
//!
//! This example demonstrates:
//! - Handling session disconnection
//! - Automatic reconnection behavior
//! - State preservation across reconnects
//! - Graceful shutdown
//!
//! The Agent SDK automatically:
//! - Detects when transport is disconnected
//! - Attempts to reconnect with exponential backoff
//! - Reestablishes the session
//!
//! This example shows how to work with these behaviors in your application.
//!
//! Run with: cargo run --example reconnection

use std::time::Duration;
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

    // Step 2: Create a session
    let session = client.create_session().await?;

    // Step 3: Send initial query
    println!("Sending initial query...");
    match session.query_str("What is the capital of France?").await {
        Ok(response) => {
            println!("✅ Got response");
            for content_block in &response.message.content {
                match content_block {
                    turboclaude_protocol::ContentBlock::Text { text } => {
                        println!("Response: {}", text);
                    }
                    _ => println!("(non-text content)"),
                }
            }
        }
        Err(e) => eprintln!("❌ Error: {}", e),
    }

    // Step 4: Check session state
    println!("\n📊 Session state: {:?}", session.state().await);

    // Step 5: Simulate waiting (in real applications, this could be a long-running session)
    println!("\nWaiting 2 seconds...");
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Step 6: Send another query (if disconnected, SDK will attempt reconnect)
    println!("\nSending follow-up query...");
    match session.query_str("What is the capital of Germany?").await {
        Ok(response) => {
            println!("✅ Got response");
            for content_block in &response.message.content {
                match content_block {
                    turboclaude_protocol::ContentBlock::Text { text } => {
                        println!("Response: {}", text);
                    }
                    _ => println!("(non-text content)"),
                }
            }
        }
        Err(e) => eprintln!("❌ Error: {}", e),
    }

    // Step 7: Check final session state
    println!("\n📊 Final session state: {:?}", session.state().await);

    // Step 8: Graceful shutdown
    println!("\nClosing session...");
    session.close().await?;
    println!("✅ Session closed");

    Ok(())
}

/// Example: Monitor session health
///
/// This shows how to implement health monitoring in your application
#[allow(dead_code)]
async fn monitor_session_health(session: &turboclaudeagent::AgentSession) {
    loop {
        // Check connection status using convenience method
        if session.is_connected().await {
            println!("✅ Session is healthy");
        } else {
            println!("⚠️  Session disconnected, waiting for reconnect...");
        }

        // You can also get full state for more details
        let state = session.state().await;
        println!(
            "   Model: {}, Active queries: {}",
            state.current_model, state.active_queries
        );

        tokio::time::sleep(Duration::from_secs(1)).await;

        // In a real application, you might want to break on certain conditions
        // For this example, we'll run indefinitely (but the function is marked dead_code)
    }
}
