//! Tests for response wrapper modes (raw response with headers)
//!
//! These tests ensure COMPLETE FEATURE PARITY with Python SDK's with_raw_response functionality.
//!
//! Python SDK reference: tests/test_client.py and tests/test_response.py
//! Key features tested:
//! - response.parsed() access to parsed body
//! - response.status_code() for HTTP status
//! - response.headers() for response headers
//! - response.request_id() helper
//! - response.rate_limit_info() helper
//! - response.retries_taken() for retry count
//! - response.elapsed() for timing
//! - Works with all endpoints (messages, batches, count_tokens)

use turboclaude::{Client, MessageRequest, Message, Role};
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

mod common;
use common::*;

#[tokio::test]
async fn test_messages_with_raw_response_basic() {
    let mock_server = MockServer::start().await;

    let mock_response = ResponseTemplate::new(200)
        .append_header("request-id", "req_123456")
        .append_header("anthropic-ratelimit-requests-limit", "100")
        .append_header("anthropic-ratelimit-requests-remaining", "95")
        .append_header("anthropic-ratelimit-requests-reset", "2025-01-20T00:00:00Z")
        .set_body_json(message_response());

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    // Test with_raw_response() returns RawResponse
    let raw = client.messages()
        .with_raw_response()
        .create(request)
        .await
        .unwrap();

    // Python SDK: response.status_code
    assert_eq!(raw.status_code(), 200);

    // Python SDK: response.headers
    assert!(raw.headers().get("request-id").is_some());

    // Python SDK: accessing parsed data directly
    assert_eq!(raw.parsed().id, "msg_01XFDUDYJgAACzvnptvVoYEL");
    assert_eq!(raw.parsed().role, Role::Assistant);

    // Python SDK: response helper methods
    assert_eq!(raw.request_id(), Some("req_123456".to_string()));

    // Python SDK: rate limit helpers
    let (limit, remaining, reset) = raw.rate_limit_info().unwrap();
    assert_eq!(limit, 100);
    assert_eq!(remaining, 95);
    assert_eq!(reset, "2025-01-20T00:00:00Z");

    // Python SDK: retries_taken property
    assert_eq!(raw.retries_taken(), 0);

    // Python SDK: elapsed property (duration is always valid)
    let _ = raw.elapsed(); // Just verify it exists and can be called

    // Python SDK: consuming response
    let message = raw.into_parsed();
    assert_eq!(message.id, "msg_01XFDUDYJgAACzvnptvVoYEL");
}

#[tokio::test]
async fn test_raw_response_status_code_property() {
    // Python SDK test: response.status_code property
    let mock_server = MockServer::start().await;

    let mock_response = ResponseTemplate::new(201)
        .set_body_json(message_response());

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![Message::user("test")])
        .build()
        .unwrap();

    let raw = client.messages()
        .with_raw_response()
        .create(request)
        .await
        .unwrap();

    assert_eq!(raw.status_code(), 201);
}

#[tokio::test]
async fn test_raw_response_headers_property() {
    // Python SDK test: response.headers property
    let mock_server = MockServer::start().await;

    let mock_response = ResponseTemplate::new(200)
        .append_header("x-custom-header", "custom-value")
        .append_header("content-type", "application/json")
        .set_body_json(message_response());

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![Message::user("test")])
        .build()
        .unwrap();

    let raw = client.messages()
        .with_raw_response()
        .create(request)
        .await
        .unwrap();

    let headers = raw.headers();
    assert_eq!(headers.get("x-custom-header").unwrap(), "custom-value");
    assert!(headers.get("content-type").is_some());
}

#[tokio::test]
async fn test_count_tokens_with_raw_response() {
    // Python SDK test: with_raw_response works on all endpoints
    let mock_server = MockServer::start().await;

    let mock_response = ResponseTemplate::new(200)
        .append_header("request-id", "req_token_count")
        .set_body_json(token_count_response());

    Mock::given(method("POST"))
        .and(path("/v1/messages/count_tokens"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    let raw = client.messages()
        .with_raw_response()
        .count_tokens(request)
        .await
        .unwrap();

    assert_eq!(raw.parsed().input_tokens, 15);
    assert_eq!(raw.request_id(), Some("req_token_count".to_string()));
    assert_eq!(raw.status_code(), 200);
}

#[tokio::test]
async fn test_batch_create_with_raw_response() {
    // Python SDK test: batches.with_raw_response.create()
    let mock_server = MockServer::start().await;

    let mock_response = ResponseTemplate::new(200)
        .append_header("request-id", "req_batch_create")
        .append_header("anthropic-ratelimit-requests-limit", "50")
        .append_header("anthropic-ratelimit-requests-remaining", "49")
        .append_header("anthropic-ratelimit-requests-reset", "2025-01-20T00:00:00Z")
        .set_body_json(batch_create_response());

    Mock::given(method("POST"))
        .and(path("/v1/messages/batches"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let batch_request = turboclaude::BatchRequest {
        custom_id: "req-1".to_string(),
        params: MessageRequest::builder()
            .model("claude-3-5-sonnet-20241022")
            .max_tokens(1024u32)
            .messages(vec![Message::user("Hello")])
            .build()
            .unwrap(),
    };

    let raw = client.messages()
        .with_raw_response()
        .batches()
        .create(vec![batch_request])
        .await
        .unwrap();

    assert_eq!(raw.parsed().id, "msgbatch_01234567890");
    assert_eq!(raw.request_id(), Some("req_batch_create".to_string()));
    assert_eq!(raw.status_code(), 200);

    let (limit, remaining, _) = raw.rate_limit_info().unwrap();
    assert_eq!(limit, 50);
    assert_eq!(remaining, 49);
}

#[tokio::test]
async fn test_batch_get_with_raw_response() {
    // Python SDK test: batches.with_raw_response.retrieve()
    let mock_server = MockServer::start().await;

    let mock_response = ResponseTemplate::new(200)
        .append_header("request-id", "req_batch_get")
        .set_body_json(batch_completed_response());

    Mock::given(method("GET"))
        .and(path("/v1/messages/batches/batch_123"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let raw = client.messages()
        .with_raw_response()
        .batches()
        .get("batch_123")
        .await
        .unwrap();

    assert_eq!(raw.parsed().id, "msgbatch_01234567890");
    assert_eq!(raw.request_id(), Some("req_batch_get".to_string()));
}

#[tokio::test]
async fn test_batch_cancel_with_raw_response() {
    // Python SDK test: batches.with_raw_response.cancel()
    let mock_server = MockServer::start().await;

    let mock_response = ResponseTemplate::new(200)
        .append_header("request-id", "req_batch_cancel")
        .set_body_json(batch_completed_response());

    Mock::given(method("POST"))
        .and(path("/v1/messages/batches/batch_123/cancel"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let raw = client.messages()
        .with_raw_response()
        .batches()
        .cancel("batch_123")
        .await
        .unwrap();

    assert_eq!(raw.parsed().id, "msgbatch_01234567890");
    assert_eq!(raw.request_id(), Some("req_batch_cancel".to_string()));
}

#[tokio::test]
async fn test_raw_response_missing_headers() {
    // Python SDK test: missing headers return None
    let mock_server = MockServer::start().await;

    let mock_response = ResponseTemplate::new(200)
        .set_body_json(message_response());

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    let raw = client.messages()
        .with_raw_response()
        .create(request)
        .await
        .unwrap();

    // Missing headers should return None
    assert_eq!(raw.request_id(), None);
    assert_eq!(raw.rate_limit_info(), None);
}

#[tokio::test]
async fn test_raw_response_partial_rate_limit_headers() {
    // Python SDK test: partial headers should return None for rate_limit_info
    let mock_server = MockServer::start().await;

    let mock_response = ResponseTemplate::new(200)
        .append_header("anthropic-ratelimit-requests-limit", "100")
        // Missing remaining and reset headers
        .set_body_json(message_response());

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    let raw = client.messages()
        .with_raw_response()
        .create(request)
        .await
        .unwrap();

    // Should return None if any rate limit header is missing
    assert_eq!(raw.rate_limit_info(), None);
}

#[tokio::test]
async fn test_raw_response_clone() {
    // Python SDK doesn't test cloning, but Rust should support it
    let mock_server = MockServer::start().await;

    let mock_response = ResponseTemplate::new(200)
        .append_header("request-id", "req_clone")
        .set_body_json(message_response());

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    let raw = client.messages()
        .with_raw_response()
        .create(request)
        .await
        .unwrap();

    // RawResponse should be cloneable
    let cloned = raw.clone();
    assert_eq!(cloned.request_id(), raw.request_id());
    assert_eq!(cloned.parsed().id, raw.parsed().id);
    assert_eq!(cloned.status_code(), raw.status_code());
}

#[tokio::test]
async fn test_raw_response_retries_taken() {
    // Python SDK test: retries_taken property
    let mock_server = MockServer::start().await;

    // First response is a server error, second is success
    let error_response = ResponseTemplate::new(500);
    let success_response = ResponseTemplate::new(200)
        .set_body_json(message_response());

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(error_response)
        .in_sequence(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(success_response)
        .in_sequence(2)
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .max_retries(2)
        .build()
        .unwrap();

    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    let raw = client.messages()
        .with_raw_response()
        .create(request)
        .await
        .unwrap();

    // Should have been retried once
    assert_eq!(raw.retries_taken(), 1);
}

#[tokio::test]
async fn test_raw_response_elapsed() {
    // Python SDK test: elapsed property
    let mock_server = MockServer::start().await;

    let mock_response = ResponseTemplate::new(200)
        .set_body_json(message_response());

    Mock::given(method("POST"))
        .and(path("/v/messages"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    let raw = client.messages()
        .with_raw_response()
        .create(request)
        .await
        .unwrap();

    // Elapsed time should be greater than zero
    assert!(raw.elapsed() > std::time::Duration::from_secs(0));
}

#[tokio::test]
async fn test_standard_vs_raw_response_same_data() {
    // Test that with_raw_response and standard mode return same parsed data
    let mock_server = MockServer::start().await;

    let mock_response = ResponseTemplate::new(200)
        .set_body_json(message_response());

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(mock_response.clone())
        .expect(2) // Will be called twice
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let request1 = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    let request2 = request1.clone();

    // Standard response
    let standard = client.messages()
        .create(request1)
        .await
        .unwrap();

    // Raw response
    let raw = client.messages()
        .with_raw_response()
        .create(request2)
        .await
        .unwrap();

    // Both should have same parsed data
    assert_eq!(standard.id, raw.parsed().id);
    assert_eq!(standard.role, raw.parsed().role);
    assert_eq!(standard.model, raw.parsed().model);
}

#[tokio::test]
async fn test_models_list_with_raw_response() {
    // Python SDK test: models.with_raw_response.list()
    let mock_server = MockServer::start().await;

    let mock_response = ResponseTemplate::new(200)
        .append_header("request-id", "req_models_list")
        .set_body_json(model_list_response());

    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let raw = client.models()
        .with_raw_response()
        .list()
        .await
        .unwrap();

    assert_eq!(raw.status_code(), 200);
    assert_eq!(raw.request_id(), Some("req_models_list".to_string()));
    assert_eq!(raw.parsed().len(), 2);
    assert_eq!(raw.parsed()[0].id, "claude-3-5-sonnet-20241022");
}

#[tokio::test]
async fn test_models_get_with_raw_response() {
    // Python SDK test: models.with_raw_response.get()
    let mock_server = MockServer::start().await;

    let mock_response = ResponseTemplate::new(200)
        .append_header("request-id", "req_models_get")
        .set_body_json(model_get_response());

    Mock::given(method("GET"))
        .and(path("/v1/models/claude-3-5-sonnet-20241022"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let raw = client.models()
        .with_raw_response()
        .get("claude-3-5-sonnet-20241022")
        .await
        .unwrap();

    assert_eq!(raw.status_code(), 200);
    assert_eq!(raw.request_id(), Some("req_models_get".to_string()));
    assert_eq!(raw.parsed().id, "claude-3-5-sonnet-20241022");
    assert_eq!(raw.parsed().display_name, "Claude 3.5 Sonnet");
}
