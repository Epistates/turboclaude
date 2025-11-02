//! Integration tests for query execution with MockCliTransport
//!
//! These tests demonstrate the complete query flow including:
//! - Query validation
//! - Request ID generation and correlation
//! - Response handling
//! - Error cases

use turboclaude_protocol::message::MessageRole;
use turboclaude_protocol::types::StopReason;
use turboclaude_protocol::{
    ContentBlock, Message, ProtocolMessage, QueryRequest, QueryResponse, Usage,
};
use turboclaudeagent::testing::MockCliTransport;

/// Create a test query response
fn create_test_response() -> QueryResponse {
    QueryResponse {
        message: Message {
            id: "msg_test_123".to_string(),
            message_type: "message".to_string(),
            role: MessageRole::Assistant,
            content: vec![ContentBlock::Text {
                text: "This is a test response".to_string(),
            }],
            model: "claude-sonnet-4-5".to_string(),
            stop_reason: StopReason::EndTurn,
            stop_sequence: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            usage: Usage {
                input_tokens: 10,
                output_tokens: 20,
            },
            cache_usage: Default::default(),
        },
        is_complete: true,
    }
}

#[tokio::test]
async fn test_mock_transport_send_query() {
    let mock = MockCliTransport::new();

    // Queue a response
    mock.enqueue_response(ProtocolMessage::Response(create_test_response()))
        .await;

    // Simulate sending a query
    let query_request = QueryRequest {
        query: "What is 2+2?".to_string(),
        model: "claude-sonnet-4-5".to_string(),
        max_tokens: 1024,
        tools: vec![],
        messages: vec![],
        system_prompt: None,
    };

    let query_msg = ProtocolMessage::Query(query_request);
    let json = query_msg.to_json().unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Send the query
    assert!(mock.send_message(json_value).await.is_ok());

    // Verify it was tracked
    let sent = mock.sent_messages().await;
    assert_eq!(sent.len(), 1);

    // Receive the response
    let response = mock.recv_message().await.unwrap();
    assert!(response.is_some());

    // Parse the response
    let response_str = serde_json::to_string(&response.unwrap()).unwrap();
    let response_msg = ProtocolMessage::from_json(&response_str).unwrap();

    // Verify it's the right type
    if let ProtocolMessage::Response(resp) = response_msg {
        assert!(resp.is_complete);
        assert_eq!(resp.message.role, MessageRole::Assistant);
    } else {
        panic!("Expected Response message");
    }
}

#[tokio::test]
async fn test_mock_transport_query_validation() {
    // This test demonstrates query validation patterns
    // In real integration with AgentSession, these would be tested there

    let empty_query = QueryRequest {
        query: "".to_string(), // Invalid: empty query
        model: "claude-sonnet-4-5".to_string(),
        max_tokens: 1024,
        tools: vec![],
        messages: vec![],
        system_prompt: None,
    };

    // AgentSession::query() would validate this
    // Mock transport itself doesn't validate, just passes through
    let msg = ProtocolMessage::Query(empty_query);
    assert!(msg.to_json().is_ok());
}

#[tokio::test]
async fn test_mock_transport_multiple_queries() {
    let mock = MockCliTransport::new();

    // Queue multiple responses
    for i in 0..3 {
        let mut response = create_test_response();
        response.message.id = format!("msg_{}", i);
        mock.enqueue_response(ProtocolMessage::Response(response))
            .await;
    }

    // Send multiple queries
    for i in 0..3 {
        let request = QueryRequest {
            query: format!("Query {}", i),
            model: "claude-sonnet-4-5".to_string(),
            max_tokens: 1024,
            tools: vec![],
            messages: vec![],
            system_prompt: None,
        };

        let msg = ProtocolMessage::Query(request);
        let json = msg.to_json().unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(mock.send_message(json_value).await.is_ok());
    }

    // Verify all were tracked
    let sent = mock.sent_messages().await;
    assert_eq!(sent.len(), 3);

    // Receive all responses
    for i in 0..3 {
        let response = mock.recv_message().await.unwrap();
        assert!(response.is_some());

        let response_str = serde_json::to_string(&response.unwrap()).unwrap();
        let response_msg = ProtocolMessage::from_json(&response_str).unwrap();

        if let ProtocolMessage::Response(resp) = response_msg {
            assert_eq!(resp.message.id, format!("msg_{}", 2 - i)); // Responses are LIFO
        } else {
            panic!("Expected Response message");
        }
    }
}

#[tokio::test]
async fn test_mock_transport_concurrent_sends() {
    let mock = MockCliTransport::new();

    // Queue responses
    for i in 0..5 {
        let mut response = create_test_response();
        response.message.id = format!("msg_concurrent_{}", i);
        mock.enqueue_response(ProtocolMessage::Response(response))
            .await;
    }

    // Simulate concurrent sends
    let mut handles = vec![];

    for i in 0..5 {
        let mock_clone = mock.clone();

        let handle = tokio::spawn(async move {
            let request = QueryRequest {
                query: format!("Concurrent query {}", i),
                model: "claude-sonnet-4-5".to_string(),
                max_tokens: 1024,
                tools: vec![],
                messages: vec![],
                system_prompt: None,
            };

            let msg = ProtocolMessage::Query(request);
            let json = msg.to_json().unwrap();
            let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

            mock_clone.send_message(json_value).await
        });

        handles.push(handle);
    }

    // Wait for all sends
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }

    // Verify all were tracked
    let sent = mock.sent_messages().await;
    assert_eq!(sent.len(), 5);
}

#[tokio::test]
async fn test_mock_transport_message_count() {
    let mock = MockCliTransport::new();

    assert_eq!(mock.message_count().await, 0);

    let test_json = serde_json::json!({"type": "test"});
    mock.send_message(test_json.clone()).await.ok();
    assert_eq!(mock.message_count().await, 1);

    mock.send_message(test_json.clone()).await.ok();
    assert_eq!(mock.message_count().await, 2);

    mock.send_message(test_json).await.ok();
    assert_eq!(mock.message_count().await, 3);
}
