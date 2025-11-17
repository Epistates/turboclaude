//! # TurboClaude SDK
//!
//! Rust SDK for Anthropic's Claude API supporting:
//! - Messages API with streaming
//! - Batch processing
//! - Tool use and function calling
//! - Multi-cloud providers (Anthropic, AWS Bedrock, Google Vertex AI)
//! - Multiple authentication methods
//! - Automatic retries and rate limiting
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use turboclaude::{Client, Message, MessageRequest};
//! use turboclaude_protocol::types::models;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new("your-api-key");
//!
//!     let message = client.messages()
//!         .create(MessageRequest::builder()
//!             .model(models::CLAUDE_SONNET_4_5_20250514)
//!             .max_tokens(1024u32)
//!             .messages(vec![
//!                 Message::user("Hello, Claude!")
//!             ])
//!             .build()?)
//!         .await?;
//!
//!     println!("{}", message.text());
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![deny(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// Re-export commonly used types
pub use client::Client;
pub use config::ClientConfig;
pub use context::{AdaptiveStrategy, PruningPolicy};
pub use error::{Error, Result};
pub use http::RawResponse;
pub use resources::{BatchRequest, TokenCount};
pub use types::*;

// Module declarations
pub mod client;
pub mod config;
pub mod context;
pub mod error;
pub mod http;
pub mod observability;
pub mod resources;
pub mod streaming;
pub mod streaming_validation;
pub mod types;
pub mod validation;

// Schema generation utilities (requires schema feature)
#[cfg(feature = "schema")]
#[cfg_attr(docsrs, doc(cfg(feature = "schema")))]
pub mod schema;

// Provider modules (optional, feature-gated)
#[cfg(any(feature = "bedrock", feature = "vertex"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "bedrock", feature = "vertex"))))]
pub mod providers;

// Tools module (requires schema feature for full functionality)
#[cfg(feature = "schema")]
#[cfg_attr(docsrs, doc(cfg(feature = "schema")))]
pub mod tools;

// Re-export key dependencies for convenience
pub use async_trait::async_trait;
pub use serde::{Deserialize, Serialize};
pub use serde_json::Value as JsonValue;

/// Prelude module for common imports
///
/// # Examples
///
/// ```rust
/// use turboclaude::prelude::*;
/// ```
pub mod prelude {

    pub use crate::{
        Client, ClientConfig, Error, Result,
        streaming::MessageStream,
        types::{
            ContentBlock, Message, MessageRequest, MessageRequestBuilder, Model, Role, Tool,
            ToolChoice, Usage,
        },
    };
}

/// SDK version, automatically updated from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default API base URL
pub const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";

/// Default API version header value
pub const DEFAULT_API_VERSION: &str = "2023-06-01";

// Constants from Python SDK
/// Legacy human prompt constant (for compatibility)
pub const HUMAN_PROMPT: &str = "\n\nHuman:";

/// Legacy AI prompt constant (for compatibility)
pub const AI_PROMPT: &str = "\n\nAssistant:";

#[cfg(test)]
mod property_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_constants() {
        assert_eq!(DEFAULT_BASE_URL, "https://api.anthropic.com");
        assert_eq!(DEFAULT_API_VERSION, "2023-06-01");
    }
}
