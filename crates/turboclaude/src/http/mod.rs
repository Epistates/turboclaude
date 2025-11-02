//! HTTP client abstraction and middleware
//!
//! This module provides the HTTP layer for the SDK, including retry logic,
//! rate limiting, and middleware support similar to the Python SDK.

pub use anthropic_provider::{AnthropicHttpProvider, AnthropicHttpProviderBuilder};
pub use provider::HttpProvider;
pub use request::RequestBuilder;
pub use response::{RawResponse, Response};

mod anthropic_provider;
pub mod middleware;
pub mod provider;
mod request;
mod response;

// Re-export HTTP types from the http crate for convenience
pub use http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
