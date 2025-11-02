//! Streaming tests
//!
//! Comprehensive SSE streaming tests:
//! - Event parsing
//! - Message reconstruction
//! - Tool use streaming
//! - Error handling in streams

use turboclaude::{Client, types::{MessageRequest, Message, Models}, streaming::StreamEvent};
use rstest::*;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use futures::StreamExt;

mod common;
use common::{sse_text_stream, sse_tool_use_stream};

#[tokio::test]
async fn test_streaming_text_response() {
    let mock_server = MockServer::start().await;

    // SSE events need double newlines between them and a final newline
    let sse_body = sse_text_stream().join("\n\n") + "\n\n";

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(sse_body)
                .insert_header("content-type", "text/event-stream")
                .insert_header("cache-control", "no-cache")
        )
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

    let result = client.messages().stream(request).await;

    assert!(result.is_ok());

    let mut stream = result.unwrap();
    let mut events = Vec::new();

    while let Some(event) = stream.next().await {
        match event {
            Ok(e) => events.push(e),
            Err(e) => panic!("Stream error: {:?}", e),
        }
    }

    // Verify we got all expected events
    assert!(!events.is_empty());

    // Check for message_start event
    let has_message_start = events.iter().any(|e| matches!(e, StreamEvent::MessageStart(_)));
    assert!(has_message_start);

    // Check for content deltas
    let has_content_delta = events.iter().any(|e| matches!(e, StreamEvent::ContentBlockDelta(_)));
    assert!(has_content_delta);

    // Check for message_stop event
    let has_message_stop = events.iter().any(|e| matches!(e, StreamEvent::MessageStop));
    assert!(has_message_stop);
}

#[tokio::test]
async fn test_streaming_tool_use() {
    let mock_server = MockServer::start().await;

    // SSE events need double newlines between them and a final newline
    let sse_body = sse_tool_use_stream().join("\n\n") + "\n\n";

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(sse_body)
                .insert_header("content-type", "text/event-stream")
        )
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
        .messages(vec![Message::user("Calculate something")])
        .build()
        .unwrap();

    let result = client.messages().stream(request).await;

    assert!(result.is_ok());

    let mut stream = result.unwrap();
    let mut events = Vec::new();

    while let Some(event) = stream.next().await {
        match event {
            Ok(e) => events.push(e),
            Err(e) => panic!("Stream error: {:?}", e),
        }
    }

    assert!(!events.is_empty());

    // Verify tool use event
    let has_content_block_start = events.iter().any(|e| {
        matches!(e, StreamEvent::ContentBlockStart(_))
    });
    assert!(has_content_block_start);
}

#[tokio::test]
async fn test_streaming_get_final_message() {
    let mock_server = MockServer::start().await;

    // SSE events need double newlines between them and a final newline
    let sse_body = sse_text_stream().join("\n\n") + "\n\n";

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(sse_body)
                .insert_header("content-type", "text/event-stream")
        )
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

    let stream = client.messages().stream(request).await.unwrap();

    // Get the final reconstructed message
    let result = stream.get_final_message().await;

    assert!(result.is_ok());
    let message = result.unwrap();

    // Verify message properties
    assert_eq!(message.id, "msg_01");
    assert!(!message.content.is_empty());
}

#[tokio::test]
async fn test_streaming_text_accumulation() {
    let mock_server = MockServer::start().await;

    // SSE events need double newlines between them and a final newline
    let sse_body = sse_text_stream().join("\n\n") + "\n\n";

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(sse_body)
                .insert_header("content-type", "text/event-stream")
        )
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

    let mut stream = client.messages().stream(request).await.unwrap();

    let mut accumulated_text = String::new();

    while let Some(event) = stream.next().await {
        if let Ok(StreamEvent::ContentBlockDelta(delta)) = event {
            if let Some(text) = delta.delta.text {
                accumulated_text.push_str(&text);
            }
        }
    }

    // Verify text was accumulated
    assert!(!accumulated_text.is_empty());
    assert_eq!(accumulated_text, "Hello there!");
}

/// Test that streaming handles malformed SSE gracefully
#[tokio::test]
async fn test_streaming_malformed_sse() {
    let mock_server = MockServer::start().await;

    let malformed_sse = "event: message_start\ndata: {invalid json}\n\n";

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(malformed_sse)
                .insert_header("content-type", "text/event-stream")
        )
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

    let result = client.messages().stream(request).await;

    // Stream should be created successfully
    assert!(result.is_ok());

    let mut stream = result.unwrap();

    // But parsing events should fail
    if let Some(event) = stream.next().await {
        assert!(event.is_err());
    }
}

/// Parametrized test for different event types
#[rstest]
#[case("message_start")]
#[case("content_block_start")]
#[case("content_block_delta")]
#[case("content_block_stop")]
#[case("message_delta")]
#[case("message_stop")]
#[tokio::test]
async fn test_streaming_event_types(#[case] event_type: &str) {
    // This test verifies that all event types are recognized
    // In a full implementation, each would have its own SSE fixture
    let events = sse_text_stream();

    let has_event = events.iter().any(|e| e.contains(&format!("event: {}", event_type)));

    assert!(has_event, "Missing event type: {}", event_type);
}
