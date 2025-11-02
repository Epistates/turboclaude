//! Example Using Permission Callbacks
//!
//! This example demonstrates the permission system which allows you to:
//! - Control which tools Claude can use
//! - Inspect and modify tool parameters before execution
//! - Block dangerous operations
//! - Enforce policies dynamically
//!
//! Common uses:
//! - Restricting file system access
//! - Limiting external API calls
//! - Enforcing parameter validation
//! - Audit logging
//!
//! Run with: cargo run --example with_permissions

use turboclaude_protocol::{PermissionCheckRequest, PermissionResponse};
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

    // Step 2: Create a session with permission configuration
    let session = client.create_session().await?;

    // Step 3: Register a permission handler
    // This function is called every time Claude wants to use a tool
    session.register_permission_handler(|request: PermissionCheckRequest| {
        Box::pin(async move {
            let tool_name = &request.tool;

            // Example: Block file system write operations
            if tool_name.contains("write") || tool_name.contains("delete") {
                println!("🚫 [Permission] Blocking dangerous tool: {}", tool_name);
                return Ok(PermissionResponse {
                    allow: false,
                    modified_input: None,
                    reason: Some(format!(
                        "Tool '{}' is not allowed in this context",
                        tool_name
                    )),
                });
            }

            // Example: Log read operations
            if tool_name.contains("read") {
                println!("✅ [Permission] Allowing read operation: {}", tool_name);
            }

            Ok(PermissionResponse {
                allow: true,
                modified_input: None,
                reason: None,
            })
        })
    });

    // Step 4: Send a query that might invoke tools
    println!("Sending query with permission checks enabled...\n");
    let response = session
        .query_str("What files are in the current directory? But don't delete anything.")
        .await?;

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

    // Step 6: Try another query that might hit permission blocks
    println!("\n\nSending query that might use blocked tools...\n");
    let response2 = session
        .query_str("Create a file called test.txt with content 'hello'")
        .await?;

    println!("\nResponse:");
    for content_block in &response2.message.content {
        match content_block {
            turboclaude_protocol::ContentBlock::Text { text } => {
                println!("{}", text);
            }
            _ => println!("(non-text content)"),
        }
    }

    Ok(())
}
