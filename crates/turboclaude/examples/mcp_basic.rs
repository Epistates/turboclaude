//! Multi-Claude Provider (MCP) Basic Example
//!
//! This example demonstrates how to use the `McpHttpProvider` to create a
//! client that can route requests to multiple Claude providers (Anthropic,
//! Bedrock, Vertex) based on model availability and priority.
//!
//! ## Prerequisites
//!
//! 1. **Anthropic API Key**:
//!    - Environment variable: `ANTHROPIC_API_KEY`
//!
//! 2. **AWS Credentials** (for Bedrock):
//!    - Environment variables: `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`
//!    - AWS credentials file: `~/.aws/credentials`
//!    - IAM role (when running on AWS services)
//!
//! 3. **GCP Credentials** (for Vertex):
//!    - Environment variable: `GOOGLE_APPLICATION_CREDENTIALS` pointing to your
//!      service account JSON key file.
//!    - Or run `gcloud auth application-default login`
//!
//! 4. **Region/Project Configuration**:
//!    - `AWS_REGION` (e.g., "us-east-1")
//!    - `VERTEX_PROJECT_ID`
//!    - `VERTEX_LOCATION` (e.g., "us-central1")
//!
//! ## Usage
//!
//! Make sure to set all required environment variables first.
//!
//! ```bash
//! # Run with all features enabled for MCP
//! cargo run --example mcp_basic --features="anthropic,bedrock,vertex"
//! ```

use std::sync::Arc;
use turboclaude::{
    Client,
    providers::{
        anthropic::AnthropicHttpProvider,
        bedrock::BedrockHttpProvider,
        vertex::VertexHttpProvider,
        mcp::McpHttpProvider,
    },
    types::{MessageParam, MessageRequest, Role},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for debugging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 Multi-Claude Provider (MCP) Basic Example\n");

    // --- Provider Setup ---
    println!("🔧 Setting up individual providers...");

    // 1. Anthropic Provider (requires ANTHROPIC_API_KEY)
    let anthropic_provider = Arc::new(
        AnthropicHttpProvider::builder()
            .api_key(std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY not set"))
            .build()?,
    );
    println!("   ✅ Anthropic provider configured.");

    // 2. AWS Bedrock Provider (requires AWS credentials and region)
    let bedrock_provider = Arc::new(
        BedrockHttpProvider::builder()
            .region(std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string()))
            .build()
            .await?,
    );
    println!("   ✅ AWS Bedrock provider configured.");

    // 3. Google Vertex Provider (requires GCP credentials and project info)
    let vertex_provider = Arc::new(
        VertexHttpProvider::builder()
            .project_id(std::env::var("VERTEX_PROJECT_ID").expect("VERTEX_PROJECT_ID not set"))
            .location(std::env::var("VERTEX_LOCATION").expect("VERTEX_LOCATION not set"))
            .build()
            .await?,
    );
    println!("   ✅ Google Vertex provider configured.");

    // --- MCP Setup ---
    println!("\n🔧 Creating Multi-Claude Provider (MCP)...");

    // Combine providers into MCP. The order determines priority.
    // Here, Anthropic is tried first, then Bedrock, then Vertex.
    let mcp = Arc::new(
        McpHttpProvider::builder()
            .with_provider(anthropic_provider)
            .with_provider(bedrock_provider)
            .with_provider(vertex_provider)
            .build(),
    );
    println!("   ✅ MCP created with 3 providers.");

    // --- Client and Request ---
    let client = Client::from_provider(mcp);

    println!("\n📝 Creating message request...");
    let request = MessageRequest::builder()
        // Use a generic model name. MCP will find a provider that supports it.
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![MessageParam {
            role: Role::User,
            content: vec![turboclaude::types::ContentBlockParam::Text {
                text: "Hello! Explain what a Multi-Claude Provider (MCP) is in one sentence.".to_string(),
            }],
        }])
        .build()?;

    println!("   ✅ Request created for model: {}", request.model);

    // --- Send Request via MCP ---
    println!("\n📤 Sending request via MCP...");
    let response = client.messages().create(request).await?;
    println!("   ✅ Response received!");

    // The model ID in the response will be the fully-qualified name
    // used by the provider that handled the request.
    let provider_used = if response.model.starts_with("anthropic.") {
        "AWS Bedrock"
    } else if response.model.contains("/models/") {
        "Google Vertex"
    } else {
        "Anthropic"
    };

    println!("\n📨 Response Details:");
    println!("   Provider Used: {}", provider_used);
    println!("   Model (fully-qualified): {}", response.model);
    println!("   Stop reason: {:?}", response.stop_reason);
    println!("\n📊 Usage:");
    println!("   Input tokens: {}", response.usage.input_tokens);
    println!("   Output tokens: {}", response.usage.output_tokens);
    println!("\n💬 Content:");

    println!("   {}", response.text());

    println!("\n✨ MCP example completed successfully!");

    Ok(())
}
