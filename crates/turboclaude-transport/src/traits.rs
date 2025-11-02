//! Transport trait and implementations
//!
//! Defines the generic Transport trait that can be implemented by different
//! transport mechanisms (HTTP, subprocess, etc.).

use crate::error::Result;
use async_trait::async_trait;

/// HTTP request specification
///
/// Represents an HTTP request to be sent via the Transport.
#[derive(Debug, Clone)]
pub struct HttpRequest {
    /// HTTP method (GET, POST, etc.)
    pub method: String,

    /// Request URL
    pub url: String,

    /// Request headers
    pub headers: std::collections::HashMap<String, String>,

    /// Request body (optional)
    pub body: Option<Vec<u8>>,
}

impl HttpRequest {
    /// Create a new HTTP request
    pub fn new(method: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            url: url.into(),
            headers: std::collections::HashMap::new(),
            body: None,
        }
    }

    /// Add a header to the request
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set the request body
    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    /// Set the request body from string
    pub fn with_text_body(mut self, text: impl Into<String>) -> Self {
        self.body = Some(text.into().into_bytes());
        self
    }
}

/// HTTP response
///
/// Represents an HTTP response received from the server.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// HTTP status code
    pub status: u16,

    /// Response headers
    pub headers: std::collections::HashMap<String, String>,

    /// Response body
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Create a new HTTP response
    pub fn new(
        status: u16,
        headers: std::collections::HashMap<String, String>,
        body: Vec<u8>,
    ) -> Self {
        Self {
            status,
            headers,
            body,
        }
    }

    /// Check if response is successful (2xx)
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    /// Check if response is an error (4xx or 5xx)
    pub fn is_error(&self) -> bool {
        self.status >= 400
    }

    /// Get the response body as a string
    pub fn text(&self) -> std::result::Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }

    /// Parse response body as JSON
    ///
    /// # Errors
    ///
    /// Returns an error if the response body cannot be parsed as valid JSON
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> crate::error::Result<T> {
        serde_json::from_slice(&self.body)
            .map_err(|e| crate::error::TransportError::Serialization(e.to_string()))
    }

    /// Get a header value by name (case-insensitive)
    pub fn get_header(&self, name: &str) -> Option<&str> {
        let name_lower = name.to_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == name_lower)
            .map(|(_, v)| v.as_str())
    }
}

/// Generic transport trait for different transport mechanisms
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send an HTTP request and receive a response
    async fn send_http(&self, request: HttpRequest) -> Result<HttpResponse>;

    /// Check if transport is connected
    async fn is_connected(&self) -> bool;

    /// Close the transport connection
    async fn close(&mut self) -> Result<()>;
}
