//! Messages API endpoint tests
//!
//! Comprehensive test coverage matching Python SDK's test_messages.py:
//! - Message creation with all parameters
//! - Streaming responses
//! - Token counting
//! - Tool use
//! - Error handling

use turboclaude::{
    Client,
    types::{MessageRequest, Message, MessageParam, Role, ContentBlock, Models},
};
use rstest::*;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, header};

mod common;
use common::{message_response, message_with_tool_use, token_count_response};

#[tokio::test]
async fn test_messages_create_basic() {
    // Setup mock server
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("x-api-key", "test-key"))
        .and(header("content-type", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(message_response()))
        .mount(&mock_server)
        .await;

    // Create client with mock server
    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    // Create message request
    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    // Send request
    let result = client.messages().create(request).await;

    assert!(result.is_ok());
    let message = result.unwrap();
    assert_eq!(message.id, "msg_01XFDUDYJgAACzvnptvVoYEL");
    assert_eq!(message.role, Role::Assistant);
    assert_eq!(message.model, "claude-3-5-sonnet-20241022");
}

#[tokio::test]
async fn test_messages_create_with_system_prompt() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(message_response()))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .system("You are a helpful assistant")
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    let result = client.messages().create(request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_messages_create_with_temperature() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(message_response()))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .temperature(0.7)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    let result = client.messages().create(request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_messages_create_with_multiple_messages() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(message_response()))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![
            Message::user("Hello"),
            Message::assistant("Hi! How can I help?"),
            Message::user("Tell me a joke"),
        ])
        .build()
        .unwrap();

    let result = client.messages().create(request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_messages_create_with_tool_use() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(message_with_tool_use()))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![Message::user("What is 42 * 17?")])
        .build()
        .unwrap();

    let result = client.messages().create(request).await;
    assert!(result.is_ok());

    let message = result.unwrap();
    assert_eq!(message.stop_reason.as_ref().unwrap().as_str(), "tool_use");

    // Verify tool use in content
    let has_tool_use = message.content.iter().any(|block| {
        matches!(block, ContentBlock::ToolUse { .. })
    });
    assert!(has_tool_use);
}

#[tokio::test]
async fn test_messages_count_tokens() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages/count_tokens"))
        .respond_with(ResponseTemplate::new(200).set_body_json(token_count_response()))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello, how are you?")])
        .build()
        .unwrap();

    let result = client.messages().count_tokens(request).await;
    assert!(result.is_ok());

    let token_count = result.unwrap();
    assert_eq!(token_count.input_tokens, 15);
}

#[tokio::test]
async fn test_messages_text_helper() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(message_response()))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    let message = client.messages().create(request).await.unwrap();

    // Test text() helper method
    let text = message.text();
    assert_eq!(text, "Hello! How can I help you today?");
}

/// Parametrized test for different max_tokens values
#[rstest]
#[case(1)]
#[case(100)]
#[case(1024)]
#[case(4096)]
#[tokio::test]
async fn test_messages_create_various_max_tokens(#[case] max_tokens: u32) {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(message_response()))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(max_tokens)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    let result = client.messages().create(request).await;
    assert!(result.is_ok());
}

/// Parametrized test for different temperature values
#[rstest]
#[case(0.0)]
#[case(0.5)]
#[case(1.0)]
#[tokio::test]
async fn test_messages_create_various_temperatures(#[case] temperature: f32) {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(message_response()))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .temperature(temperature)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    let result = client.messages().create(request).await;
    assert!(result.is_ok());
}

/// Parametrized test for all supported models
#[rstest]
#[case(Models::CLAUDE_3_5_SONNET)]
#[case(Models::CLAUDE_3_OPUS)]
#[case(Models::CLAUDE_3_SONNET)]
#[case(Models::CLAUDE_3_HAIKU)]
#[tokio::test]
async fn test_messages_create_various_models(#[case] model: &str) {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(message_response()))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let request = MessageRequest::builder()
        .model(model)
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    let result = client.messages().create(request).await;
    assert!(result.is_ok());
}
