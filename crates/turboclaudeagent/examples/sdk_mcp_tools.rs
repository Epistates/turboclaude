//! Example demonstrating SDK MCP server usage for in-process tools.
//!
//! This example shows how to create in-process MCP servers that run within
//! the same process as your application, eliminating subprocess overhead.
//!
//! # Features Demonstrated
//!
//! - Creating an SDK MCP server with multiple tools
//! - Type-safe input/output with serde
//! - Error handling in tool execution
//! - Integration with AgentSession
//!
//! # Running
//!
//! ```bash
//! cargo run --example sdk_mcp_tools
//! ```

use serde::{Deserialize, Serialize};
use turboclaudeagent::SessionConfig;
use turboclaudeagent::mcp::sdk::*;

/// Input for calculator operations
#[derive(Deserialize, Debug)]
struct CalcInput {
    a: i32,
    b: i32,
}

/// Output for calculator operations
#[derive(Serialize, Debug)]
struct CalcOutput {
    result: i32,
}

/// Input for string operations
#[derive(Deserialize, Debug)]
struct StringInput {
    text: String,
}

/// Output for string operations
#[derive(Serialize, Debug)]
struct StringOutput {
    processed: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("SDK MCP Tools Example");
    println!("=====================\n");

    // Create a calculator SDK MCP server
    let calculator = SdkMcpServerBuilder::new("calculator")
        .tool("add", "Add two numbers", |input: CalcInput| async move {
            println!("  [Tool] add({}, {})", input.a, input.b);
            Ok(CalcOutput {
                result: input.a + input.b,
            })
        })
        .tool(
            "multiply",
            "Multiply two numbers",
            |input: CalcInput| async move {
                println!("  [Tool] multiply({}, {})", input.a, input.b);
                Ok(CalcOutput {
                    result: input.a * input.b,
                })
            },
        )
        .tool(
            "divide",
            "Divide two numbers",
            |input: CalcInput| async move {
                println!("  [Tool] divide({}, {})", input.a, input.b);
                if input.b == 0 {
                    return Err::<CalcOutput, _>(SdkToolError::ExecutionFailed(
                        "Division by zero".to_string(),
                    ));
                }
                Ok(CalcOutput {
                    result: input.a / input.b,
                })
            },
        )
        .build();

    // Create a text processing SDK MCP server
    let text_processor = SdkMcpServerBuilder::new("text")
        .tool(
            "uppercase",
            "Convert text to uppercase",
            |input: StringInput| async move {
                println!("  [Tool] uppercase(\"{}\")", input.text);
                Ok(StringOutput {
                    processed: input.text.to_uppercase(),
                })
            },
        )
        .tool(
            "lowercase",
            "Convert text to lowercase",
            |input: StringInput| async move {
                println!("  [Tool] lowercase(\"{}\")", input.text);
                Ok(StringOutput {
                    processed: input.text.to_lowercase(),
                })
            },
        )
        .tool("reverse", "Reverse text", |input: StringInput| async move {
            println!("  [Tool] reverse(\"{}\")", input.text);
            Ok(StringOutput {
                processed: input.text.chars().rev().collect(),
            })
        })
        .build();

    println!("Created SDK MCP servers:");
    println!("  - calculator: {} tools", calculator.tool_count());
    println!("  - text: {} tools\n", text_processor.tool_count());

    // Demonstrate direct tool execution
    println!("Direct Tool Execution:");
    println!("======================\n");

    // Test calculator
    println!("1. Calculator tests:");
    let result = calculator
        .execute_tool("add", serde_json::json!({"a": 5, "b": 3}))
        .await?;
    println!("   Result: {}\n", result);

    let result = calculator
        .execute_tool("multiply", serde_json::json!({"a": 7, "b": 6}))
        .await?;
    println!("   Result: {}\n", result);

    // Test error handling
    println!("2. Error handling:");
    match calculator
        .execute_tool("divide", serde_json::json!({"a": 10, "b": 0}))
        .await
    {
        Ok(_) => println!("   Unexpected success"),
        Err(e) => println!("   Caught error: {}\n", e),
    }

    // Test text processor
    println!("3. Text processing tests:");
    let result = text_processor
        .execute_tool("uppercase", serde_json::json!({"text": "hello world"}))
        .await?;
    println!("   Result: {}\n", result);

    let result = text_processor
        .execute_tool("reverse", serde_json::json!({"text": "rust"}))
        .await?;
    println!("   Result: {}\n", result);

    // Demonstrate server introspection
    println!("Server Introspection:");
    println!("=====================\n");

    println!("Calculator tools:");
    for tool in calculator.list_tools() {
        println!("  - {}: {}", tool.name(), tool.description());
    }
    println!();

    println!("Text processing tools:");
    for tool in text_processor.list_tools() {
        println!("  - {}: {}", tool.name(), tool.description());
    }
    println!();

    // Show configuration integration
    println!("Session Configuration:");
    println!("======================\n");

    let config = SessionConfig::new()
        .add_sdk_server(calculator)
        .add_sdk_server(text_processor);

    println!(
        "Created session config with {} SDK servers",
        config.sdk_servers.len()
    );
    println!("Total tools available: {}", {
        let mut count = 0;
        for server in &config.sdk_servers {
            count += server.tool_count();
        }
        count
    });

    println!("\n✓ Example completed successfully!");
    println!("\nNote: To use these tools in a real session, you would:");
    println!("  1. Create an AgentSession with the configured SDK servers");
    println!("  2. Query Claude, which can then call these tools");
    println!("  3. Tools execute in-process with zero subprocess overhead");

    Ok(())
}
