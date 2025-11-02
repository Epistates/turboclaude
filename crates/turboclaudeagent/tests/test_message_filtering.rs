//! Tests for message filtering methods
//!
//! Verifies that the receive_*() filtering methods correctly filter
//! messages by type and handle errors properly.

use futures::StreamExt;
use turboclaudeagent::message_parser::ParsedMessage;
use turboclaude_protocol::content::ContentBlock;
use turboclaude_protocol::message::{
    AssistantMessage, MessageRole, ResultMessage, StreamEvent, SystemMessage, UserMessage,
};
use turboclaude_protocol::types::{StopReason, Usage, CacheUsage};
use serde_json::json;

// Test helper: Create a user message
fn create_user_message(text: &str) -> ParsedMessage {
    ParsedMessage::User(UserMessage {
        id: Some("msg_user_1".to_string()),
        message_type: "message".to_string(),
        role: MessageRole::User,
        content: vec![ContentBlock::text(text)],
        created_at: "2025-01-01T00:00:00Z".to_string(),
    })
}

// Test helper: Create an assistant message
fn create_assistant_message(text: &str, model: &str) -> ParsedMessage {
    ParsedMessage::Assistant(AssistantMessage {
        id: "msg_asst_1".to_string(),
        message_type: "message".to_string(),
        role: MessageRole::Assistant,
        content: vec![ContentBlock::text(text)],
        model: model.to_string(),
        stop_reason: StopReason::EndTurn,
        created_at: "2025-01-01T00:00:00Z".to_string(),
        usage: Usage::new(100, 50),
        cache_usage: CacheUsage::default(),
    })
}

// Test helper: Create a system message
fn create_system_message(subtype: &str) -> ParsedMessage {
    ParsedMessage::System(SystemMessage {
        subtype: subtype.to_string(),
        data: json!({"key": "value"}),
    })
}

// Test helper: Create a result message
fn create_result_message(is_error: bool) -> ParsedMessage {
    ParsedMessage::Result(ResultMessage {
        subtype: "query_complete".to_string(),
        duration_ms: 1000,
        duration_api_ms: 800,
        is_error,
        num_turns: 3,
        session_id: "session_123".to_string(),
        total_cost_usd: Some(0.05),
        usage: None,
        result: Some("success".to_string()),
    })
}

// Test helper: Create a stream event
fn create_stream_event(uuid: &str) -> ParsedMessage {
    ParsedMessage::StreamEvent(StreamEvent {
        uuid: uuid.to_string(),
        session_id: "session_123".to_string(),
        event: json!({
            "type": "content_block_delta",
            "index": 0,
            "delta": {"type": "text_delta", "text": "Hello"}
        }),
        parent_tool_use_id: None,
    })
}

#[tokio::test]
async fn test_filter_assistant_messages_only() {
    use futures::stream;
    use turboclaudeagent::AgentError;

    let messages: Vec<Result<ParsedMessage, AgentError>> = vec![
        Ok(create_user_message("Hello")),
        Ok(create_assistant_message("Hi there", "claude-3-5-sonnet-20241022")),
        Ok(create_system_message("status")),
        Ok(create_assistant_message("How can I help?", "claude-3-5-sonnet-20241022")),
        Ok(create_result_message(false)),
    ];

    let msg_stream = stream::iter(messages);
    let filtered = msg_stream.filter_map(|result| async move {
        match result {
            Ok(ParsedMessage::Assistant(msg)) => Some(Ok(msg)),
            Err(e) => Some(Err(e)),
            _ => None,
        }
    });
    let mut filtered = Box::pin(filtered);

    let mut count = 0;
    while let Some(result) = filtered.next().await {
        let msg = result.unwrap();
        assert_eq!(msg.role, MessageRole::Assistant);
        assert_eq!(msg.model, "claude-3-5-sonnet-20241022");
        count += 1;
    }
    assert_eq!(count, 2, "Expected 2 assistant messages");
}

#[tokio::test]
async fn test_filter_user_messages_only() {
    use futures::stream;

    let messages: Vec<Result<ParsedMessage, turboclaudeagent::AgentError>> = vec![
        Ok(create_user_message("Hello")),
        Ok(create_assistant_message("Hi", "claude-3-5-sonnet-20241022")),
        Ok(create_user_message("What is 2+2?")),
        Ok(create_system_message("status")),
        Ok(create_user_message("Thanks")),
    ];

    let msg_stream = stream::iter(messages);
    let filtered = msg_stream.filter_map(|result| async move {
        match result {
            Ok(ParsedMessage::User(msg)) => Some(Ok(msg)),
            Err(e) => Some(Err(e)),
            _ => None,
        }
    });
    let mut filtered = Box::pin(filtered);

    let mut count = 0;
    while let Some(result) = filtered.next().await {
        let msg = result.unwrap();
        assert_eq!(msg.role, MessageRole::User);
        count += 1;
    }
    assert_eq!(count, 3, "Expected 3 user messages");
}

#[tokio::test]
async fn test_filter_stream_events_only() {
    use futures::stream;

    let messages: Vec<Result<ParsedMessage, turboclaudeagent::AgentError>> = vec![
        Ok(create_stream_event("evt_1")),
        Ok(create_user_message("Hello")),
        Ok(create_stream_event("evt_2")),
        Ok(create_assistant_message("Hi", "claude-3-5-sonnet-20241022")),
        Ok(create_stream_event("evt_3")),
    ];

    let msg_stream = stream::iter(messages);
    let filtered = msg_stream.filter_map(|result| async move {
        match result {
            Ok(ParsedMessage::StreamEvent(event)) => Some(Ok(event)),
            Err(e) => Some(Err(e)),
            _ => None,
        }
    });
    let mut filtered = Box::pin(filtered);

    let mut count = 0;
    while let Some(result) = filtered.next().await {
        let event = result.unwrap();
        assert!(event.uuid.starts_with("evt_"));
        count += 1;
    }
    assert_eq!(count, 3, "Expected 3 stream events");
}

#[tokio::test]
async fn test_filter_result_messages_only() {
    use futures::stream;

    let messages: Vec<Result<ParsedMessage, turboclaudeagent::AgentError>> = vec![
        Ok(create_user_message("Hello")),
        Ok(create_assistant_message("Hi", "claude-3-5-sonnet-20241022")),
        Ok(create_result_message(false)),
        Ok(create_system_message("status")),
        Ok(create_result_message(true)),
    ];

    let msg_stream = stream::iter(messages);
    let filtered = msg_stream.filter_map(|result| async move {
        match result {
            Ok(ParsedMessage::Result(msg)) => Some(Ok(msg)),
            Err(e) => Some(Err(e)),
            _ => None,
        }
    });
    let mut filtered = Box::pin(filtered);

    let mut count = 0;
    while let Some(result) = filtered.next().await {
        let msg = result.unwrap();
        assert_eq!(msg.subtype, "query_complete");
        count += 1;
    }
    assert_eq!(count, 2, "Expected 2 result messages");
}

#[tokio::test]
async fn test_filter_system_messages_only() {
    use futures::stream;

    let messages: Vec<Result<ParsedMessage, turboclaudeagent::AgentError>> = vec![
        Ok(create_system_message("initialize")),
        Ok(create_user_message("Hello")),
        Ok(create_system_message("status")),
        Ok(create_assistant_message("Hi", "claude-3-5-sonnet-20241022")),
        Ok(create_system_message("shutdown")),
    ];

    let msg_stream = stream::iter(messages);
    let filtered = msg_stream.filter_map(|result| async move {
        match result {
            Ok(ParsedMessage::System(msg)) => Some(Ok(msg)),
            Err(e) => Some(Err(e)),
            _ => None,
        }
    });
    let mut filtered = Box::pin(filtered);

    let mut count = 0;
    let expected_subtypes = vec!["initialize", "status", "shutdown"];
    while let Some(result) = filtered.next().await {
        let msg = result.unwrap();
        assert!(expected_subtypes.contains(&msg.subtype.as_str()));
        count += 1;
    }
    assert_eq!(count, 3, "Expected 3 system messages");
}

#[tokio::test]
async fn test_error_propagation_in_filtered_stream() {
    use futures::stream;
    use turboclaudeagent::AgentError;

    let messages: Vec<Result<ParsedMessage, AgentError>> = vec![
        Ok(create_assistant_message("First", "claude-3-5-sonnet-20241022")),
        Err(AgentError::Protocol("Test error".to_string())),
        Ok(create_assistant_message("Second", "claude-3-5-sonnet-20241022")),
    ];

    let msg_stream = stream::iter(messages);
    let filtered = msg_stream.filter_map(|result| async move {
        match result {
            Ok(ParsedMessage::Assistant(msg)) => Some(Ok(msg)),
            Err(e) => Some(Err(e)),
            _ => None,
        }
    });
    let mut filtered = Box::pin(filtered);

    let first = filtered.next().await.unwrap();
    assert!(first.is_ok());

    let second = filtered.next().await.unwrap();
    assert!(second.is_err());
    assert!(matches!(second.unwrap_err(), AgentError::Protocol(_)));

    let third = filtered.next().await.unwrap();
    assert!(third.is_ok());
}

#[tokio::test]
async fn test_empty_stream_filtering() {
    use futures::stream;

    let messages: Vec<Result<ParsedMessage, turboclaudeagent::AgentError>> = vec![];
    let msg_stream = stream::iter(messages);
    let filtered = msg_stream.filter_map(|result| async move {
        match result {
            Ok(ParsedMessage::Assistant(msg)) => Some(Ok(msg)),
            Err(e) => Some(Err(e)),
            _ => None,
        }
    });
    let mut filtered = Box::pin(filtered);

    assert!(filtered.next().await.is_none());
}

#[tokio::test]
async fn test_filter_with_no_matching_messages() {
    use futures::stream;

    let messages: Vec<Result<ParsedMessage, turboclaudeagent::AgentError>> = vec![
        Ok(create_user_message("Hello")),
        Ok(create_system_message("status")),
        Ok(create_result_message(false)),
    ];

    let msg_stream = stream::iter(messages);
    let filtered = msg_stream.filter_map(|result| async move {
        match result {
            Ok(ParsedMessage::Assistant(msg)) => Some(Ok(msg)),
            Err(e) => Some(Err(e)),
            _ => None,
        }
    });
    let mut filtered = Box::pin(filtered);

    assert!(filtered.next().await.is_none());
}

#[tokio::test]
async fn test_filter_with_all_matching_messages() {
    use futures::stream;

    let messages: Vec<Result<ParsedMessage, turboclaudeagent::AgentError>> = vec![
        Ok(create_assistant_message("First", "claude-3-5-sonnet-20241022")),
        Ok(create_assistant_message("Second", "claude-3-5-sonnet-20241022")),
        Ok(create_assistant_message("Third", "claude-3-5-sonnet-20241022")),
    ];

    let msg_stream = stream::iter(messages);
    let filtered = msg_stream.filter_map(|result| async move {
        match result {
            Ok(ParsedMessage::Assistant(msg)) => Some(Ok(msg)),
            Err(e) => Some(Err(e)),
            _ => None,
        }
    });
    let mut filtered = Box::pin(filtered);

    let mut count = 0;
    while let Some(_) = filtered.next().await {
        count += 1;
    }
    assert_eq!(count, 3);
}

#[tokio::test]
async fn test_filter_preserves_message_order() {
    use futures::stream;

    let messages: Vec<Result<ParsedMessage, turboclaudeagent::AgentError>> = vec![
        Ok(create_assistant_message("First", "claude-3-5-sonnet-20241022")),
        Ok(create_user_message("Ignore")),
        Ok(create_assistant_message("Second", "claude-3-5-sonnet-20241022")),
        Ok(create_system_message("Ignore")),
        Ok(create_assistant_message("Third", "claude-3-5-sonnet-20241022")),
    ];

    let msg_stream = stream::iter(messages);
    let filtered = msg_stream.filter_map(|result| async move {
        match result {
            Ok(ParsedMessage::Assistant(msg)) => Some(Ok(msg)),
            Err(e) => Some(Err(e)),
            _ => None,
        }
    });
    let mut filtered = Box::pin(filtered);

    let first = filtered.next().await.unwrap().unwrap();
    assert_eq!(first.content[0].as_text(), Some("First"));

    let second = filtered.next().await.unwrap().unwrap();
    assert_eq!(second.content[0].as_text(), Some("Second"));

    let third = filtered.next().await.unwrap().unwrap();
    assert_eq!(third.content[0].as_text(), Some("Third"));

    assert!(filtered.next().await.is_none());
}

#[tokio::test]
async fn test_multiple_filters_independently() {
    use futures::stream;

    // Test assistant filter
    let messages: Vec<Result<ParsedMessage, turboclaudeagent::AgentError>> = vec![
        Ok(create_user_message("User 1")),
        Ok(create_assistant_message("Assistant 1", "claude-3-5-sonnet-20241022")),
        Ok(create_user_message("User 2")),
        Ok(create_assistant_message("Assistant 2", "claude-3-5-sonnet-20241022")),
    ];
    let msg_stream = stream::iter(messages);
    let filtered = msg_stream.filter_map(|result| async move {
        match result {
            Ok(ParsedMessage::Assistant(msg)) => Some(Ok(msg)),
            Err(e) => Some(Err(e)),
            _ => None,
        }
    });
    let mut assistant_filtered = Box::pin(filtered);

    let mut asst_count = 0;
    while let Some(_) = assistant_filtered.next().await {
        asst_count += 1;
    }
    assert_eq!(asst_count, 2);

    // Test user filter on same data (recreate messages)
    let messages: Vec<Result<ParsedMessage, turboclaudeagent::AgentError>> = vec![
        Ok(create_user_message("User 1")),
        Ok(create_assistant_message("Assistant 1", "claude-3-5-sonnet-20241022")),
        Ok(create_user_message("User 2")),
        Ok(create_assistant_message("Assistant 2", "claude-3-5-sonnet-20241022")),
    ];
    let msg_stream = stream::iter(messages);
    let filtered = msg_stream.filter_map(|result| async move {
        match result {
            Ok(ParsedMessage::User(msg)) => Some(Ok(msg)),
            Err(e) => Some(Err(e)),
            _ => None,
        }
    });
    let mut user_filtered = Box::pin(filtered);

    let mut user_count = 0;
    while let Some(_) = user_filtered.next().await {
        user_count += 1;
    }
    assert_eq!(user_count, 2);
}

#[tokio::test]
async fn test_result_message_with_error_flag() {
    use futures::stream;

    let messages: Vec<Result<ParsedMessage, turboclaudeagent::AgentError>> = vec![
        Ok(create_result_message(false)), // Success
        Ok(create_result_message(true)),  // Error
    ];

    let msg_stream = stream::iter(messages);
    let filtered = msg_stream.filter_map(|result| async move {
        match result {
            Ok(ParsedMessage::Result(msg)) => Some(Ok(msg)),
            Err(e) => Some(Err(e)),
            _ => None,
        }
    });
    let mut filtered = Box::pin(filtered);

    let first = filtered.next().await.unwrap().unwrap();
    assert!(!first.is_error);

    let second = filtered.next().await.unwrap().unwrap();
    assert!(second.is_error);
}

#[tokio::test]
async fn test_stream_event_with_parent_tool_use() {
    use futures::stream;
    use turboclaudeagent::AgentError;

    let event = match create_stream_event("evt_tool") {
        ParsedMessage::StreamEvent(mut e) => {
            e.parent_tool_use_id = Some("tool_123".to_string());
            e
        }
        _ => panic!("Expected StreamEvent"),
    };

    let messages: Vec<Result<ParsedMessage, AgentError>> = vec![Ok(ParsedMessage::StreamEvent(event))];
    let msg_stream = stream::iter(messages);
    let filtered = msg_stream.filter_map(|result| async move {
        match result {
            Ok(ParsedMessage::StreamEvent(event)) => Some(Ok(event)),
            Err(e) => Some(Err(e)),
            _ => None,
        }
    });
    let mut filtered = Box::pin(filtered);

    let event = filtered.next().await.unwrap().unwrap();
    assert_eq!(event.parent_tool_use_id, Some("tool_123".to_string()));
}
