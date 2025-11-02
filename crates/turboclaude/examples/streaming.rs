//! Example demonstrating streaming responses with the Turboclaude SDK
//!
//! This example shows how to:
//! 1. Create a streaming message request
//! 2. Handle stream events in real-time
//! 3. Display partial responses as they arrive
//! 4. Accumulate the complete message
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
//! cargo run --example streaming
//! ```

use futures::StreamExt;
use turboclaude::Client;
use turboclaude::streaming::StreamEvent;
use turboclaude::types::{ContentBlockParam, MessageParam, MessageRequest, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌊 Streaming Example\n");

    // Create a client
    let client = Client::new(&std::env::var("ANTHROPIC_API_KEY")?);

    // Create a message request with streaming enabled
    println!("📝 Creating streaming message request...");
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![MessageParam {
            role: Role::User,
            content: vec![ContentBlockParam::Text {
                text: "Tell me a short story about a robot learning to paint. Make it creative and fun!"
                    .to_string(),
            }],
        }])
        .stream(true)
        .build()?;

    println!("✅ Request created\n");

    // Send the streaming request
    println!("📤 Sending streaming request to Claude...\n");
    println!("📨 Response (streaming):\n");
    println!("{}", "─".repeat(50));

    let mut stream = client.messages().stream(request).await?;

    let mut event_count = 0;
    let mut text_chunks = Vec::new();

    // Process each event as it arrives
    while let Some(event_result) = stream.next().await {
        match event_result {
            Ok(stream_event) => {
                match stream_event {
                    StreamEvent::MessageStart(start) => {
                        event_count += 1;
                        println!("\n🏁 Message started");
                        println!("   Model: {}", start.message.model);
                        println!(
                            "   Initial tokens: {}",
                            start
                                .message
                                .usage
                                .as_ref()
                                .map(|u| u.input_tokens)
                                .unwrap_or(0)
                        );
                    }
                    StreamEvent::ContentBlockStart(start) => {
                        event_count += 1;
                        println!("\n📝 Content block #{} started", start.index + 1);
                    }
                    StreamEvent::ContentBlockDelta(delta) => {
                        event_count += 1;
                        if let Some(text) = &delta.delta.text {
                            // Print text as it arrives
                            print!("{}", text);
                            // Flush to ensure it displays immediately
                            use std::io::Write;
                            std::io::stdout().flush()?;
                            // Collect for final display
                            text_chunks.push(text.clone());
                        }
                    }
                    StreamEvent::ContentBlockStop(_) => {
                        event_count += 1;
                        println!("\n\n✅ Content block completed");
                    }
                    StreamEvent::MessageDelta(delta) => {
                        event_count += 1;
                        if let Some(stop_reason) = &delta.delta.stop_reason {
                            println!("\n⏸️  Stop reason: {:?}", stop_reason);
                        }
                        if let Some(usage) = &delta.usage {
                            println!("   Output tokens: {}", usage.output_tokens);
                        }
                    }
                    StreamEvent::MessageStop => {
                        event_count += 1;
                        println!("\n🏁 Message stream completed");
                    }
                    StreamEvent::Ping => {
                        // Pings keep the connection alive, don't print them
                    }
                    StreamEvent::Unknown => {
                        println!("\n⚠️  Unknown event");
                    }
                }
            }
            Err(e) => {
                println!("\n❌ Stream error: {}", e);
                return Err(e.into());
            }
        }
    }

    println!("\n{}", "─".repeat(50));

    // Display summary
    println!("\n--- Summary ---\n");
    let full_text = text_chunks.join("");
    println!("✅ Streaming complete!");
    println!("   Total events received: {}", event_count);
    println!("   Total characters: {}", full_text.len());
    println!("   Words: ~{}", full_text.split_whitespace().count());
    println!();
    println!("💡 Streaming benefits:");
    println!("   ✓ Real-time response display");
    println!("   ✓ Better UX for long-form content");
    println!("   ✓ Faster perceived response time");
    println!("   ✓ Lower memory footprint for large responses");
    println!();

    Ok(())
}
