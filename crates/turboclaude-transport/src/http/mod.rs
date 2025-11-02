//! HTTP transport implementation
//!
//! Provides an HTTP client that implements the Transport trait.
//! Handles retries, rate limiting, middleware, and all HTTP concerns.

pub mod client;
pub mod retry;

pub use client::HttpTransport;
pub use retry::RetryPolicy;
