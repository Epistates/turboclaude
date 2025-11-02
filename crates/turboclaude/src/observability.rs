//! Centralized observability utilities for structured logging and metrics
//!
//! This module provides reusable logging and metrics tracking to avoid duplication
//! across the codebase. All HTTP requests/responses are logged through this layer.

use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// HTTP request metadata for structured logging
#[derive(Debug, Clone)]
pub struct RequestMetadata {
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Request path
    pub path: String,
    /// Request body size in bytes (optional)
    pub body_size: Option<usize>,
}

impl RequestMetadata {
    /// Create new request metadata
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
            body_size: None,
        }
    }

    /// Set the request body size
    pub fn with_body_size(mut self, size: usize) -> Self {
        self.body_size = Some(size);
        self
    }

    /// Log request being sent
    pub fn log_request(&self) {
        debug!(
            method = %self.method,
            path = %self.path,
            body_size = self.body_size,
            "Sending HTTP request"
        );
    }
}

/// HTTP response metadata for structured logging
#[derive(Debug, Clone)]
pub struct ResponseMetadata {
    /// HTTP status code
    pub status: u16,
    /// Response body size in bytes (optional)
    pub body_size: Option<usize>,
    /// Time elapsed for the request
    pub elapsed: Duration,
    /// Number of retries taken (if any)
    pub retries: u32,
}

impl ResponseMetadata {
    /// Create new response metadata
    pub fn new(status: u16, elapsed: Duration) -> Self {
        Self {
            status,
            body_size: None,
            elapsed,
            retries: 0,
        }
    }

    /// Set the response body size
    pub fn with_body_size(mut self, size: usize) -> Self {
        self.body_size = Some(size);
        self
    }

    /// Set the number of retries
    pub fn with_retries(mut self, retries: u32) -> Self {
        self.retries = retries;
        self
    }

    /// Log successful response
    pub fn log_success(&self, request: &RequestMetadata) {
        info!(
            method = %request.method,
            path = %request.path,
            status = self.status,
            elapsed_ms = self.elapsed.as_millis(),
            body_size = self.body_size,
            retries = self.retries,
            "HTTP request succeeded"
        );
    }

    /// Log failed response
    pub fn log_error(&self, request: &RequestMetadata, error: &str) {
        warn!(
            method = %request.method,
            path = %request.path,
            status = self.status,
            elapsed_ms = self.elapsed.as_millis(),
            error = %error,
            retries = self.retries,
            "HTTP request failed"
        );
    }
}

/// Timer for measuring request duration
pub struct RequestTimer {
    start: Instant,
}

impl RequestTimer {
    /// Start a new timer
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Get elapsed duration
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

/// Stream event logging context
pub struct StreamContext {
    /// Total events received
    pub event_count: u32,
    /// Total duration
    pub elapsed: Duration,
}

impl StreamContext {
    /// Create new stream context
    pub fn new() -> Self {
        Self {
            event_count: 0,
            elapsed: Duration::from_secs(0),
        }
    }

    /// Log stream started
    pub fn log_started(path: &str) {
        debug!(path = %path, "Opening streaming response");
    }

    /// Log stream event received
    pub fn log_event(&mut self, event_type: &str) {
        self.event_count += 1;
        debug!(
            event_num = self.event_count,
            event_type = %event_type,
            "Stream event received"
        );
    }

    /// Log stream completed
    pub fn log_complete(&self, path: &str) {
        info!(
            path = %path,
            event_count = self.event_count,
            elapsed_ms = self.elapsed.as_millis(),
            "Stream completed successfully"
        );
    }

    /// Log stream error
    pub fn log_error(&self, path: &str, error: &str) {
        warn!(
            path = %path,
            event_count = self.event_count,
            elapsed_ms = self.elapsed.as_millis(),
            error = %error,
            "Stream error"
        );
    }
}

impl Default for StreamContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Log validation error
pub fn log_validation_error(field: &str, reason: &str) {
    debug!(
        field = %field,
        reason = %reason,
        "Request validation failed"
    );
}

/// Log validation success
pub fn log_validation_complete(field_count: usize) {
    debug!(fields_validated = field_count, "Request validation passed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_metadata_creation() {
        let metadata = RequestMetadata::new("POST", "/v1/messages");
        assert_eq!(metadata.method, "POST");
        assert_eq!(metadata.path, "/v1/messages");
        assert_eq!(metadata.body_size, None);
    }

    #[test]
    fn test_request_metadata_with_body_size() {
        let metadata = RequestMetadata::new("POST", "/v1/messages").with_body_size(1024);
        assert_eq!(metadata.body_size, Some(1024));
    }

    #[test]
    fn test_response_metadata_creation() {
        let elapsed = Duration::from_millis(500);
        let metadata = ResponseMetadata::new(200, elapsed);
        assert_eq!(metadata.status, 200);
        assert_eq!(metadata.elapsed, elapsed);
        assert_eq!(metadata.retries, 0);
    }

    #[test]
    fn test_response_metadata_with_retries() {
        let elapsed = Duration::from_millis(500);
        let metadata = ResponseMetadata::new(200, elapsed).with_retries(2);
        assert_eq!(metadata.retries, 2);
    }

    #[test]
    fn test_request_timer() {
        let timer = RequestTimer::start();
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed();
        assert!(elapsed.as_millis() >= 10);
    }

    #[test]
    fn test_stream_context() {
        let mut ctx = StreamContext::new();
        assert_eq!(ctx.event_count, 0);

        ctx.log_event("ContentBlockDelta");
        assert_eq!(ctx.event_count, 1);

        ctx.log_event("MessageStop");
        assert_eq!(ctx.event_count, 2);
    }
}
