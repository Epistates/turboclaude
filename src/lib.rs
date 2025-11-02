//! # Anthropic Rust SDK
//!
//! A fully-featured, idiomatic Rust SDK for Anthropic's Claude API.
//!
//! This SDK provides complete feature parity with the official Python SDK,
//! including support for:
//! - Messages API (with streaming)
//! - Batch processing
//! - Tool use
//! - Beta features
//! - Multiple authentication methods
//! - Automatic retries and rate limiting
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use anthropic::{Client, Message, MessageRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new("your-api-key");
//!
//!     let message = client.messages()
//!         .create(MessageRequest::builder()
//!             .model("claude-3-5-sonnet-20241022")
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
pub use error::{Error, Result};
pub use http::RawResponse;
pub use resources::{TokenCount, BatchRequest};
pub use types::*;

// Module declarations
pub mod client;
pub mod config;
pub mod error;
pub mod http;
pub mod resources;
pub mod streaming;
pub mod types;

// Tools module (requires schema feature for full functionality)
#[cfg(feature = "schema")]
#[cfg_attr(docsrs, doc(cfg(feature = "schema")))]
pub mod tools;

// Optional blocking client
#[cfg(feature = "blocking")]
#[cfg_attr(docsrs, doc(cfg(feature = "blocking")))]
pub mod blocking;

// Optional MCP integration
#[cfg(feature = "mcp")]
#[cfg_attr(docsrs, doc(cfg(feature = "mcp")))]
pub mod mcp;

// Re-export key dependencies for convenience
pub use async_trait::async_trait;
pub use serde::{Deserialize, Serialize};
pub use serde_json::Value as JsonValue;

/// Prelude module for common imports
///
/// # Examples
///
/// ```rust
/// use anthropic::prelude::*;
/// ```
pub mod prelude {

    pub use crate::{
        Client, ClientConfig, Error, Result,
        types::{
            Message, MessageRequest, MessageRequestBuilder,
            ContentBlock, Role, Tool, ToolChoice,
            Usage, Model,
        },
        streaming::MessageStream,
    };

    #[cfg(feature = "blocking")]
    pub use crate::blocking::Client as BlockingClient;
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