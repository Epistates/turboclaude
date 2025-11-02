//! Example Using Hooks for Lifecycle Events
//!
//! This example demonstrates the hook system which allows you to react to
//! key lifecycle events:
//! - PreQuery: Before a query is sent
//! - PostQuery: After a response is received
//! - PreToolUse: Before a tool would be called
//! - PostToolUse: After a tool has been used
//!
//! Hooks are useful for:
//! - Logging and monitoring
//! - Metrics collection
//! - State management
//! - Validation
//!
//! Run with: cargo run --example with_hooks

use turboclaude_protocol::{HookRequest, HookResponse};
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

    // Step 3: Register hooks for lifecycle events
    // PreQuery hook
    session.register_hook("PreQuery".to_string(), |event: HookRequest| {
        Box::pin(async move {
            // Extract query from event data (JSON value)
            let query = event
                .data
                .get("query")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            println!("🔵 [Hook] About to send query: {}", query);

            Ok(HookResponse::continue_exec())
        })
    });

    // PostQuery hook
    session.register_hook("PostQuery".to_string(), |_event: HookRequest| {
        Box::pin(async move {
            println!("🟢 [Hook] Received response");

            Ok(HookResponse::continue_exec())
        })
    });

    // PreToolUse hook
    session.register_hook("PreToolUse".to_string(), |event: HookRequest| {
        Box::pin(async move {
            // Extract tool name from event data
            let tool_name = event
                .data
                .get("tool_name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            println!("🟡 [Hook] Tool would be used: {}", tool_name);

            Ok(HookResponse::continue_exec())
        })
    });

    // Step 4: Send a query and observe hooks firing
    println!("Sending query with hooks enabled...\n");
    let response = session.query_str("What is the capital of France?").await?;

    // Step 5: Print the response
    println!("\nResponse:");
    for content_block in &response.message.content {
        match content_block {
            turboclaude_protocol::ContentBlock::Text { text } => {
                println!("{}", text);
            }
            _ => println!("(non-text content)"),
        }
    }

    Ok(())
}
