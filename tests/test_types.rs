//! Type serialization tests
//!
//! Tests for serde serialization/deserialization of all SDK types:
//! - Message types
//! - Content blocks
//! - Tool types
//! - Error types
//! - Roundtrip testing

use turboclaude::{TokenCount, BatchRequest, types::*};
use serde_json::json;
use pretty_assertions::assert_eq;
use insta::assert_json_snapshot;

#[test]
fn test_message_serialization() {
    let message = Message {
        id: "msg_123".to_string(),
        message_type: "message".to_string(),
        role: Role::Assistant,
        content: vec![ContentBlock::Text {
            text: "Hello".to_string(),
        }],
        model: "claude-3-5-sonnet-20241022".to_string(),
        stop_reason: Some(StopReason::EndTurn),
        stop_sequence: None,
        usage: Usage {
            input_tokens: 10,
            output_tokens: 5,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        },
    };

    // Serialize
    let json = serde_json::to_value(&message).unwrap();

    // Verify fields
    assert_eq!(json["id"], "msg_123");
    assert_eq!(json["role"], "assistant");
    assert_eq!(json["model"], "claude-3-5-sonnet-20241022");

    // Deserialize
    let deserialized: Message = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.id, message.id);
    assert_eq!(deserialized.role, message.role);
}

#[test]
fn test_message_param_serialization() {
    let param = MessageParam {
        role: Role::User,
        content: vec![ContentBlockParam::Text {
            text: "Hello".to_string(),
        }],
    };

    let json = serde_json::to_value(&param).unwrap();

    assert_eq!(json["role"], "user");
    assert!(json["content"].is_array());

    // Roundtrip
    let deserialized: MessageParam = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.role, param.role);
}

#[test]
fn test_content_block_text() {
    let block = ContentBlock::Text {
        text: "Test text".to_string(),
    };

    let json = serde_json::to_value(&block).unwrap();

    assert_eq!(json["type"], "text");
    assert_eq!(json["text"], "Test text");

    // Roundtrip
    let deserialized: ContentBlock = serde_json::from_value(json).unwrap();
    match deserialized {
        ContentBlock::Text { text } => assert_eq!(text, "Test text"),
        _ => panic!("Wrong content block type"),
    }
}

#[test]
fn test_content_block_tool_use() {
    let input = json!({
        "operation": "add",
        "a": 1,
        "b": 2
    });

    let block = ContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input,
    };

    let json = serde_json::to_value(&block).unwrap();

    assert_eq!(json["type"], "tool_use");
    assert_eq!(json["id"], "tool_123");
    assert_eq!(json["name"], "calculator");

    // Roundtrip
    let deserialized: ContentBlock = serde_json::from_value(json).unwrap();
    match deserialized {
        ContentBlock::ToolUse { id, name, .. } => {
            assert_eq!(id, "tool_123");
            assert_eq!(name, "calculator");
        }
        _ => panic!("Wrong content block type"),
    }
}

#[test]
fn test_role_serialization() {
    assert_eq!(serde_json::to_value(Role::User).unwrap(), "user");
    assert_eq!(serde_json::to_value(Role::Assistant).unwrap(), "assistant");

    // Deserialize
    assert_eq!(
        serde_json::from_value::<Role>(json!("user")).unwrap(),
        Role::User
    );
    assert_eq!(
        serde_json::from_value::<Role>(json!("assistant")).unwrap(),
        Role::Assistant
    );
}

#[test]
fn test_usage_serialization() {
    let usage = Usage {
        input_tokens: 100,
        output_tokens: 50,
        cache_creation_input_tokens: None,
        cache_read_input_tokens: None,
    };

    let json = serde_json::to_value(&usage).unwrap();

    assert_eq!(json["input_tokens"], 100);
    assert_eq!(json["output_tokens"], 50);

    // Roundtrip
    let deserialized: Usage = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.input_tokens, usage.input_tokens);
    assert_eq!(deserialized.output_tokens, usage.output_tokens);
}

#[test]
fn test_tool_serialization() {
    let schema = json!({
        "type": "object",
        "properties": {
            "location": {"type": "string"}
        }
    });

    let tool = Tool::new("get_weather", "Gets weather info", schema.clone());

    let json = serde_json::to_value(&tool).unwrap();

    assert_eq!(json["name"], "get_weather");
    assert_eq!(json["description"], "Gets weather info");

    // Roundtrip
    let deserialized: Tool = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.name, "get_weather");
}

#[test]
fn test_tool_choice_serialization() {
    // Auto
    let auto = ToolChoice::Auto;
    assert_eq!(serde_json::to_value(auto).unwrap(), json!({"type": "auto"}));

    // Any
    let any = ToolChoice::Any;
    assert_eq!(serde_json::to_value(any).unwrap(), json!({"type": "any"}));

    // Tool
    let tool = ToolChoice::Tool {
        name: "calculator".to_string(),
    };
    let json = serde_json::to_value(tool).unwrap();
    assert_eq!(json["type"], "tool");
    assert_eq!(json["name"], "calculator");
}

#[test]
fn test_message_request_builder() {
    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .temperature(0.7)
        .system("You are helpful")
        .build()
        .unwrap();

    assert_eq!(request.model, Models::CLAUDE_3_5_SONNET);
    assert_eq!(request.max_tokens, 1024);
    assert_eq!(request.temperature, Some(0.7));
    assert_eq!(request.system, Some(SystemPrompt::from("You are helpful")));
}

#[test]
fn test_message_request_serialization() {
    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    let json = serde_json::to_value(&request).unwrap();

    assert_eq!(json["model"], Models::CLAUDE_3_5_SONNET);
    assert_eq!(json["max_tokens"], 1024);
    assert!(json["messages"].is_array());
}

#[test]
fn test_batch_request_serialization() {
    let message_request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    let batch_request = BatchRequest {
        custom_id: "request-1".to_string(),
        params: message_request,
    };

    let json = serde_json::to_value(&batch_request).unwrap();

    assert_eq!(json["custom_id"], "request-1");
    assert!(json["params"].is_object());

    // Roundtrip
    let deserialized: BatchRequest = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.custom_id, "request-1");
}

#[test]
fn test_model_serialization() {
    let model = Model {
        model_type: "model".to_string(),
        id: "claude-3-5-sonnet-20241022".to_string(),
        display_name: "Claude 3.5 Sonnet".to_string(),
        created_at: chrono::DateTime::parse_from_rfc3339("2024-10-22T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc),
    };

    let json = serde_json::to_value(&model).unwrap();

    assert_eq!(json["id"], "claude-3-5-sonnet-20241022");
    assert_eq!(json["display_name"], "Claude 3.5 Sonnet");

    // Roundtrip
    let deserialized: Model = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.id, model.id);
}

#[test]
fn test_token_count_serialization() {
    let token_count = TokenCount {
        input_tokens: 123,
    };

    let json = serde_json::to_value(&token_count).unwrap();

    assert_eq!(json["input_tokens"], 123);

    // Roundtrip
    let deserialized: TokenCount = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.input_tokens, 123);
}

/// Snapshot tests for complex types
#[test]
fn test_message_snapshot() {
    let message = Message {
        id: "msg_123".to_string(),
        message_type: "message".to_string(),
        role: Role::Assistant,
        content: vec![
            ContentBlock::Text {
                text: "Hello".to_string(),
            },
            ContentBlock::ToolUse {
                id: "tool_1".to_string(),
                name: "calculator".to_string(),
                input: json!({"a": 1, "b": 2}),
            },
        ],
        model: "claude-3-5-sonnet-20241022".to_string(),
        stop_reason: Some(StopReason::ToolUse),
        stop_sequence: None,
        usage: Usage {
            input_tokens: 10,
            output_tokens: 20,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        },
    };

    assert_json_snapshot!(message);
}

#[test]
fn test_message_request_snapshot() {
    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![
            Message::user("What is 2+2?"),
            Message::assistant("Let me calculate that."),
        ])
        .temperature(0.7)
        .system("You are a helpful calculator")
        .build()
        .unwrap();

    assert_json_snapshot!(request);
}

/// Test that all Model constants are valid
#[test]
fn test_model_constants() {
    let models = vec![
        Models::CLAUDE_3_5_SONNET,
        Models::CLAUDE_3_OPUS,
        Models::CLAUDE_3_SONNET,
        Models::CLAUDE_3_HAIKU,
    ];

    for model in models {
        // Each should be a non-empty string with valid format
        assert!(!model.is_empty());
        assert!(model.starts_with("claude-"));
        assert!(model.contains("20")); // Should have a date
    }
}

/// Test MessageParam helper methods
#[test]
fn test_message_param_helpers() {
    let user_msg = Message::user("Hello");
    assert_eq!(user_msg.role, Role::User);

    let assistant_msg = Message::assistant("Hi there");
    assert_eq!(assistant_msg.role, Role::Assistant);
}

/// Test Message text() helper
#[test]
fn test_message_text_helper() {
    let message = Message {
        id: "msg_123".to_string(),
        message_type: "message".to_string(),
        role: Role::Assistant,
        content: vec![
            ContentBlock::Text {
                text: "Hello ".to_string(),
            },
            ContentBlock::Text {
                text: "world!".to_string(),
            },
        ],
        model: "claude-3-5-sonnet-20241022".to_string(),
        stop_reason: Some(StopReason::EndTurn),
        stop_sequence: None,
        usage: Usage {
            input_tokens: 5,
            output_tokens: 3,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        },
    };

    assert_eq!(message.text(), "Hello world!");
}
