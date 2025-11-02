//! Error types for the Anthropic SDK
//!
//! This module provides a comprehensive error hierarchy similar to the Python SDK's
//! exception types, but following Rust idioms with the `thiserror` crate.

use std::time::Duration;
use thiserror::Error;

/// Result type alias for operations that can fail with an Anthropic SDK error.
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the Anthropic SDK.
///
/// This enum represents all possible errors that can occur when using the SDK,
/// mirroring the Python SDK's exception hierarchy but adapted for Rust.
#[derive(Debug, Error)]
pub enum Error {
    /// API returned a bad request error (400).
    #[error("Bad request: {message}")]
    BadRequest {
        /// Error message from the API
        message: String,
        /// Optional error type from API
        error_type: Option<String>,
    },

    /// Authentication failed (401).
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Permission denied (403).
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Resource not found (404).
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Conflict error (409).
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Unprocessable entity (422).
    #[error("Unprocessable entity: {message}")]
    UnprocessableEntity {
        /// Error message
        message: String,
        /// Validation errors if provided
        errors: Option<Vec<ValidationError>>,
    },

    /// Rate limit exceeded (429).
    #[error("Rate limit exceeded")]
    RateLimit {
        /// Time to wait before retrying, if provided by the API
        retry_after: Option<Duration>,
        /// Number of requests allowed
        limit: Option<u32>,
        /// Number of requests remaining
        remaining: Option<u32>,
        /// Time when the rate limit resets
        reset_at: Option<chrono::DateTime<chrono::Utc>>,
    },

    /// Internal server error (500+).
    #[error("Internal server error: {0}")]
    InternalServerError(String),

    /// Service overloaded (529).
    #[error("Service overloaded: {0}")]
    Overloaded(String),

    /// Generic API error for status codes not covered above.
    #[error("API error (status {status}): {message}")]
    ApiError {
        /// HTTP status code
        status: u16,
        /// Error message
        message: String,
        /// Optional error type from API
        error_type: Option<String>,
        /// Request ID for debugging
        request_id: Option<String>,
    },

    /// Failed to deserialize API response.
    #[error("Failed to parse API response: {0}")]
    ResponseValidation(String),

    /// Network or connection error.
    #[error("Connection error: {0}")]
    Connection(String),

    /// Request timeout.
    #[error("Request timeout after {0:?}")]
    Timeout(Duration),

    /// Invalid request parameters.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Invalid URL provided.
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Streaming error.
    #[error("Streaming error: {0}")]
    Streaming(String),

    /// HTTP client configuration or initialization error.
    #[error("HTTP client error: {0}")]
    HttpClient(String),

    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Generic I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Missing required configuration.
    #[error("Missing required configuration: {0}")]
    MissingConfig(String),

    /// Feature not available (used for feature-gated functionality).
    #[error("Feature not available: {0}. Enable the '{1}' feature to use this functionality.")]
    FeatureNotAvailable(&'static str, &'static str),

    /// Tool execution error
    #[error("Tool execution error: {0}")]
    ToolExecution(String),

    /// Generic error with context.
    #[error("{context}: {source}")]
    WithContext {
        /// Context description
        context: String,
        /// Underlying error
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Other errors not covered by specific variants.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Validation error details for UnprocessableEntity errors.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Field that failed validation
    pub field: String,
    /// Validation error message
    pub message: String,
    /// Error code if provided
    pub code: Option<String>,
}

impl Error {
    /// Create an API error from an HTTP response status and body.
    pub fn from_response(status: u16, body: &str, headers: &http::HeaderMap) -> Self {
        // Try to parse error from JSON body
        if let Ok(api_error) = serde_json::from_str::<ApiErrorResponse>(body) {
            match status {
                400 => Error::BadRequest {
                    message: api_error.error.message,
                    error_type: Some(api_error.error.error_type),
                },
                401 => Error::Authentication(api_error.error.message),
                403 => Error::PermissionDenied(api_error.error.message),
                404 => Error::NotFound(api_error.error.message),
                409 => Error::Conflict(api_error.error.message),
                422 => {
                    // Parse validation errors if present
                    let errors = api_error.error.details
                        .and_then(|d| d.validation_errors)
                        .map(|ve| {
                            ve.into_iter()
                                .map(|e| ValidationError {
                                    field: e.field,
                                    message: e.message,
                                    code: e.code,
                                })
                                .collect()
                        });

                    Error::UnprocessableEntity {
                        message: api_error.error.message,
                        errors,
                    }
                }
                429 => {
                    // Parse rate limit headers
                    let retry_after = headers
                        .get("retry-after")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|s| s.parse::<u64>().ok())
                        .map(Duration::from_secs);

                    Error::RateLimit {
                        retry_after,
                        limit: parse_header_u32(headers, "anthropic-ratelimit-limit"),
                        remaining: parse_header_u32(headers, "anthropic-ratelimit-remaining"),
                        reset_at: parse_header_datetime(headers, "anthropic-ratelimit-reset"),
                    }
                }
                529 => Error::Overloaded(api_error.error.message),
                s if s >= 500 => Error::InternalServerError(api_error.error.message),
                _ => Error::ApiError {
                    status,
                    message: api_error.error.message,
                    error_type: Some(api_error.error.error_type),
                    request_id: headers
                        .get("x-request-id")
                        .and_then(|v| v.to_str().ok())
                        .map(String::from),
                },
            }
        } else {
            // Fallback to simple status-based error
            match status {
                400 => Error::BadRequest {
                    message: body.to_string(),
                    error_type: None,
                },
                401 => Error::Authentication(body.to_string()),
                403 => Error::PermissionDenied(body.to_string()),
                404 => Error::NotFound(body.to_string()),
                409 => Error::Conflict(body.to_string()),
                422 => Error::UnprocessableEntity {
                    message: body.to_string(),
                    errors: None,
                },
                429 => Error::RateLimit {
                    retry_after: None,
                    limit: None,
                    remaining: None,
                    reset_at: None,
                },
                529 => Error::Overloaded(body.to_string()),
                s if s >= 500 => Error::InternalServerError(body.to_string()),
                _ => Error::ApiError {
                    status,
                    message: body.to_string(),
                    error_type: None,
                    request_id: None,
                },
            }
        }
    }

    /// Check if this error is retryable.
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::RateLimit { .. } => true,
            Error::InternalServerError(_) => true,
            Error::Overloaded(_) => true,
            Error::Connection(_) => true,
            Error::Timeout(_) => true,
            Error::ApiError { status, .. } => *status >= 500 || *status == 408 || *status == 409,
            _ => false,
        }
    }

    /// Get retry delay if this is a rate limit error with retry-after.
    pub fn retry_after(&self) -> Option<Duration> {
        if let Error::RateLimit { retry_after, .. } = self {
            *retry_after
        } else {
            None
        }
    }

    /// Add context to an error.
    pub fn context<C>(self, context: C) -> Self
    where
        C: std::fmt::Display + Send + Sync + 'static,
    {
        Error::WithContext {
            context: context.to_string(),
            source: Box::new(self),
        }
    }
}

// Helper structures for parsing API error responses

#[derive(Debug, serde::Deserialize)]
struct ApiErrorResponse {
    error: ApiErrorDetails,
}

#[derive(Debug, serde::Deserialize)]
struct ApiErrorDetails {
    message: String,
    #[serde(rename = "type")]
    error_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<ErrorDetails>,
}

#[derive(Debug, serde::Deserialize)]
struct ErrorDetails {
    validation_errors: Option<Vec<ApiValidationError>>,
}

#[derive(Debug, serde::Deserialize)]
struct ApiValidationError {
    field: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<String>,
}

// Helper functions for parsing headers

fn parse_header_u32(headers: &http::HeaderMap, name: &str) -> Option<u32> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
}

fn parse_header_datetime(headers: &http::HeaderMap, name: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_is_retryable() {
        assert!(Error::RateLimit {
            retry_after: None,
            limit: None,
            remaining: None,
            reset_at: None,
        }
        .is_retryable());

        assert!(Error::InternalServerError("test".to_string()).is_retryable());
        assert!(Error::Connection("test".to_string()).is_retryable());
        assert!(Error::Timeout(Duration::from_secs(30)).is_retryable());

        assert!(!Error::BadRequest {
            message: "test".to_string(),
            error_type: None,
        }
        .is_retryable());

        assert!(!Error::Authentication("test".to_string()).is_retryable());
    }

    #[test]
    fn test_error_retry_after() {
        let error = Error::RateLimit {
            retry_after: Some(Duration::from_secs(60)),
            limit: None,
            remaining: None,
            reset_at: None,
        };

        assert_eq!(error.retry_after(), Some(Duration::from_secs(60)));

        let error = Error::BadRequest {
            message: "test".to_string(),
            error_type: None,
        };

        assert_eq!(error.retry_after(), None);
    }

    #[test]
    fn test_error_context() {
        let error = Error::NotFound("resource".to_string());
        let with_context = error.context("Failed to fetch message");

        match with_context {
            Error::WithContext { context, .. } => {
                assert_eq!(context, "Failed to fetch message");
            }
            _ => panic!("Expected WithContext variant"),
        }
    }
}