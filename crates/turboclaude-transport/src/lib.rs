//! Generic transport abstraction layer for TurboClaude
//!
//! Provides a trait-based transport abstraction that enables both REST HTTP
//! and CLI subprocess transports. This layer is used by both the REST client
//! and Agent client to communicate with Claude.
//!
//! # Architecture
//!
//! - **Transport trait**: Generic interface for any transport implementation
//! - **HTTP transport**: REST API client via reqwest

#![deny(unsafe_code)]
#![warn(missing_docs)]
//! - **Subprocess transport**: CLI-based communication via stdin/stdout
//! - **Error handling**: Unified error types across transports
//!
//! # Usage
//!
//! ```ignore
//! use turboclaude_transport::{Transport, http::HttpTransport};
//! use turboclaude_transport::traits::HttpRequest;
//!
//! let transport = HttpTransport::new()?;
//! let request = HttpRequest::new("GET", "https://api.anthropic.com/v1/messages");
//! let response = transport.send_http(request).await?;
//! ```

pub mod error;
pub mod http;
pub mod subprocess;
pub mod traits;

// Re-export commonly used types
pub use error::{Result, TransportError};
pub use http::HttpTransport;
pub use subprocess::{CliTransport, ProcessConfig};
pub use traits::{HttpRequest, HttpResponse, Transport};
