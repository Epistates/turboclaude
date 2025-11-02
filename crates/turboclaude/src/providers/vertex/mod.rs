//! Google Vertex AI provider for Claude models
//!
//! This module provides access to Claude models through Google Cloud's Vertex AI
//! using the official Google Cloud Rust SDK.
//!
//! ## Authentication
//!
//! The Vertex provider uses Google Cloud credentials through the standard authentication chain:
//! - Application Default Credentials (ADC) via `gcloud auth application-default login`
//! - Service account key files
//! - Workload Identity (when running on GKE)
//! - Compute Engine default service account (when running on GCE)
//!
//! ## Example
//!
//! ```rust,no_run
//! use turboclaude::Client;
//! use turboclaude::providers::vertex::VertexHttpProvider;
//! use turboclaude::types::MessageRequest;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create Vertex provider
//! let provider = Arc::new(VertexHttpProvider::builder()
//!     .project_id("my-gcp-project")
//!     .region("us-east5")
//!     .build()
//!     .await?);
//!
//! // Use with standard Client
//! let client = Client::from_provider(provider);
//!
//! // Send a message
//! // let response = client.messages()
//! //     .create(MessageRequest::builder()
//! //         .model("claude-sonnet-4-5@20250929")
//! //         .max_tokens(1024u32)
//! //         .messages(vec![Message::user("Hello from Vertex AI!")])
//! //         .build()?);
//! # Ok(())
//! # }
//! ```
//!
//! ## Model IDs
//!
//! Google Vertex AI uses versioned model IDs:
//! - `claude-sonnet-4-5@20250929` - Claude 4.5 Sonnet (Sept 2025)
//! - `claude-opus-4-1@20250805` - Claude 4.1 Opus (Aug 2025)
//! - `claude-haiku-4-5@20250929` - Claude 4.5 Haiku (Sept 2025)
//!
//! ## Limitations
//!
//! The following features are not yet supported in Google Vertex AI:
//! - Message Batches API
//! - Token counting API
//!
//! ## References
//!
//! - [Google Vertex AI Documentation](https://cloud.google.com/vertex-ai/docs)
//! - [Anthropic on Vertex AI](https://docs.claude.com/claude/reference/claude-on-vertex-ai)

mod error;
mod http;

pub use error::VertexError;
pub use http::{VertexHttpProvider, VertexHttpProviderBuilder};

/// API version for Vertex AI
pub const VERTEX_API_VERSION: &str = "vertex-2023-10-16";
