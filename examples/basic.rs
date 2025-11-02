//! Basic example demonstrating message creation with the Anthropic SDK

use anthropic::{Client, types::{MessageRequest, Message, Models}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client with your API key
    // The API key can also be set via the ANTHROPIC_API_KEY environment variable
    let client = Client::builder()
        .api_key("sk-ant-api-key")  // Replace with your actual API key
        .build()?;

    // Create a simple message request
    let request = MessageRequest::builder()
        .model(Models::CLAUDE_4_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![
            Message::user("What is the capital of France?")
        ])
        .build()?;

    println!("Sending request to Claude...");

    // Send the request
    match client.messages().create(request).await {
        Ok(message) => {
            println!("\nResponse from Claude:");
            println!("{}", message.text());
            println!("\nUsage:");
            println!("  Input tokens: {}", message.usage.input_tokens);
            println!("  Output tokens: {}", message.usage.output_tokens);
            println!("  Total tokens: {}", message.usage.total_tokens());
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}