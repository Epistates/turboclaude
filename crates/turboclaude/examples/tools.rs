//! Example demonstrating tool use with the Turboclaude SDK
//!
//! This example shows how to:
//! 1. Define tools with JSON schemas
//! 2. Send messages with tool definitions
//! 3. Handle tool use responses from the model
//! 4. Continue conversations with tool results
//!
//! # Prerequisites
//!
//! Set your API key:
//! ```bash
//! export ANTHROPIC_API_KEY=sk-ant-...
//! ```
//!
//! # Usage
//!
//! ```bash
//! cargo run --example tools
//! ```

use serde_json::json;
use turboclaude::Client;
use turboclaude::types::{ContentBlockParam, MessageParam, MessageRequest, Role, Tool};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Tool Use Example\n");

    // Create a client
    let client = Client::new(&std::env::var("ANTHROPIC_API_KEY")?);

    // Define tools that Claude can use
    println!("📋 Defining tools...\n");

    // Tool 1: Calculator
    let calculator_tool = Tool {
        name: "calculator".to_string(),
        description: "Performs basic arithmetic operations (add, subtract, multiply, divide)"
            .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"],
                    "description": "The arithmetic operation to perform"
                },
                "a": {
                    "type": "number",
                    "description": "First number"
                },
                "b": {
                    "type": "number",
                    "description": "Second number"
                }
            },
            "required": ["operation", "a", "b"]
        }),
    };

    // Tool 2: Get Weather (mock tool)
    let get_weather_tool = Tool {
        name: "get_weather".to_string(),
        description: "Gets the current weather for a location".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City name with optional state/country, e.g., 'San Francisco, CA' or 'Paris, France'"
                },
                "unit": {
                    "type": "string",
                    "enum": ["celsius", "fahrenheit"],
                    "description": "Temperature unit for the response (default: celsius)"
                }
            },
            "required": ["location"]
        }),
    };

    println!("✅ Defined tools:");
    println!("   1. calculator - Arithmetic operations");
    println!("   2. get_weather - Weather lookup\n");

    // Create a message request with tools
    println!("📝 Creating message request with tools...");
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .tools(vec![calculator_tool, get_weather_tool])
        .messages(vec![MessageParam {
            role: Role::User,
            content: vec![ContentBlockParam::Text {
                text: "What is 42 multiplied by 17? Also, what's the weather like in Paris?"
                    .to_string(),
            }],
        }])
        .build()?;

    println!("✅ Request created\n");

    // Send the request
    println!("📤 Sending request to Claude...\n");
    let response = client.messages().create(request).await?;

    println!("📨 Response from Claude:\n");

    // Process the response content blocks
    let mut tool_calls_made = 0;
    for (i, block) in response.content.iter().enumerate() {
        match block {
            turboclaude::types::ContentBlock::Text { text, .. } => {
                println!("📝 [Text Response]:");
                println!("{}\n", text);
            }
            turboclaude::types::ContentBlock::ToolUse { id, name, input } => {
                tool_calls_made += 1;
                println!("🔨 [Tool Use #{}]:", i + 1);
                println!("    ID: {}", id);
                println!("    Tool: {}", name);
                println!("    Input: {}\n", serde_json::to_string_pretty(input)?);

                // Simulate tool execution
                match name.as_str() {
                    "calculator" => {
                        if let Some(operation) = input.get("operation").and_then(|v| v.as_str()) {
                            if let (Some(a), Some(b)) = (
                                input.get("a").and_then(|v| v.as_f64()),
                                input.get("b").and_then(|v| v.as_f64()),
                            ) {
                                let result = match operation {
                                    "add" => a + b,
                                    "subtract" => a - b,
                                    "multiply" => a * b,
                                    "divide" if b != 0.0 => a / b,
                                    _ => f64::NAN,
                                };
                                println!(
                                    "    💡 Would execute: {} {} {} = {}\n",
                                    a, operation, b, result
                                );
                            }
                        }
                    }
                    "get_weather" => {
                        if let Some(location) = input.get("location").and_then(|v| v.as_str()) {
                            println!("    💡 Would fetch weather for: {}\n", location);
                        }
                    }
                    _ => {
                        println!("    💡 Unknown tool\n");
                    }
                }
            }
            _ => {}
        }
    }

    // Summary
    println!("--- Summary ---\n");
    println!("✅ Response Details:");
    println!("   ID: {}", response.id);
    println!("   Model: {}", response.model);
    println!("   Stop reason: {:?}", response.stop_reason);
    println!("   Tool calls made: {}", tool_calls_made);
    println!("\n📊 Token Usage:");
    println!("   Input tokens: {}", response.usage.input_tokens);
    println!("   Output tokens: {}", response.usage.output_tokens);
    println!();
    println!("💡 Next Steps:");
    println!("   In a real application, you would:");
    println!("   1. Execute the tool(s) with the provided input");
    println!("   2. Get the result(s)");
    println!("   3. Send a follow-up message with the tool result(s) for continued conversation");
    println!();

    Ok(())
}
