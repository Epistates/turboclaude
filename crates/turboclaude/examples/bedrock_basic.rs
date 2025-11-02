//! Basic AWS Bedrock example
//!
//! This example demonstrates how to use the BedrockHttpProvider to send
//! messages to Claude via AWS Bedrock.
//!
//! ## Prerequisites
//!
//! 1. AWS credentials configured (one of):
//!    - Environment variables: AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY
//!    - AWS credentials file: ~/.aws/credentials
//!    - IAM role (when running on AWS services)
//!
//! 2. AWS region configured:
//!    - Environment variable: AWS_REGION
//!    - Or specify via builder: .region("us-east-1")
//!
//! 3. Access to Claude models in AWS Bedrock
//!
//! ## Usage
//!
//! ```bash
//! # Using default AWS credential chain
//! export AWS_REGION=us-east-1
//! cargo run --example bedrock_basic --features bedrock
//!
//! # With explicit region
//! AWS_REGION=us-west-2 cargo run --example bedrock_basic --features bedrock
//! ```

use std::sync::Arc;
use turboclaude::{
    Client,
    providers::bedrock::BedrockHttpProvider,
    types::{MessageParam, MessageRequest, Role},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for debugging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 AWS Bedrock Basic Example\n");

    // Create Bedrock provider
    println!("🔧 Creating Bedrock provider...");
    let provider = Arc::new(
        BedrockHttpProvider::builder()
            // Region can be omitted if AWS_REGION env var is set
            .region(std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string()))
            // Optional: configure timeout (default: 600 seconds)
            .timeout(std::time::Duration::from_secs(120))
            // Optional: configure retries (default: 2)
            .max_retries(3)
            .build()
            .await?,
    );

    println!("✅ Provider created successfully!");
    println!(
        "   Region: {}\n",
        std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string())
    );

    // Create client with Bedrock provider
    let client = Client::from_provider(provider);

    // Create a simple message request
    println!("📝 Creating message request...");
    let request = MessageRequest::builder()
        // Model ID - automatically normalized to Bedrock format
        // "claude-3-5-sonnet-20241022" → "anthropic.claude-3-5-sonnet-20241022-v2:0"
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![MessageParam {
            role: Role::User,
            content: vec![turboclaude::types::ContentBlockParam::Text {
                text: "Hello! Please tell me a short fact about AWS Bedrock in one sentence."
                    .to_string(),
            }],
        }])
        .build()
        .expect("Failed to build message request");

    println!("✅ Request created");
    println!("   Model: {}", request.model);
    println!("   Max tokens: {}\n", request.max_tokens);

    // Send the request
    println!("📤 Sending request to Bedrock...");
    let response = client
        .messages()
        .create(request)
        .await
        .expect("Failed to send message");

    println!("✅ Response received!\n");

    // Display the response
    println!("📨 Response Details:");
    println!("   ID: {}", response.id);
    println!("   Model: {}", response.model);
    println!("   Role: {:?}", response.role);
    println!("   Stop reason: {:?}", response.stop_reason);
    println!("\n📊 Usage:");
    println!("   Input tokens: {}", response.usage.input_tokens);
    println!("   Output tokens: {}", response.usage.output_tokens);
    println!("\n💬 Content:");

    for (i, block) in response.content.iter().enumerate() {
        if let turboclaude::types::ContentBlock::Text { text, citations: _ } = block {
            println!("   [Block {}]: {}", i + 1, text);
        }
    }

    println!("\n✨ Example completed successfully!");

    Ok(())
}
