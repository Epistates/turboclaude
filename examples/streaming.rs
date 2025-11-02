//! Example demonstrating streaming responses with the Anthropic SDK

use anthropic::{Client, types::{MessageRequest, Message, Models}, streaming::StreamEvent};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let client = Client::builder()
        .api_key("sk-ant-api-key")  // Replace with your actual API key
        .build()?;

    // Create a message request
    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![
            Message::user("Tell me a short story about a robot learning to paint.")
        ])
        .stream(true)  // Enable streaming
        .build()?;

    println!("Streaming response from Claude...\n");

    // Get the stream
    let mut stream = client.messages().stream(request).await?;

    // Process events as they arrive
    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::ContentBlockDelta(delta) => {
                if let Some(text) = delta.delta.text {
                    print!("{}", text);
                    std::io::Write::flush(&mut std::io::stdout())?;
                }
            }
            StreamEvent::MessageStart(start) => {
                println!("Message started (model: {})", start.message.model);
            }
            StreamEvent::MessageDelta(delta) => {
                if let Some(reason) = delta.delta.stop_reason {
                    println!("\n\nStop reason: {:?}", reason);
                }
            }
            StreamEvent::MessageStop => {
                println!("\n\nMessage completed!");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}