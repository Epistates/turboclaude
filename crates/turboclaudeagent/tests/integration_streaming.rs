//! Integration tests for streaming response handling
//!
//! Tests for iterating over response content blocks and parsing streaming events

use serde_json::json;
use turboclaude_protocol::message::MessageRole;
use turboclaude_protocol::{
    ContentBlock, Message, ProtocolMessage, QueryRequest, QueryResponse, Usage, types::StopReason,
};
use turboclaudeagent::message_parser::ParsedMessage;
use turboclaudeagent::testing::MockCliTransport;

fn create_test_response(blocks: Vec<ContentBlock>) -> QueryResponse {
    QueryResponse {
        message: Message {
            id: "msg_test_123".to_string(),
            message_type: "message".to_string(),
            role: MessageRole::Assistant,
            content: blocks,
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
async fn test_single_text_content_block() {
    let mock = MockCliTransport::new();

    // Queue response with single text block
    let blocks = vec![ContentBlock::Text {
        text: "This is a test response".to_string(),
    }];
    mock.enqueue_response(ProtocolMessage::Response(create_test_response(blocks)))
        .await;

    // Send query
    let query_request = QueryRequest {
        query: "What is 2+2?".to_string(),
        model: "claude-sonnet-4-5".to_string(),
        max_tokens: 1024,
        tools: vec![],
        messages: vec![],
        system_prompt: None,
    };

    let msg = ProtocolMessage::Query(query_request);
    let json = msg.to_json().unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(mock.send_message(json_value).await.is_ok());

    // Receive response
    let response = mock.recv_message().await.unwrap();
    assert!(response.is_some());

    // Parse and verify content blocks
    let response_str = serde_json::to_string(&response.unwrap()).unwrap();
    let response_msg = ProtocolMessage::from_json(&response_str).unwrap();

    if let ProtocolMessage::Response(resp) = response_msg {
        assert_eq!(resp.message.content.len(), 1);
        match &resp.message.content[0] {
            ContentBlock::Text { text } => {
                assert_eq!(text, "This is a test response");
            }
            _ => panic!("Expected text content"),
        }
    }
}

#[tokio::test]
async fn test_multiple_text_blocks() {
    let mock = MockCliTransport::new();

    // Queue response with multiple text blocks
    let blocks = vec![
        ContentBlock::Text {
            text: "First block".to_string(),
        },
        ContentBlock::Text {
            text: "Second block".to_string(),
        },
        ContentBlock::Text {
            text: "Third block".to_string(),
        },
    ];
    mock.enqueue_response(ProtocolMessage::Response(create_test_response(blocks)))
        .await;

    // Send query
    let query_request = QueryRequest {
        query: "Test".to_string(),
        model: "claude-sonnet-4-5".to_string(),
        max_tokens: 1024,
        tools: vec![],
        messages: vec![],
        system_prompt: None,
    };

    let msg = ProtocolMessage::Query(query_request);
    let json = msg.to_json().unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(mock.send_message(json_value).await.is_ok());

    // Receive response
    let response = mock.recv_message().await.unwrap();
    assert!(response.is_some());

    // Verify all blocks
    let response_str = serde_json::to_string(&response.unwrap()).unwrap();
    let response_msg = ProtocolMessage::from_json(&response_str).unwrap();

    if let ProtocolMessage::Response(resp) = response_msg {
        assert_eq!(resp.message.content.len(), 3);

        // Iterate and verify each block
        let texts: Vec<String> = resp
            .message
            .content
            .iter()
            .filter_map(|block| {
                if let ContentBlock::Text { text } = block {
                    Some(text.clone())
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(texts.len(), 3);
        assert_eq!(texts[0], "First block");
        assert_eq!(texts[1], "Second block");
        assert_eq!(texts[2], "Third block");
    }
}

#[tokio::test]
async fn test_mixed_content_blocks() {
    let mock = MockCliTransport::new();

    // Queue response with mixed content types
    let blocks = vec![
        ContentBlock::Text {
            text: "Text response".to_string(),
        },
        ContentBlock::ToolUse {
            id: "tool_123".to_string(),
            name: "calculator".to_string(),
            input: serde_json::json!({"operation": "add", "a": 2, "b": 2}),
        },
    ];
    mock.enqueue_response(ProtocolMessage::Response(create_test_response(blocks)))
        .await;

    // Send query
    let query_request = QueryRequest {
        query: "Calculate 2+2 using tools".to_string(),
        model: "claude-sonnet-4-5".to_string(),
        max_tokens: 1024,
        tools: vec![],
        messages: vec![],
        system_prompt: None,
    };

    let msg = ProtocolMessage::Query(query_request);
    let json = msg.to_json().unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(mock.send_message(json_value).await.is_ok());

    // Receive response
    let response = mock.recv_message().await.unwrap();
    assert!(response.is_some());

    // Verify mixed content
    let response_str = serde_json::to_string(&response.unwrap()).unwrap();
    let response_msg = ProtocolMessage::from_json(&response_str).unwrap();

    if let ProtocolMessage::Response(resp) = response_msg {
        assert_eq!(resp.message.content.len(), 2);

        // Verify first is text
        match &resp.message.content[0] {
            ContentBlock::Text { text } => {
                assert_eq!(text, "Text response");
            }
            _ => panic!("Expected text content at index 0"),
        }

        // Verify second is tool use
        match &resp.message.content[1] {
            ContentBlock::ToolUse { id, name, input } => {
                assert_eq!(id, "tool_123");
                assert_eq!(name, "calculator");
                assert_eq!(input.get("operation").and_then(|v| v.as_str()), Some("add"));
            }
            _ => panic!("Expected tool use content at index 1"),
        }
    }
}

#[tokio::test]
async fn test_iterate_content_blocks() {
    let mock = MockCliTransport::new();

    // Queue response
    let blocks = vec![
        ContentBlock::Text {
            text: "Block 1".to_string(),
        },
        ContentBlock::Text {
            text: "Block 2".to_string(),
        },
        ContentBlock::Text {
            text: "Block 3".to_string(),
        },
    ];
    mock.enqueue_response(ProtocolMessage::Response(create_test_response(blocks)))
        .await;

    // Send query
    let query_request = QueryRequest {
        query: "Test iteration".to_string(),
        model: "claude-sonnet-4-5".to_string(),
        max_tokens: 1024,
        tools: vec![],
        messages: vec![],
        system_prompt: None,
    };

    let msg = ProtocolMessage::Query(query_request);
    let json = msg.to_json().unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(mock.send_message(json_value).await.is_ok());

    // Receive and iterate blocks
    let response = mock.recv_message().await.unwrap();
    assert!(response.is_some());

    let response_str = serde_json::to_string(&response.unwrap()).unwrap();
    let response_msg = ProtocolMessage::from_json(&response_str).unwrap();

    if let ProtocolMessage::Response(resp) = response_msg {
        // Demonstrate iteration pattern
        let mut block_count = 0;
        for block in resp.message.content.iter() {
            block_count += 1;
            if let ContentBlock::Text { text } = block {
                assert!(!text.is_empty());
            }
        }
        assert_eq!(block_count, 3);
    }
}

#[tokio::test]
async fn test_large_response_content() {
    let mock = MockCliTransport::new();

    // Queue response with large text
    let large_text = "x".repeat(10000);
    let blocks = vec![ContentBlock::Text {
        text: large_text.clone(),
    }];
    mock.enqueue_response(ProtocolMessage::Response(create_test_response(blocks)))
        .await;

    // Send query
    let query_request = QueryRequest {
        query: "Generate large response".to_string(),
        model: "claude-sonnet-4-5".to_string(),
        max_tokens: 4096,
        tools: vec![],
        messages: vec![],
        system_prompt: None,
    };

    let msg = ProtocolMessage::Query(query_request);
    let json = msg.to_json().unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(mock.send_message(json_value).await.is_ok());

    // Receive and verify large response
    let response = mock.recv_message().await.unwrap();
    assert!(response.is_some());

    let response_str = serde_json::to_string(&response.unwrap()).unwrap();
    let response_msg = ProtocolMessage::from_json(&response_str).unwrap();

    if let ProtocolMessage::Response(resp) = response_msg {
        match &resp.message.content[0] {
            ContentBlock::Text { text } => {
                assert_eq!(text.len(), 10000);
            }
            _ => panic!("Expected text content"),
        }
    }
}

#[tokio::test]
async fn test_response_filtering_by_type() {
    let mock = MockCliTransport::new();

    // Queue response with mixed types
    let blocks = vec![
        ContentBlock::Text {
            text: "Text 1".to_string(),
        },
        ContentBlock::ToolUse {
            id: "t1".to_string(),
            name: "tool1".to_string(),
            input: serde_json::json!({}),
        },
        ContentBlock::Text {
            text: "Text 2".to_string(),
        },
        ContentBlock::ToolResult {
            tool_use_id: "t1".to_string(),
            content: Some("Result".to_string()),
            is_error: Some(false),
        },
        ContentBlock::Text {
            text: "Text 3".to_string(),
        },
    ];
    mock.enqueue_response(ProtocolMessage::Response(create_test_response(blocks)))
        .await;

    // Send query
    let query_request = QueryRequest {
        query: "Filter test".to_string(),
        model: "claude-sonnet-4-5".to_string(),
        max_tokens: 1024,
        tools: vec![],
        messages: vec![],
        system_prompt: None,
    };

    let msg = ProtocolMessage::Query(query_request);
    let json = msg.to_json().unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(mock.send_message(json_value).await.is_ok());

    // Receive and filter
    let response = mock.recv_message().await.unwrap();
    assert!(response.is_some());

    let response_str = serde_json::to_string(&response.unwrap()).unwrap();
    let response_msg = ProtocolMessage::from_json(&response_str).unwrap();

    if let ProtocolMessage::Response(resp) = response_msg {
        // Filter to only text blocks
        let text_blocks: Vec<&ContentBlock> = resp
            .message
            .content
            .iter()
            .filter(|block| matches!(block, ContentBlock::Text { .. }))
            .collect();

        assert_eq!(text_blocks.len(), 3);

        // Filter to only tool use
        let tool_blocks: Vec<&ContentBlock> = resp
            .message
            .content
            .iter()
            .filter(|block| matches!(block, ContentBlock::ToolUse { .. }))
            .collect();

        assert_eq!(tool_blocks.len(), 1);
    }
}

#[tokio::test]
async fn test_response_content_block_mapping() {
    let mock = MockCliTransport::new();

    // Queue response
    let blocks = vec![
        ContentBlock::Text {
            text: "Response".to_string(),
        },
        ContentBlock::Text {
            text: "with".to_string(),
        },
        ContentBlock::Text {
            text: "words".to_string(),
        },
    ];
    mock.enqueue_response(ProtocolMessage::Response(create_test_response(blocks)))
        .await;

    // Send query
    let query_request = QueryRequest {
        query: "Map test".to_string(),
        model: "claude-sonnet-4-5".to_string(),
        max_tokens: 1024,
        tools: vec![],
        messages: vec![],
        system_prompt: None,
    };

    let msg = ProtocolMessage::Query(query_request);
    let json = msg.to_json().unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(mock.send_message(json_value).await.is_ok());

    // Receive and map
    let response = mock.recv_message().await.unwrap();
    assert!(response.is_some());

    let response_str = serde_json::to_string(&response.unwrap()).unwrap();
    let response_msg = ProtocolMessage::from_json(&response_str).unwrap();

    if let ProtocolMessage::Response(resp) = response_msg {
        // Map content blocks to text
        let texts: Vec<String> = resp
            .message
            .content
            .iter()
            .filter_map(|block| {
                if let ContentBlock::Text { text } = block {
                    Some(text.clone())
                } else {
                    None
                }
            })
            .collect();

        // Join to verify mapping
        let joined = texts.join(" ");
        assert_eq!(joined, "Response with words");
    }
}

// ===== New streaming message parser tests =====

#[tokio::test]
async fn test_stream_event_parsing() {
    use turboclaudeagent::message_parser::parse_message;

    let json = json!({
        "type": "stream_event",
        "uuid": "evt_456",
        "session_id": "session_456",
        "event": {
            "type": "message_start",
            "message": {
                "id": "msg_123",
                "type": "message",
                "role": "assistant",
                "content": [],
                "model": "claude-sonnet-4-5-20250514"
            }
        },
        "parent_tool_use_id": "tool_789"
    });

    let parsed = parse_message(json).unwrap();

    match parsed {
        ParsedMessage::StreamEvent(event) => {
            assert_eq!(event.uuid, "evt_456");
            assert_eq!(event.session_id, "session_456");
            assert_eq!(event.parent_tool_use_id, Some("tool_789".to_string()));
            assert!(event.event.is_object());

            // Verify event structure
            let event_obj = event.event.as_object().unwrap();
            assert_eq!(
                event_obj.get("type").unwrap().as_str().unwrap(),
                "message_start"
            );
        }
        _ => panic!("Expected StreamEvent"),
    }
}

#[tokio::test]
async fn test_partial_message_handling() {
    use turboclaudeagent::message_parser::parse_message;

    // Test content_block_start event
    let start_event = json!({
        "type": "stream_event",
        "uuid": "evt_001",
        "session_id": "session_001",
        "event": {
            "type": "content_block_start",
            "index": 0,
            "content_block": {
                "type": "text",
                "text": ""
            }
        }
    });

    let parsed = parse_message(start_event).unwrap();
    assert!(matches!(parsed, ParsedMessage::StreamEvent(_)));

    // Test content_block_delta event
    let delta_event = json!({
        "type": "stream_event",
        "uuid": "evt_002",
        "session_id": "session_001",
        "event": {
            "type": "content_block_delta",
            "index": 0,
            "delta": {
                "type": "text_delta",
                "text": "Hello"
            }
        }
    });

    let parsed = parse_message(delta_event).unwrap();
    assert!(matches!(parsed, ParsedMessage::StreamEvent(_)));

    // Test content_block_stop event
    let stop_event = json!({
        "type": "stream_event",
        "uuid": "evt_003",
        "session_id": "session_001",
        "event": {
            "type": "content_block_stop",
            "index": 0
        }
    });

    let parsed = parse_message(stop_event).unwrap();
    assert!(matches!(parsed, ParsedMessage::StreamEvent(_)));

    // Test message_delta event
    let message_delta = json!({
        "type": "stream_event",
        "uuid": "evt_004",
        "session_id": "session_001",
        "event": {
            "type": "message_delta",
            "delta": {
                "stop_reason": "end_turn"
            },
            "usage": {
                "output_tokens": 25
            }
        }
    });

    let parsed = parse_message(message_delta).unwrap();
    assert!(matches!(parsed, ParsedMessage::StreamEvent(_)));

    // Test message_stop event
    let message_stop = json!({
        "type": "stream_event",
        "uuid": "evt_005",
        "session_id": "session_001",
        "event": {
            "type": "message_stop"
        }
    });

    let parsed = parse_message(message_stop).unwrap();
    assert!(matches!(parsed, ParsedMessage::StreamEvent(_)));
}

#[tokio::test]
async fn test_result_message_with_usage() {
    use turboclaudeagent::message_parser::parse_message;

    let json = json!({
        "type": "result",
        "subtype": "query_complete",
        "duration_ms": 2500,
        "duration_api_ms": 2000,
        "is_error": false,
        "num_turns": 5,
        "session_id": "session_xyz",
        "total_cost_usd": 0.15,
        "usage": {
            "input_tokens": 1000,
            "output_tokens": 500,
            "cache_creation_input_tokens": 0,
            "cache_read_input_tokens": 0
        },
        "result": "Query completed successfully"
    });

    let parsed = parse_message(json).unwrap();

    match parsed {
        ParsedMessage::Result(result) => {
            assert_eq!(result.subtype, "query_complete");
            assert_eq!(result.duration_ms, 2500);
            assert_eq!(result.duration_api_ms, 2000);
            assert!(!result.is_error);
            assert_eq!(result.num_turns, 5);
            assert_eq!(result.session_id, "session_xyz");
            assert_eq!(result.total_cost_usd, Some(0.15));
            assert!(result.usage.is_some());
            assert_eq!(
                result.result,
                Some("Query completed successfully".to_string())
            );
        }
        _ => panic!("Expected Result message"),
    }
}

#[tokio::test]
async fn test_system_message_subtypes() {
    use turboclaudeagent::message_parser::parse_message;

    // Test initialize subtype
    let init_msg = json!({
        "type": "system",
        "subtype": "initialize",
        "data": {
            "version": "1.0.0",
            "capabilities": ["streaming", "tool_use"]
        }
    });

    let parsed = parse_message(init_msg).unwrap();
    match parsed {
        ParsedMessage::System(system) => {
            assert_eq!(system.subtype, "initialize");
            assert!(system.data.is_object());
        }
        _ => panic!("Expected System message"),
    }

    // Test status subtype
    let status_msg = json!({
        "type": "system",
        "subtype": "status",
        "data": {
            "connected": true,
            "model": "claude-sonnet-4-5-20250514"
        }
    });

    let parsed = parse_message(status_msg).unwrap();
    match parsed {
        ParsedMessage::System(system) => {
            assert_eq!(system.subtype, "status");
        }
        _ => panic!("Expected System message"),
    }
}

#[tokio::test]
async fn test_complex_assistant_message_with_tool_use() {
    use turboclaudeagent::message_parser::parse_message;

    let json = json!({
        "type": "assistant",
        "message": {
            "content": [
                {
                    "type": "thinking",
                    "thinking": "I need to check the current directory"
                },
                {
                    "type": "text",
                    "text": "Let me list the files for you."
                },
                {
                    "type": "tool_use",
                    "id": "toolu_123",
                    "name": "bash",
                    "input": {
                        "command": "ls -la"
                    }
                }
            ],
            "model": "claude-sonnet-4-5-20250514"
        }
    });

    let parsed = parse_message(json).unwrap();

    match parsed {
        ParsedMessage::Assistant(assistant) => {
            assert_eq!(assistant.content.len(), 3);
            assert_eq!(assistant.content[0].type_name(), "thinking");
            assert!(assistant.content[1].is_text());
            assert!(assistant.content[2].is_tool_use());

            // Verify tool use details
            if let Some((id, name, input)) = assistant.content[2].as_tool_use() {
                assert_eq!(id, "toolu_123");
                assert_eq!(name, "bash");
                assert_eq!(input.get("command").unwrap().as_str().unwrap(), "ls -la");
            } else {
                panic!("Expected tool use block");
            }
        }
        _ => panic!("Expected Assistant message"),
    }
}

#[tokio::test]
async fn test_error_result_message() {
    use turboclaudeagent::message_parser::parse_message;

    let json = json!({
        "type": "result",
        "subtype": "query_error",
        "duration_ms": 500,
        "duration_api_ms": 100,
        "is_error": true,
        "num_turns": 1,
        "session_id": "session_err",
        "result": "API rate limit exceeded"
    });

    let parsed = parse_message(json).unwrap();

    match parsed {
        ParsedMessage::Result(result) => {
            assert!(result.is_error);
            assert_eq!(result.subtype, "query_error");
            assert_eq!(result.result, Some("API rate limit exceeded".to_string()));
        }
        _ => panic!("Expected Result message"),
    }
}
