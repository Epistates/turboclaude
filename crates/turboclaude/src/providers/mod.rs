//! Alternative Claude API providers (AWS Bedrock, Google Vertex AI)
//!
//! This module provides support for accessing Claude models through different cloud providers.
//! Each provider offers the same Claude capabilities but with provider-specific authentication
//! and API endpoints.
//!
//! ## Available Providers
//!
//! ### AWS Bedrock
//! Access Claude models through AWS Bedrock. Requires the `bedrock` feature flag.
//!
//! ```rust,no_run
//! # #[cfg(feature = "bedrock")]
//! # {
//! use turboclaude::providers::bedrock::BedrockClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = BedrockClient::builder()
//!     .region("us-east-1")
//!     .build()
//!     .await?;
//!
//! let message = client.messages()
//!     .create(/* ... */)
//!     .await?;
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! ### Google Vertex AI
//! Access Claude models through Google Cloud Vertex AI. Requires the `vertex` feature flag.
//!
//! ```rust,no_run
//! # #[cfg(feature = "vertex")]
//! # {
//! use turboclaude::providers::vertex::VertexClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = VertexClient::builder()
//!     .project_id("my-gcp-project")
//!     .region("us-central1")
//!     .build()
//!     .await?;
//!
//! let message = client.messages()
//!     .create(/* ... */)
//!     .await?;
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! ## Provider Comparison
//!
//! | Feature | Direct Anthropic | AWS Bedrock | Google Vertex |
//! |---------|-----------------|-------------|---------------|
//! | Authentication | API Key | AWS IAM | Google ADC |
//! | Regions | Global | AWS Regions | GCP Regions |
//! | Batching | ✅ | ❌ | ❌ |
//! | Streaming | ✅ | ✅ | ✅ |
//! | Prompt Caching | ✅ | TBD | TBD |
//!
//! ## Architecture
//!
//! Each provider is implemented as a separate client that reuses the core SDK infrastructure:
//! - HTTP client and middleware (retries, timeouts, etc.)
//! - Type system (Message, ContentBlock, Tool, etc.)
//! - Resource abstractions (Messages, Beta, etc.)
//!
//! Providers only customize:
//! 1. Authentication mechanism
//! 2. Base URL construction
//! 3. Request/response transformation for provider-specific APIs

/// Shared utilities used by multiple provider implementations
///
/// This module contains code that is reused across providers (Bedrock, Vertex, etc.)
/// to avoid duplication and maintain consistency. This includes:
/// - System prompt extraction
/// - Content block transformation
/// - HTTP error handling
/// - Request deserialization
pub mod shared;

#[cfg(feature = "bedrock")]
#[cfg_attr(docsrs, doc(cfg(feature = "bedrock")))]
pub mod bedrock;

#[cfg(feature = "vertex")]
#[cfg_attr(docsrs, doc(cfg(feature = "vertex")))]
pub mod vertex;
