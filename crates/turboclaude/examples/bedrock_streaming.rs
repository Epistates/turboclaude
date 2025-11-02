//! AWS Bedrock Streaming example
//!
//! This example demonstrates how to use streaming with the BedrockHttpProvider
//! to receive Claude responses in real-time via AWS Bedrock's ConverseStream API.
//!
//! ## Prerequisites
//!
//! Same as bedrock_basic.rs:
//! 1. AWS credentials configured
//! 2. AWS region configured
//! 3. Access to Claude models in AWS Bedrock
//!
//! ## Usage
//!
//! ```bash
//! export AWS_REGION=us-east-1
//! cargo run --example bedrock_streaming --features bedrock
//! ```

use futures::StreamExt;
use std::sync::Arc;
use turboclaude::{
    Client,
    providers::bedrock::BedrockHttpProvider,
    streaming::{ContentBlockDeltaEvent, StreamEvent},
    types::{MessageParam, MessageRequest, Role},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 AWS Bedrock Streaming Example\n");

    // Create Bedrock provider
    println!("🔧 Creating Bedrock provider...");
    let provider = Arc::new(
        BedrockHttpProvider::builder()
            .region(std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string()))
            .build()
            .await?,
    );
    println!("✅ Provider created!\n");

    // Create client
    let client = Client::from_provider(provider);

    // Create message request
    println!("📝 Creating streaming request...");
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![MessageParam {
            role: Role::User,
            content: vec![turboclaude::types::ContentBlockParam::Text {
                text: "Write a haiku about cloud computing.".to_string(),
            }],
        }])
        .build()
        .expect("Failed to build request");

    println!("✅ Request created\n");

    // Create streaming request
    println!("📤 Starting stream...\n");
    println!("💬 Response:\n");

    let mut stream = client
        .messages()
        .stream(request)
        .await
        .expect("Failed to create stream");

    let mut total_tokens_input = 0u32;
    let mut total_tokens_output = 0u32;

    // Process stream events
    while let Some(event) = stream.next().await {
        match event {
            Ok(StreamEvent::MessageStart(start_event)) => {
                println!("🎬 Stream started");
                println!("   Message ID: {}", start_event.message.id);
                println!("   Model: {}\n", start_event.message.model);
            }

            Ok(StreamEvent::ContentBlockStart(start)) => {
                println!("📝 Content block {} started", start.index);
            }

            Ok(StreamEvent::ContentBlockDelta(delta)) => {
                // Print text as it arrives (no newline)
                if let Some(text) = &delta.delta.text {
                    print!("{}", text);
                    use std::io::Write;
                    std::io::stdout().flush()?;
                }
            }

            Ok(StreamEvent::ContentBlockStop(_stop)) => {
                println!("\n📝 Content block finished");
            }

            Ok(StreamEvent::MessageDelta(delta)) => {
                if let Some(stop_reason) = delta.delta.stop_reason {
                    println!("\n🛑 Stop reason: {:?}", stop_reason);
                }
                if let Some(usage) = delta.usage {
                    total_tokens_output += usage.output_tokens;
                }
            }

            Ok(StreamEvent::MessageStop) => {
                println!("✅ Stream completed");
            }

            Err(e) => {
                eprintln!("\n❌ Stream error: {}", e);
                return Err(e.into());
            }

            // Ping events are used for keep-alive
            _ => {}
        }
    }

    // Display usage stats
    println!("\n📊 Final Usage:");
    println!("   Input tokens: {}", total_tokens_input);
    println!("   Output tokens: {}", total_tokens_output);

    println!("\n✨ Streaming example completed!");

    Ok(())
}
