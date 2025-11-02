//! Example: Query Builder Pattern
//!
//! This example demonstrates the elegant builder pattern for constructing queries.
//! The `query_str()` method returns a `QueryBuilder` that can be awaited directly
//! or chained with configuration methods.
//!
//! This provides maximum flexibility:
//! - Simple queries: just await directly
//! - Complex queries: chain configuration methods
//! - All fields customizable: max_tokens, system_prompt, model, tools, messages
//!
//! Run with: cargo run --example builder_pattern

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

    println!("=== Query Builder Pattern Examples ===\n");

    // Example 1: Simple query (await directly, uses defaults)
    println!("1. Simple query (defaults):");
    let response = session.query_str("What is 2+2?").await?;
    print_response(&response);

    // Example 2: Override max_tokens
    println!("\n2. Query with custom max_tokens:");
    let response = session
        .query_str("Explain quantum computing")
        .max_tokens(8000) // Allow longer response
        .await?;
    print_response(&response);

    // Example 3: Add system prompt
    println!("\n3. Query with system prompt:");
    let response = session
        .query_str("What are the three laws of thermodynamics?")
        .system_prompt("You are a physics professor. Be precise and educational.")
        .await?;
    print_response(&response);

    // Example 4: Multiple chained configurations
    println!("\n4. Query with multiple configurations:");
    let response = session
        .query_str("Compare Rust and Python for web development")
        .max_tokens(6000)
        .system_prompt("You are a senior software architect. Be balanced and technical.")
        .await?;
    print_response(&response);

    // Example 5: Override model for specific query
    println!("\n5. Query with different model:");
    let response = session
        .query_str("Write a haiku about coding")
        .model("claude-3-haiku-20240307") // Use faster model for simple task
        .max_tokens(100)
        .await?;
    print_response(&response);

    // Example 6: Show that builder can be stored and executed later
    println!("\n6. Store builder and execute later:");
    let builder = session
        .query_str("What is the Fibonacci sequence?")
        .max_tokens(2000)
        .system_prompt("Explain like I'm a beginner programmer");

    // Do other work here...
    println!("   (doing other work...)");

    // Execute when ready
    let response = builder.await?;
    print_response(&response);

    println!("\n=== All examples completed! ===");
    Ok(())
}

/// Helper to print response content
fn print_response(response: &turboclaude_protocol::QueryResponse) {
    for content_block in &response.message.content {
        match content_block {
            turboclaude_protocol::ContentBlock::Text { text } => {
                // Truncate long responses for demo
                let preview = if text.len() > 200 {
                    format!("{}... (truncated)", &text[..200])
                } else {
                    text.clone()
                };
                println!("   Response: {}", preview);
            }
            _ => println!("   (non-text content)"),
        }
    }
}
