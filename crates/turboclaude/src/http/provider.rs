// ! HTTP provider trait for abstracting different API backends
//!
//! This module defines the `HttpProvider` trait which allows the SDK to work
//! with different HTTP backends (standard Anthropic API, AWS Bedrock, Google Vertex, etc.)
//! while maintaining a unified interface.

use crate::{
    error::Result,
    http::{Method, RequestBuilder, Response},
};
use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;
use std::fmt;

/// Provider trait for making HTTP requests to different backends.
///
/// This trait abstracts the HTTP layer to allow the SDK to work with:
/// - Standard Anthropic API (`AnthropicHttpProvider`)
/// - AWS Bedrock (`BedrockHttpProvider`)
/// - Google Vertex AI (`VertexHttpProvider`)
/// - Future providers as needed
///
/// Implementations handle authentication, request/response translation,
/// and streaming for their specific backend.
#[async_trait]
pub trait HttpProvider: Send + Sync + fmt::Debug {
    /// Make a request and return the raw response.
    ///
    /// # Arguments
    ///
    /// * `method` - HTTP method (GET, POST, etc.)
    /// * `path` - API endpoint path (e.g., "/v1/messages")
    /// * `body` - Optional request body (will be serialized to JSON)
    ///
    /// # Returns
    ///
    /// A `Response` object that can be parsed into the desired type.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The request fails (network, timeout, etc.)
    /// - The API returns an error status code
    async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<&(dyn erased_serde::Serialize + Send + Sync)>,
    ) -> Result<Response>;

    /// Make a streaming request and return a stream of bytes.
    ///
    /// # Arguments
    ///
    /// * `method` - HTTP method (usually POST for streaming)
    /// * `path` - API endpoint path (e.g., "/v1/messages")
    /// * `body` - Optional request body (will be serialized to JSON)
    ///
    /// # Returns
    ///
    /// A stream of byte chunks that can be parsed into events.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The request fails to initiate
    /// - Authentication fails
    /// - The endpoint doesn't support streaming
    async fn request_streaming(
        &self,
        method: Method,
        path: &str,
        body: Option<&(dyn erased_serde::Serialize + Send + Sync)>,
    ) -> Result<Box<dyn Stream<Item = Result<Bytes>> + Send + Unpin>>;

    /// Create a `RequestBuilder` for this provider.
    ///
    /// This method allows resources to use the builder pattern while
    /// still working with provider abstractions.
    ///
    /// # Arguments
    ///
    /// * `method` - HTTP method
    /// * `path` - API endpoint path
    ///
    /// # Returns
    ///
    /// A configured `RequestBuilder` for this provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the URL cannot be constructed.
    fn create_request(&self, method: Method, path: &str) -> Result<RequestBuilder>;

    /// Get the provider name for debugging/logging.
    fn provider_name(&self) -> &'static str;

    /// Check if this provider supports beta features.
    ///
    /// Not all providers may support beta endpoints.
    fn supports_beta(&self) -> bool {
        true
    }

    /// Get the base URL for this provider (for debugging).
    fn base_url(&self) -> &str;

    /// Cast to `std::any::Any` for downcasting to concrete types.
    ///
    /// This allows callers to downcast to specific provider implementations
    /// when needed for provider-specific functionality.
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Helper function to serialize a body to JSON bytes.
///
/// This is used internally by provider implementations to convert
/// request bodies to the format needed by HTTP clients.
pub(crate) fn serialize_body(
    body: &(dyn erased_serde::Serialize + Send + Sync),
) -> Result<Vec<u8>> {
    serde_json::to_vec(body).map_err(crate::error::Error::Serialization)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestRequest {
        message: String,
    }

    #[test]
    fn test_serialize_body() {
        let req = TestRequest {
            message: "test".to_string(),
        };
        let bytes = serialize_body(&req).unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["message"], "test");
    }
}
