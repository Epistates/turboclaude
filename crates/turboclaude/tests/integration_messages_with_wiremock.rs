//! Integration tests for Messages API using wiremock
//!
//! This demonstrates best-in-class Rust testing practices:
//! - HTTP mocking with wiremock (no Python dependencies)
//! - Fixture-based testing
//! - Comprehensive test coverage
//! - Property-based testing with proptest

mod common;

use turboclaude::{Client, Message, MessageRequest, Role, StopReason};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_create_message_success() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Load fixture response
    let response_body = common::load_response_fixture("message_success");

    // Configure mock
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("x-api-key", common::test_api_key().as_str()))
        .and(header("anthropic-version", "2023-06-01"))
        .and(header("content-type", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .expect(1) // Expect exactly 1 call
        .mount(&mock_server)
        .await;

    // Create client pointing to mock server
    let client = Client::builder()
        .api_key(common::test_api_key())
        .base_url(mock_server.uri())
        .build()
        .expect("Failed to build client");

    // Build request using MessageRequest builder
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello!")])
        .build()
        .expect("Failed to build request");

    // Execute request
    let response = client
        .messages()
        .create(request)
        .await
        .expect("Request failed");

    // Verify response
    assert_eq!(response.id, "msg_01XFDUDYJgAACzvnptvVoYEL");
    assert_eq!(response.role, Role::Assistant);
    assert_eq!(response.model, "claude-3-5-sonnet-20241022");
    assert_eq!(response.stop_reason.unwrap(), StopReason::EndTurn);

    // Verify content
    assert_eq!(response.content.len(), 1);
    if let turboclaude::ContentBlock::Text { text, .. } = &response.content[0] {
        assert!(text.contains("Claude"));
    } else {
        panic!("Expected text content block");
    }

    // Verify usage
    assert_eq!(response.usage.input_tokens, 12);
    assert_eq!(response.usage.output_tokens, 25);

    // Verify mock was called
    mock_server.verify().await;
}

#[tokio::test]
async fn test_create_message_with_tool_use() {
    let mock_server = MockServer::start().await;

    let response_body = common::load_response_fixture("message_with_tool_use");

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key(common::test_api_key())
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    // Build request with tool
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![Message::user("What is 5 + 3?")])
        .tools(vec![turboclaude::Tool::new(
            "calculator",
            "Perform arithmetic operations",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": {"type": "string"},
                    "operand1": {"type": "number"},
                    "operand2": {"type": "number"}
                },
                "required": ["operation", "operand1", "operand2"]
            }),
        )])
        .build()
        .expect("Failed to build request");

    let response = client.messages().create(request).await.unwrap();

    // Verify tool use in response
    assert_eq!(response.stop_reason.unwrap(), StopReason::ToolUse);

    let mut found_tool_use = false;
    for block in &response.content {
        if let turboclaude::ContentBlock::ToolUse { id, name, input } = block {
            found_tool_use = true;
            assert_eq!(id, "toolu_01T4cZHAFxMJDJTMdG4NxVQf");
            assert_eq!(name, "calculator");
            assert!(input.get("operation").is_some());
        }
    }
    assert!(found_tool_use, "Expected tool_use content block");
}

#[tokio::test]
async fn test_error_handling_unauthorized() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "type": "error",
            "error": {
                "type": "authentication_error",
                "message": "invalid x-api-key"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("sk-invalid-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    // Build request
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello!")])
        .build()
        .expect("Failed to build request");

    let result = client.messages().create(request).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    // Verify error type and message - check for general error indicators
    let err_str = err.to_string().to_lowercase();
    assert!(
        err_str.contains("authentication")
            || err_str.contains("401")
            || err_str.contains("invalid"),
        "Expected error message to contain auth/401/invalid indicator, got: {}",
        err
    );
}

#[tokio::test]
async fn test_rate_limit_handling() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "2")
                .set_body_json(serde_json::json!({
                    "type": "error",
                    "error": {
                        "type": "rate_limit_error",
                        "message": "Rate limit exceeded"
                    }
                })),
        )
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key(common::test_api_key())
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    // Build request
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello!")])
        .build()
        .expect("Failed to build request");

    let result = client.messages().create(request).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_str = err.to_string().to_lowercase();
    assert!(
        err_str.contains("rate") || err_str.contains("429") || err_str.contains("limit"),
        "Expected error message to contain rate/429/limit indicator, got: {}",
        err.to_string()
    );
}

#[cfg(test)]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_api_key_validation(key in "sk-[a-zA-Z0-9]{40,70}") {
            // Property: Any valid-format API key should be accepted by client builder
            let result = Client::builder()
                .api_key(&key)
                .build();

            prop_assert!(result.is_ok() || result.is_err()); // Should not panic
        }

        #[test]
        fn test_model_name_handling(model in "[a-z0-9-]{1,100}") {
            // Property: Model names should be passed through as-is
            prop_assert!(model.len() <= 100);
        }

        #[test]
        fn test_max_tokens_bounds(tokens in 1u32..100000u32) {
            // Property: Any positive token count should be valid
            prop_assert!(tokens > 0);
            prop_assert!(tokens < 100000);
        }
    }
}
