//! AWS Bedrock provider for Claude models
//!
//! This module provides access to Claude models through AWS Bedrock using the official AWS SDK.
//!
//! ## Authentication
//!
//! The Bedrock provider uses AWS credentials through the standard AWS credential chain:
//! - Environment variables (AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, AWS_SESSION_TOKEN)
//! - AWS credentials file (~/.aws/credentials)
//! - IAM role (when running on AWS services like EC2, Lambda, ECS)
//! - AWS profile (via aws_profile parameter)
//!
//! ## Example
//!
//! ```rust,no_run
//! use turboclaude::Client;
//! use turboclaude::providers::bedrock::BedrockHttpProvider;
//! use turboclaude::types::MessageRequest;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create Bedrock provider
//! let provider = Arc::new(BedrockHttpProvider::builder()
//!     .region("us-east-1")
//!     .build()
//!     .await?);
//!
//! // Use with standard Client (once Client supports custom providers)
//! // let client = Client::from_provider(provider);
//!
//! // Send a message
//! // let response = client.messages()
//! //     .create(MessageRequest::builder()
//! //         .model("claude-3-5-sonnet-20241022")
//! //         .max_tokens(1024u32)
//! //         .messages(vec![Message::user("Hello from Bedrock!")])
//! //         .build()?)
//! //     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Model IDs
//!
//! AWS Bedrock uses provider-specific model IDs. Common Claude model IDs:
//! - `anthropic.claude-3-5-sonnet-20241022-v2:0` - Claude 3.5 Sonnet (latest)
//! - `anthropic.claude-3-5-sonnet-20240620-v1:0` - Claude 3.5 Sonnet (June 2024)
//! - `anthropic.claude-3-opus-20240229-v1:0` - Claude 3 Opus
//! - `anthropic.claude-3-sonnet-20240229-v1:0` - Claude 3 Sonnet
//! - `anthropic.claude-3-haiku-20240307-v1:0` - Claude 3 Haiku
//!
//! The provider will automatically transform short model names (e.g., "claude-3-5-sonnet-20241022")
//! to Bedrock format if needed.
//!
//! ## Limitations
//!
//! The following features are not yet supported in AWS Bedrock:
//! - Message Batches API
//! - Token counting API
//! - Beta endpoints
//!
//! ## References
//!
//! - [AWS Bedrock Documentation](https://docs.aws.amazon.com/bedrock/)
//! - [Anthropic on AWS Bedrock](https://docs.anthropic.com/en/api/claude-on-amazon-bedrock)

mod error;
mod http; // HttpProvider implementation
mod translate; // Type translation logic

// Export HttpProvider-based implementation
pub use error::BedrockError;
pub use http::{BedrockHttpProvider, BedrockHttpProviderBuilder};

/// Default API version for Bedrock
pub const BEDROCK_API_VERSION: &str = "bedrock-2023-05-31";
