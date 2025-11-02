//! HTTP client abstraction and middleware
//!
//! This module provides the HTTP layer for the SDK, including retry logic,
//! rate limiting, and middleware support similar to the Python SDK.

pub use request::RequestBuilder;
pub use response::{Response, RawResponse};
pub use retry::RetryConfig;

mod request;
mod response;
pub mod retry;
pub mod middleware;

// Re-export HTTP types from the http crate for convenience
pub use http::{Method, StatusCode, HeaderMap, HeaderName, HeaderValue};