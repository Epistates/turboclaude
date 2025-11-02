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

    /// Invalid HTTP header name.
    #[error("Invalid HTTP header name: {0}")]
    InvalidHeaderName(String),

    /// Invalid HTTP header value.
    #[error("Invalid HTTP header value: {0}")]
    InvalidHeaderValue(String),

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
                    let errors = api_error
                        .error
                        .details
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

fn parse_header_datetime(
    headers: &http::HeaderMap,
    name: &str,
) -> Option<chrono::DateTime<chrono::Utc>> {
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
        assert!(
            Error::RateLimit {
                retry_after: None,
                limit: None,
                remaining: None,
                reset_at: None,
            }
            .is_retryable()
        );

        assert!(Error::InternalServerError("test".to_string()).is_retryable());
        assert!(Error::Connection("test".to_string()).is_retryable());
        assert!(Error::Timeout(Duration::from_secs(30)).is_retryable());

        assert!(
            !Error::BadRequest {
                message: "test".to_string(),
                error_type: None,
            }
            .is_retryable()
        );

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

    #[test]
    fn test_error_400_bad_request_parsing() {
        let json_body = r#"{"error":{"type":"invalid_request_error","message":"Missing required field: model"}}"#;
        let headers = http::HeaderMap::new();

        let error = Error::from_response(400, json_body, &headers);
        match error {
            Error::BadRequest {
                message,
                error_type,
            } => {
                assert_eq!(message, "Missing required field: model");
                assert_eq!(error_type, Some("invalid_request_error".to_string()));
            }
            _ => panic!("Expected BadRequest variant"),
        }
    }

    #[test]
    fn test_error_401_authentication() {
        let json_body = r#"{"error":{"type":"authentication_error","message":"Invalid API key"}}"#;
        let headers = http::HeaderMap::new();

        let error = Error::from_response(401, json_body, &headers);
        match error {
            Error::Authentication(msg) => {
                assert_eq!(msg, "Invalid API key");
            }
            _ => panic!("Expected Authentication variant"),
        }
    }

    #[test]
    fn test_error_403_permission_denied() {
        let json_body =
            r#"{"error":{"type":"permission_error","message":"Access denied to this resource"}}"#;
        let headers = http::HeaderMap::new();

        let error = Error::from_response(403, json_body, &headers);
        match error {
            Error::PermissionDenied(msg) => {
                assert_eq!(msg, "Access denied to this resource");
            }
            _ => panic!("Expected PermissionDenied variant"),
        }
    }

    #[test]
    fn test_error_404_not_found() {
        let json_body = r#"{"error":{"type":"not_found_error","message":"Message not found"}}"#;
        let headers = http::HeaderMap::new();

        let error = Error::from_response(404, json_body, &headers);
        match error {
            Error::NotFound(msg) => {
                assert_eq!(msg, "Message not found");
            }
            _ => panic!("Expected NotFound variant"),
        }
    }

    #[test]
    fn test_error_422_validation_errors() {
        let json_body = r#"{"error":{"type":"invalid_request_error","message":"Validation error","details":{"validation_errors":[{"field":"max_tokens","message":"must be at least 1","code":"too_small"}]}}}"#;
        let headers = http::HeaderMap::new();

        let error = Error::from_response(422, json_body, &headers);
        match error {
            Error::UnprocessableEntity { message, errors } => {
                assert_eq!(message, "Validation error");
                assert!(errors.is_some());
                if let Some(errs) = errors {
                    assert_eq!(errs.len(), 1);
                    assert_eq!(errs[0].field, "max_tokens");
                    assert_eq!(errs[0].message, "must be at least 1");
                    assert_eq!(errs[0].code, Some("too_small".to_string()));
                }
            }
            _ => panic!("Expected UnprocessableEntity variant"),
        }
    }

    #[test]
    fn test_error_429_rate_limit_headers() {
        let json_body = r#"{"error":{"type":"rate_limit_error","message":"Rate limit exceeded"}}"#;

        let mut headers = http::HeaderMap::new();
        headers.insert("retry-after", "60".parse().unwrap());
        headers.insert("anthropic-ratelimit-limit", "100".parse().unwrap());
        headers.insert("anthropic-ratelimit-remaining", "0".parse().unwrap());
        headers.insert(
            "anthropic-ratelimit-reset",
            "2025-10-23T20:00:00Z".parse().unwrap(),
        );

        let error = Error::from_response(429, json_body, &headers);
        match error {
            Error::RateLimit {
                retry_after,
                limit,
                remaining,
                reset_at,
            } => {
                assert_eq!(retry_after, Some(Duration::from_secs(60)));
                assert_eq!(limit, Some(100));
                assert_eq!(remaining, Some(0));
                assert!(reset_at.is_some());
            }
            _ => panic!("Expected RateLimit variant"),
        }
    }

    #[test]
    fn test_error_500_internal_server() {
        let json_body =
            r#"{"error":{"type":"internal_server_error","message":"Internal server error"}}"#;
        let headers = http::HeaderMap::new();

        let error = Error::from_response(500, json_body, &headers);
        match error {
            Error::InternalServerError(msg) => {
                assert_eq!(msg, "Internal server error");
            }
            _ => panic!("Expected InternalServerError variant"),
        }
    }

    #[test]
    fn test_error_529_overloaded() {
        let json_body = r#"{"error":{"type":"overloaded_error","message":"Service overloaded"}}"#;
        let headers = http::HeaderMap::new();

        let error = Error::from_response(529, json_body, &headers);
        match error {
            Error::Overloaded(msg) => {
                assert_eq!(msg, "Service overloaded");
            }
            _ => panic!("Expected Overloaded variant"),
        }
    }

    #[test]
    fn test_error_invalid_json_fallback() {
        let plain_text_body = "Internal Server Error";
        let mut headers = http::HeaderMap::new();
        headers.insert("x-request-id", "req_123".parse().unwrap());

        let error = Error::from_response(500, plain_text_body, &headers);
        match error {
            Error::InternalServerError(msg) => {
                assert_eq!(msg, "Internal Server Error");
            }
            _ => panic!("Expected InternalServerError variant (fallback)"),
        }
    }

    #[test]
    fn test_parse_header_u32_valid() {
        let mut headers = http::HeaderMap::new();
        headers.insert("x-rate-limit", "100".parse().unwrap());

        let result = parse_header_u32(&headers, "x-rate-limit");
        assert_eq!(result, Some(100));
    }

    #[test]
    fn test_parse_header_u32_missing() {
        let headers = http::HeaderMap::new();

        let result = parse_header_u32(&headers, "x-missing-header");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_header_u32_invalid() {
        let mut headers = http::HeaderMap::new();
        headers.insert("x-invalid", "not-a-number".parse().unwrap());

        let result = parse_header_u32(&headers, "x-invalid");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_header_datetime_rfc3339() {
        use chrono::Datelike;

        let mut headers = http::HeaderMap::new();
        headers.insert("x-reset-at", "2025-10-23T20:30:00Z".parse().unwrap());

        let result = parse_header_datetime(&headers, "x-reset-at");
        assert!(result.is_some());

        let dt = result.unwrap();
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 10);
        assert_eq!(dt.day(), 23);
    }

    #[test]
    fn test_parse_header_datetime_missing() {
        let headers = http::HeaderMap::new();

        let result = parse_header_datetime(&headers, "x-missing-datetime");
        assert_eq!(result, None);
    }

    #[test]
    fn test_error_with_context_chain() {
        let original_error = Error::NotFound("message".to_string());
        let with_context = original_error
            .context("Failed to fetch")
            .context("Operation failed");

        let error_string = with_context.to_string();
        assert!(error_string.contains("Operation failed"));
    }
}
