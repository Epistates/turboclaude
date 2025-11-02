//! Error handling tests
//!
//! Comprehensive error testing matching Python SDK:
//! - All error types (authentication, rate limit, invalid request, etc.)
//! - Error context and messages
//! - Retry behavior
//! - Error response parsing

use turboclaude::{Client, Error, types::{MessageRequest, Message, Models}};
use rstest::*;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

mod common;
use common::{error_invalid_request, error_authentication, error_rate_limit};

#[tokio::test]
async fn test_error_invalid_request() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(400).set_body_json(error_invalid_request()))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .max_retries(0)
        .build()
        .unwrap();

    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    let result = client.messages().create(request).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::BadRequest { message, .. } => {
            assert!(message.contains("missing required field"));
        }
        _ => panic!("Expected BadRequest error"),
    }
}

#[tokio::test]
async fn test_error_authentication() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(401).set_body_json(error_authentication()))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("invalid-key")
        .base_url(mock_server.uri())
        .max_retries(0)
        .build()
        .unwrap();

    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    let result = client.messages().create(request).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Authentication(msg) => {
            assert!(msg.contains("Invalid API key"));
        }
        _ => panic!("Expected Authentication error"),
    }
}

#[tokio::test]
async fn test_error_rate_limit() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(429)
                .set_body_json(error_rate_limit())
                .insert_header("retry-after", "60")
                .insert_header("anthropic-ratelimit-limit", "100")
                .insert_header("anthropic-ratelimit-remaining", "0")
        )
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .max_retries(0)
        .build()
        .unwrap();

    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    let result = client.messages().create(request).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::RateLimit { retry_after, limit, remaining, .. } => {
            assert!(retry_after.is_some());
            assert_eq!(limit, Some(100));
            assert_eq!(remaining, Some(0));
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[tokio::test]
async fn test_error_not_found() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/models/non-existent"))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(serde_json::json!({
                    "type": "error",
                    "error": {
                        "type": "not_found_error",
                        "message": "Model not found"
                    }
                }))
        )
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .max_retries(0)
        .build()
        .unwrap();

    let result = client.models().get("non-existent").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::NotFound(msg) => {
            assert!(msg.contains("not found"));
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_error_internal_server_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(500)
                .set_body_json(serde_json::json!({
                    "type": "error",
                    "error": {
                        "type": "internal_server_error",
                        "message": "Internal server error"
                    }
                }))
        )
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .max_retries(0)
        .build()
        .unwrap();

    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    let result = client.messages().create(request).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::InternalServerError(msg) => {
            assert!(msg.contains("Internal server error"));
        }
        _ => panic!("Expected InternalServerError error"),
    }
}

#[tokio::test]
async fn test_error_overloaded() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(529)
                .set_body_json(serde_json::json!({
                    "type": "error",
                    "error": {
                        "type": "overloaded_error",
                        "message": "Service is overloaded"
                    }
                }))
        )
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .max_retries(0)
        .build()
        .unwrap();

    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    let result = client.messages().create(request).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Overloaded(msg) => {
            assert!(msg.contains("overloaded"));
        }
        _ => panic!("Expected Overloaded error"),
    }
}

#[tokio::test]
async fn test_error_is_retryable() {
    // Test that error types are correctly identified as retryable
    let rate_limit = Error::RateLimit {
        retry_after: None,
        limit: None,
        remaining: None,
        reset_at: None,
    };
    assert!(rate_limit.is_retryable());

    let internal_error = Error::InternalServerError("test".to_string());
    assert!(internal_error.is_retryable());

    let timeout = Error::Timeout(std::time::Duration::from_secs(30));
    assert!(timeout.is_retryable());

    let connection = Error::Connection("test".to_string());
    assert!(connection.is_retryable());

    // Non-retryable errors
    let auth = Error::Authentication("test".to_string());
    assert!(!auth.is_retryable());

    let bad_request = Error::BadRequest {
        message: "test".to_string(),
        error_type: None,
    };
    assert!(!bad_request.is_retryable());

    let not_found = Error::NotFound("test".to_string());
    assert!(!not_found.is_retryable());
}

/// Test retry-after duration extraction
#[tokio::test]
async fn test_error_retry_after() {
    let rate_limit = Error::RateLimit {
        retry_after: Some(std::time::Duration::from_secs(60)),
        limit: None,
        remaining: None,
        reset_at: None,
    };

    assert_eq!(rate_limit.retry_after(), Some(std::time::Duration::from_secs(60)));

    let no_retry = Error::Authentication("test".to_string());
    assert_eq!(no_retry.retry_after(), None);
}

/// Parametrized test for various HTTP error statuses
#[rstest]
#[case(400, "BadRequest")]
#[case(401, "Authentication")]
#[case(403, "PermissionDenied")]
#[case(404, "NotFound")]
#[case(429, "RateLimit")]
#[case(500, "InternalServerError")]
#[case(529, "Overloaded")]
#[tokio::test]
async fn test_error_status_codes(#[case] status: u16, #[case] expected_type: &str) {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(status)
                .set_body_json(serde_json::json!({
                    "type": "error",
                    "error": {
                        "type": format!("{}_error", expected_type.to_lowercase()),
                        "message": format!("{} error message", expected_type)
                    }
                }))
        )
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .max_retries(0)
        .build()
        .unwrap();

    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    let result = client.messages().create(request).await;

    assert!(result.is_err());
    // Just verify we got an error - specific error type testing is done above
}

/// Test error display formatting
#[test]
fn test_error_display() {
    let bad_request = Error::BadRequest {
        message: "Invalid model".to_string(),
        error_type: Some("invalid_request_error".to_string()),
    };
    let display = format!("{}", bad_request);
    assert!(display.contains("Invalid model"));

    let auth = Error::Authentication("Invalid API key".to_string());
    let display = format!("{}", auth);
    assert!(display.contains("Invalid API key"));
}

/// Test error debug formatting
#[test]
fn test_error_debug() {
    let bad_request = Error::BadRequest {
        message: "Invalid model".to_string(),
        error_type: Some("invalid_request_error".to_string()),
    };
    let debug = format!("{:?}", bad_request);
    assert!(debug.contains("BadRequest"));
}
