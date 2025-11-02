//! Basic Google Vertex AI example
//!
//! This example demonstrates how to use the VertexHttpProvider to send
//! messages to Claude via Google Cloud Vertex AI.
//!
//! ## Prerequisites
//!
//! 1. Google Cloud project with Vertex AI API enabled
//! 2. Authentication configured (one of):
//!    - Access token via: `gcloud auth application-default print-access-token`
//!    - Service account key file (via GOOGLE_APPLICATION_CREDENTIALS)
//!    - Application Default Credentials (when running on GCP)
//!
//! 3. Set environment variables:
//!    - GOOGLE_CLOUD_PROJECT: Your GCP project ID
//!    - VERTEX_REGION: GCP region (e.g., "us-east5")
//!    - VERTEX_ACCESS_TOKEN: Access token from gcloud CLI
//!
//! ## Usage
//!
//! ```bash
//! # Get access token
//! export VERTEX_ACCESS_TOKEN=$(gcloud auth application-default print-access-token)
//! export GOOGLE_CLOUD_PROJECT=my-gcp-project
//! export VERTEX_REGION=us-east5
//!
//! cargo run --example vertex_basic --features vertex,trace
//! ```

use std::sync::Arc;
use turboclaude::{
    Client,
    providers::vertex::VertexHttpProvider,
    types::{MessageParam, MessageRequest, Role},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for debugging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 Google Vertex AI Basic Example\n");

    // Get configuration from environment
    let project_id = std::env::var("GOOGLE_CLOUD_PROJECT")
        .expect("GOOGLE_CLOUD_PROJECT environment variable must be set");
    let region = std::env::var("VERTEX_REGION").unwrap_or_else(|_| "us-east5".to_string());
    let access_token = std::env::var("VERTEX_ACCESS_TOKEN")
        .expect("VERTEX_ACCESS_TOKEN must be set. Get it via: gcloud auth application-default print-access-token");

    // Create Vertex provider
    println!("🔧 Creating Vertex AI provider...");
    let provider = Arc::new(
        VertexHttpProvider::builder()
            .project_id(project_id.clone())
            .region(region.clone())
            .access_token(access_token)
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .await?,
    );

    println!("✅ Provider created successfully!");
    println!("   Project: {}", project_id);
    println!("   Region: {}\n", region);

    // Create client with Vertex provider
    let client = Client::from_provider(provider);

    // Create a simple message request
    println!("📝 Creating message request...");
    let request = MessageRequest::builder()
        // Vertex AI model format: claude-{model}@{date}
        .model("claude-sonnet-4-5@20250929")
        .max_tokens(1024u32)
        .messages(vec![MessageParam {
            role: Role::User,
            content: vec![turboclaude::types::ContentBlockParam::Text {
                text: "Hello! Please tell me a short fact about Google Cloud Vertex AI in one sentence."
                    .to_string(),
            }],
        }])
        .build()
        .expect("Failed to build message request");

    println!("✅ Request created");
    println!("   Model: {}", request.model);
    println!("   Max tokens: {}\n", request.max_tokens);

    // Send the request
    println!("📤 Sending request to Vertex AI...");
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
