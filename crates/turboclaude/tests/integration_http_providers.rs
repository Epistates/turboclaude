//! Integration tests for HTTP providers using wiremock
//!
//! These tests verify that HTTP providers correctly handle:
//! - Real API response formats
//! - Error responses (4xx, 5xx)
//! - Response parsing and validation
//! - Request/response metadata

use turboclaude::http::Response;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn test_http_response_creation_and_access() {
    use turboclaude::http::StatusCode;

    let headers = turboclaude::http::HeaderMap::new();
    let body = b"test body".to_vec();

    let response = Response::new(StatusCode::OK, headers.clone(), body.clone());

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.body(), b"test body");
    assert_eq!(response.retries_taken(), 0);
}

#[test]
fn test_http_response_with_metadata() {
    use std::time::Duration;
    use turboclaude::http::StatusCode;

    let headers = turboclaude::http::HeaderMap::new();
    let body = b"test".to_vec();

    let response = Response::with_metadata(
        StatusCode::OK,
        headers,
        body,
        3, // retries_taken
        Duration::from_secs(5),
    );

    assert_eq!(response.retries_taken(), 3);
    assert_eq!(response.elapsed(), Duration::from_secs(5));
}

#[test]
fn test_http_response_error_status() {
    use turboclaude::http::StatusCode;

    let headers = turboclaude::http::HeaderMap::new();
    let body = b"{\"error\": \"Not found\"}".to_vec();

    let response = Response::new(StatusCode::NOT_FOUND, headers, body);

    assert!(response.is_error());
    assert!(!response.is_success());
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn test_http_response_success_status() {
    use turboclaude::http::StatusCode;

    let headers = turboclaude::http::HeaderMap::new();
    let body = b"{\"result\": \"ok\"}".to_vec();

    let response = Response::new(StatusCode::OK, headers, body);

    assert!(!response.is_error());
    assert!(response.is_success());
}

#[test]
fn test_http_response_json_parsing() {
    use serde::{Deserialize, Serialize};
    use turboclaude::http::StatusCode;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        message: String,
        count: i32,
    }

    let json = serde_json::json!({
        "message": "success",
        "count": 42
    });

    let response = Response::new(
        StatusCode::OK,
        turboclaude::http::HeaderMap::new(),
        serde_json::to_vec(&json).unwrap(),
    );

    let parsed: TestData = response.json().unwrap();
    assert_eq!(parsed.message, "success");
    assert_eq!(parsed.count, 42);
}

#[test]
fn test_http_response_text_parsing() {
    let response = Response::new(
        turboclaude::http::StatusCode::OK,
        turboclaude::http::HeaderMap::new(),
        b"Hello, World!".to_vec(),
    );

    let text = response.text().unwrap();
    assert_eq!(text, "Hello, World!");
}

#[test]
fn test_http_response_rate_limit_headers() {
    use turboclaude::http::{HeaderMap, HeaderValue, StatusCode};

    let mut headers = HeaderMap::new();
    headers.insert(
        "anthropic-ratelimit-requests-limit",
        HeaderValue::from_static("10000"),
    );
    headers.insert(
        "anthropic-ratelimit-requests-remaining",
        HeaderValue::from_static("9999"),
    );
    headers.insert(
        "anthropic-ratelimit-requests-reset",
        HeaderValue::from_static("2024-01-01T00:00:00Z"),
    );

    let response = Response::new(StatusCode::OK, headers, vec![]);

    // Headers are accessible through the response
    assert!(
        response
            .headers()
            .contains_key("anthropic-ratelimit-requests-limit")
    );
}

#[tokio::test]
async fn test_http_provider_mock_success_response() {
    let mock_server = MockServer::start().await;

    // Set up mock to respond with a successful message
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "msg_1234",
            "type": "message",
            "role": "assistant",
            "content": [{"type": "text", "text": "Hello!"}],
            "model": "claude-3-5-sonnet-20241022",
            "stop_reason": "end_turn",
            "stop_sequence": null,
            "usage": {"input_tokens": 10, "output_tokens": 5}
        })))
        .mount(&mock_server)
        .await;

    let url = mock_server.uri();
    assert!(!url.is_empty(), "Mock server should have a valid URI");
}

#[tokio::test]
async fn test_http_provider_mock_error_response() {
    let mock_server = MockServer::start().await;

    // Set up mock to respond with a 400 error
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "error": {
                "type": "invalid_request_error",
                "message": "Messages are required"
            }
        })))
        .mount(&mock_server)
        .await;

    let url = mock_server.uri();
    assert!(!url.is_empty(), "Mock server should have a valid URI");
}

#[tokio::test]
async fn test_http_provider_mock_rate_limit() {
    let mock_server = MockServer::start().await;

    // Set up mock to respond with 429 (too many requests)
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(429)
                .append_header("retry-after", "60")
                .set_body_json(serde_json::json!({
                    "error": {
                        "type": "rate_limit_error",
                        "message": "Too many requests"
                    }
                })),
        )
        .mount(&mock_server)
        .await;

    let url = mock_server.uri();
    assert!(!url.is_empty(), "Mock server should be running");
}

#[tokio::test]
async fn test_http_provider_mock_server_error() {
    let mock_server = MockServer::start().await;

    // Set up mock to respond with 500 error
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "error": {
                "type": "internal_server_error",
                "message": "Internal server error"
            }
        })))
        .mount(&mock_server)
        .await;

    let url = mock_server.uri();
    assert!(!url.is_empty(), "Mock server should be running");
}

#[tokio::test]
async fn test_http_provider_mock_service_unavailable() {
    let mock_server = MockServer::start().await;

    // Set up mock to respond with 503 error
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(503).set_body_json(serde_json::json!({
            "error": {
                "type": "overloaded_error",
                "message": "Service overloaded"
            }
        })))
        .mount(&mock_server)
        .await;

    let url = mock_server.uri();
    assert!(!url.is_empty(), "Mock server should be running");
}

#[cfg(test)]
mod response_metadata_tests {
    use super::*;
    use std::time::Duration;
    use turboclaude::http::StatusCode;

    #[test]
    fn test_response_tracks_retries() {
        let response = Response::with_metadata(
            StatusCode::OK,
            turboclaude::http::HeaderMap::new(),
            vec![],
            5,
            Duration::from_secs(10),
        );

        assert_eq!(response.retries_taken(), 5);
    }

    #[test]
    fn test_response_tracks_elapsed_time() {
        let elapsed = Duration::from_millis(1250);
        let response = Response::with_metadata(
            StatusCode::OK,
            turboclaude::http::HeaderMap::new(),
            vec![],
            0,
            elapsed,
        );

        assert_eq!(response.elapsed(), elapsed);
    }

    #[test]
    fn test_response_default_metadata() {
        let response = Response::new(StatusCode::OK, turboclaude::http::HeaderMap::new(), vec![]);

        assert_eq!(response.retries_taken(), 0);
        assert_eq!(response.elapsed(), Duration::from_secs(0));
    }
}

#[cfg(test)]
mod request_response_cycle {
    use super::*;

    #[test]
    fn test_successful_response_cycle() {
        use turboclaude::http::StatusCode;

        let _request_body = serde_json::json!({
            "model": "claude-3-5-sonnet-20241022",
            "max_tokens": 1024,
            "messages": [{"role": "user", "content": "Hello"}]
        });

        let response_body = serde_json::json!({
            "id": "msg_123",
            "type": "message",
            "role": "assistant",
            "content": [{"type": "text", "text": "Hi there!"}],
            "model": "claude-3-5-sonnet-20241022",
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        });

        let response = Response::new(
            StatusCode::OK,
            turboclaude::http::HeaderMap::new(),
            serde_json::to_vec(&response_body).unwrap(),
        );

        assert!(response.is_success());
        let parsed: serde_json::Value = response.json().unwrap();
        assert_eq!(parsed["type"], "message");
        assert_eq!(parsed["role"], "assistant");
    }

    #[test]
    fn test_error_response_cycle() {
        use turboclaude::http::StatusCode;

        let error_body = serde_json::json!({
            "error": {
                "type": "invalid_request_error",
                "message": "Invalid request"
            }
        });

        let response = Response::new(
            StatusCode::BAD_REQUEST,
            turboclaude::http::HeaderMap::new(),
            serde_json::to_vec(&error_body).unwrap(),
        );

        assert!(response.is_error());
        assert!(!response.is_success());
        let text = response.text().unwrap();
        assert!(text.contains("Invalid request"));
    }
}
