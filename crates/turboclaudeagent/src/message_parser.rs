//! Message parsing for Claude Agent SDK protocol.
//!
//! Handles parsing of JSON messages from the Claude CLI into typed message enums.
//! The CLI protocol uses a different format than the REST API, with a "type" field
//! at the top level and nested message objects.
//!
//! # Message Types
//!
//! The parser handles five message types:
//! - `user`: User messages sent to Claude
//! - `assistant`: Assistant responses from Claude
//! - `system`: System messages from the CLI
//! - `result`: Final result messages indicating query completion
//! - `stream_event`: Partial message updates during streaming
//!
//! # Example
//!
//! ```ignore
//! use turboclaudeagent::message_parser::parse_message;
//! use serde_json::json;
//!
//! let json = json!({
//!     "type": "user",
//!     "message": {
//!         "content": [{"type": "text", "text": "Hello"}]
//!     }
//! });
//!
//! let msg = parse_message(json)?;
//! ```

use serde_json::Value;
use turboclaude_protocol::content::ContentBlock;
use turboclaude_protocol::message::{
    AssistantMessage, MessageRole, ResultMessage, StreamEvent, SystemMessage, UserMessage,
};

/// Errors that can occur during message parsing
#[derive(Debug, thiserror::Error)]
pub enum MessageParseError {
    /// Invalid message format
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Unknown message type
    #[error("Unknown message type: {0}")]
    UnknownType(String),

    /// JSON deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Invalid content block type
    #[error("Invalid content block type: {0}")]
    InvalidContentBlock(String),
}

/// A parsed message from the Claude CLI
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedMessage {
    /// User message
    User(UserMessage),

    /// Assistant message
    Assistant(AssistantMessage),

    /// System message
    System(SystemMessage),

    /// Result message
    Result(ResultMessage),

    /// Stream event
    StreamEvent(StreamEvent),
}

/// Parse a JSON value into a typed Message
///
/// # Arguments
///
/// * `data` - The JSON value from the CLI output
///
/// # Returns
///
/// A `ParsedMessage` enum containing the typed message
///
/// # Errors
///
/// Returns `MessageParseError` if:
/// - The message is missing the "type" field
/// - The message type is unknown
/// - Required fields are missing
/// - JSON deserialization fails
///
/// # Example
///
/// ```ignore
/// use turboclaudeagent::message_parser::parse_message;
/// use serde_json::json;
///
/// let json = json!({
///     "type": "user",
///     "message": {
///         "content": [{"type": "text", "text": "Hello"}]
///     }
/// });
///
/// let message = parse_message(json)?;
/// ```
pub fn parse_message(data: Value) -> Result<ParsedMessage, MessageParseError> {
    // Validate input is an object
    if !data.is_object() {
        return Err(MessageParseError::InvalidFormat(format!(
            "Expected object, got {}",
            data
        )));
    }

    // Extract message type
    let message_type = data
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MessageParseError::MissingField("type".into()))?;

    match message_type {
        "user" => parse_user_message(data),
        "assistant" => parse_assistant_message(data),
        "system" => parse_system_message(data),
        "result" => parse_result_message(data),
        "stream_event" => parse_stream_event(data),
        _ => Err(MessageParseError::UnknownType(message_type.into())),
    }
}

/// Parse a JSON string into a typed Message
///
/// This is a convenience wrapper around `parse_message` that accepts a string.
///
/// # Arguments
///
/// * `s` - The JSON string to parse
///
/// # Returns
///
/// A `ParsedMessage` enum containing the typed message
///
/// # Errors
///
/// Returns `MessageParseError` if JSON parsing or message parsing fails
pub fn parse_message_str(s: &str) -> Result<ParsedMessage, MessageParseError> {
    let value: Value = serde_json::from_str(s)?;
    parse_message(value)
}

/// Parse a user message
fn parse_user_message(data: Value) -> Result<ParsedMessage, MessageParseError> {
    let message = data
        .get("message")
        .ok_or_else(|| MessageParseError::MissingField("message".into()))?;

    let content = message
        .get("content")
        .ok_or_else(|| MessageParseError::MissingField("message.content".into()))?;

    let content_blocks = parse_content_blocks(content)?;

    let user_msg = UserMessage {
        id: None,
        message_type: "message".to_string(),
        role: MessageRole::User,
        content: content_blocks,
        created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    };

    Ok(ParsedMessage::User(user_msg))
}

/// Parse an assistant message
fn parse_assistant_message(data: Value) -> Result<ParsedMessage, MessageParseError> {
    let message = data
        .get("message")
        .ok_or_else(|| MessageParseError::MissingField("message".into()))?;

    let content = message
        .get("content")
        .ok_or_else(|| MessageParseError::MissingField("message.content".into()))?;

    let model = message
        .get("model")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MessageParseError::MissingField("message.model".into()))?
        .to_string();

    let content_blocks = parse_content_blocks(content)?;

    // Create assistant message with default values
    // Note: In a real implementation, we'd parse usage, stop_reason, etc.
    let assistant_msg = AssistantMessage {
        id: format!("msg_{}", uuid::Uuid::new_v4()),
        message_type: "message".to_string(),
        role: MessageRole::Assistant,
        content: content_blocks,
        model,
        stop_reason: turboclaude_protocol::types::StopReason::EndTurn,
        created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        usage: turboclaude_protocol::types::Usage::new(0, 0),
        cache_usage: turboclaude_protocol::types::CacheUsage::default(),
    };

    Ok(ParsedMessage::Assistant(assistant_msg))
}

/// Parse a system message
fn parse_system_message(data: Value) -> Result<ParsedMessage, MessageParseError> {
    let subtype = data
        .get("subtype")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MessageParseError::MissingField("subtype".into()))?
        .to_string();

    let system_msg = SystemMessage {
        subtype,
        data: data.clone(),
    };

    Ok(ParsedMessage::System(system_msg))
}

/// Parse a result message
fn parse_result_message(data: Value) -> Result<ParsedMessage, MessageParseError> {
    let subtype = data
        .get("subtype")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MessageParseError::MissingField("subtype".into()))?
        .to_string();

    let duration_ms = data
        .get("duration_ms")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| MessageParseError::MissingField("duration_ms".into()))?;

    let duration_api_ms = data
        .get("duration_api_ms")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| MessageParseError::MissingField("duration_api_ms".into()))?;

    let is_error = data
        .get("is_error")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| MessageParseError::MissingField("is_error".into()))?;

    let num_turns =
        data.get("num_turns")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| MessageParseError::MissingField("num_turns".into()))? as u32;

    let session_id = data
        .get("session_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MessageParseError::MissingField("session_id".into()))?
        .to_string();

    let total_cost_usd = data.get("total_cost_usd").and_then(|v| v.as_f64());
    let usage = data.get("usage").cloned();
    let result = data
        .get("result")
        .and_then(|v| v.as_str())
        .map(String::from);

    let result_msg = ResultMessage {
        subtype,
        duration_ms,
        duration_api_ms,
        is_error,
        num_turns,
        session_id,
        total_cost_usd,
        usage,
        result,
    };

    Ok(ParsedMessage::Result(result_msg))
}

/// Parse a stream event
fn parse_stream_event(data: Value) -> Result<ParsedMessage, MessageParseError> {
    let uuid = data
        .get("uuid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MessageParseError::MissingField("uuid".into()))?
        .to_string();

    let session_id = data
        .get("session_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MessageParseError::MissingField("session_id".into()))?
        .to_string();

    let event = data
        .get("event")
        .ok_or_else(|| MessageParseError::MissingField("event".into()))?
        .clone();

    let parent_tool_use_id = data
        .get("parent_tool_use_id")
        .and_then(|v| v.as_str())
        .map(String::from);

    let stream_event = StreamEvent {
        uuid,
        session_id,
        event,
        parent_tool_use_id,
    };

    Ok(ParsedMessage::StreamEvent(stream_event))
}

/// Parse content blocks from JSON
fn parse_content_blocks(content: &Value) -> Result<Vec<ContentBlock>, MessageParseError> {
    // Handle both string and array content
    if let Some(text) = content.as_str() {
        return Ok(vec![ContentBlock::text(text)]);
    }

    let blocks = content.as_array().ok_or_else(|| {
        MessageParseError::InvalidFormat("content must be string or array".into())
    })?;

    let mut content_blocks = Vec::new();

    for block in blocks {
        let block_type = block
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MessageParseError::MissingField("content block type".into()))?;

        let content_block = match block_type {
            "text" => {
                let text = block
                    .get("text")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| MessageParseError::MissingField("text".into()))?;
                ContentBlock::text(text)
            }
            "thinking" => {
                let thinking = block
                    .get("thinking")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| MessageParseError::MissingField("thinking".into()))?;
                ContentBlock::thinking(thinking)
            }
            "tool_use" => {
                let id = block
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| MessageParseError::MissingField("id".into()))?;
                let name = block
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| MessageParseError::MissingField("name".into()))?;
                let input = block
                    .get("input")
                    .ok_or_else(|| MessageParseError::MissingField("input".into()))?
                    .clone();
                ContentBlock::tool_use(id, name, input)
            }
            "tool_result" => {
                let tool_use_id = block
                    .get("tool_use_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| MessageParseError::MissingField("tool_use_id".into()))?;
                let content = block.get("content").and_then(|v| v.as_str());
                let is_error = block.get("is_error").and_then(|v| v.as_bool());

                ContentBlock::ToolResult {
                    tool_use_id: tool_use_id.to_string(),
                    content: content.map(String::from),
                    is_error,
                }
            }
            _ => {
                return Err(MessageParseError::InvalidContentBlock(
                    block_type.to_string(),
                ));
            }
        };

        content_blocks.push(content_block);
    }

    Ok(content_blocks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_user_message() {
        let json = json!({
            "type": "user",
            "message": {
                "content": [{"type": "text", "text": "Hello"}]
            }
        });

        let msg = parse_message(json).unwrap();
        assert!(matches!(msg, ParsedMessage::User(_)));

        if let ParsedMessage::User(user_msg) = msg {
            assert_eq!(user_msg.role, MessageRole::User);
            assert_eq!(user_msg.content.len(), 1);
            assert_eq!(user_msg.content[0].as_text(), Some("Hello"));
        }
    }

    #[test]
    fn test_parse_user_message_string_content() {
        let json = json!({
            "type": "user",
            "message": {
                "content": "Hello, world!"
            }
        });

        let msg = parse_message(json).unwrap();
        assert!(matches!(msg, ParsedMessage::User(_)));

        if let ParsedMessage::User(user_msg) = msg {
            assert_eq!(user_msg.content.len(), 1);
            assert_eq!(user_msg.content[0].as_text(), Some("Hello, world!"));
        }
    }

    #[test]
    fn test_parse_assistant_message() {
        let json = json!({
            "type": "assistant",
            "message": {
                "content": [{"type": "text", "text": "Hi there!"}],
                "model": "claude-3-5-sonnet-20241022"
            }
        });

        let msg = parse_message(json).unwrap();
        assert!(matches!(msg, ParsedMessage::Assistant(_)));

        if let ParsedMessage::Assistant(assistant_msg) = msg {
            assert_eq!(assistant_msg.role, MessageRole::Assistant);
            assert_eq!(assistant_msg.model, "claude-3-5-sonnet-20241022");
            assert_eq!(assistant_msg.content.len(), 1);
        }
    }

    #[test]
    fn test_parse_system_message() {
        let json = json!({
            "type": "system",
            "subtype": "initialize",
            "data": {"key": "value"}
        });

        let msg = parse_message(json).unwrap();
        assert!(matches!(msg, ParsedMessage::System(_)));

        if let ParsedMessage::System(system_msg) = msg {
            assert_eq!(system_msg.subtype, "initialize");
        }
    }

    #[test]
    fn test_parse_result_message() {
        let json = json!({
            "type": "result",
            "subtype": "query_complete",
            "duration_ms": 1000,
            "duration_api_ms": 800,
            "is_error": false,
            "num_turns": 3,
            "session_id": "session_123",
            "total_cost_usd": 0.05
        });

        let msg = parse_message(json).unwrap();
        assert!(matches!(msg, ParsedMessage::Result(_)));

        if let ParsedMessage::Result(result_msg) = msg {
            assert_eq!(result_msg.subtype, "query_complete");
            assert_eq!(result_msg.duration_ms, 1000);
            assert!(!result_msg.is_error);
            assert_eq!(result_msg.num_turns, 3);
            assert_eq!(result_msg.total_cost_usd, Some(0.05));
        }
    }

    #[test]
    fn test_parse_stream_event() {
        let json = json!({
            "type": "stream_event",
            "uuid": "evt_123",
            "session_id": "session_123",
            "event": {
                "type": "content_block_delta",
                "index": 0,
                "delta": {"type": "text_delta", "text": "Hello"}
            }
        });

        let msg = parse_message(json).unwrap();
        assert!(matches!(msg, ParsedMessage::StreamEvent(_)));

        if let ParsedMessage::StreamEvent(stream_event) = msg {
            assert_eq!(stream_event.uuid, "evt_123");
            assert_eq!(stream_event.session_id, "session_123");
            assert!(stream_event.event.is_object());
        }
    }

    #[test]
    fn test_parse_missing_type() {
        let json = json!({
            "message": {
                "content": "Hello"
            }
        });

        let result = parse_message(json);
        assert!(matches!(result, Err(MessageParseError::MissingField(_))));
    }

    #[test]
    fn test_parse_unknown_type() {
        let json = json!({
            "type": "unknown",
            "data": {}
        });

        let result = parse_message(json);
        assert!(matches!(result, Err(MessageParseError::UnknownType(_))));
    }

    #[test]
    fn test_parse_invalid_format() {
        let json = json!("not an object");

        let result = parse_message(json);
        assert!(matches!(result, Err(MessageParseError::InvalidFormat(_))));
    }

    #[test]
    fn test_parse_complex_content_blocks() {
        let json = json!({
            "type": "assistant",
            "message": {
                "content": [
                    {"type": "text", "text": "Let me use a tool"},
                    {
                        "type": "tool_use",
                        "id": "tool_123",
                        "name": "bash",
                        "input": {"command": "ls"}
                    }
                ],
                "model": "claude-3-5-sonnet-20241022"
            }
        });

        let msg = parse_message(json).unwrap();
        if let ParsedMessage::Assistant(assistant_msg) = msg {
            assert_eq!(assistant_msg.content.len(), 2);
            assert!(assistant_msg.content[0].is_text());
            assert!(assistant_msg.content[1].is_tool_use());
        } else {
            panic!("Expected assistant message");
        }
    }

    #[test]
    fn test_parse_thinking_block() {
        let json = json!({
            "type": "assistant",
            "message": {
                "content": [
                    {"type": "thinking", "thinking": "Let me think about this..."},
                    {"type": "text", "text": "Here's my answer"}
                ],
                "model": "claude-3-5-sonnet-20241022"
            }
        });

        let msg = parse_message(json).unwrap();
        if let ParsedMessage::Assistant(assistant_msg) = msg {
            assert_eq!(assistant_msg.content.len(), 2);
            assert_eq!(assistant_msg.content[0].type_name(), "thinking");
            assert!(assistant_msg.content[1].is_text());
        } else {
            panic!("Expected assistant message");
        }
    }

    #[test]
    fn test_parse_tool_result_block() {
        let json = json!({
            "type": "user",
            "message": {
                "content": [
                    {
                        "type": "tool_result",
                        "tool_use_id": "tool_123",
                        "content": "Command output",
                        "is_error": false
                    }
                ]
            }
        });

        let msg = parse_message(json).unwrap();
        if let ParsedMessage::User(user_msg) = msg {
            assert_eq!(user_msg.content.len(), 1);
            assert!(user_msg.content[0].is_tool_result());
        } else {
            panic!("Expected user message");
        }
    }

    #[test]
    fn test_parse_message_str() {
        let json_str = r#"{
            "type": "user",
            "message": {
                "content": "Hello"
            }
        }"#;

        let msg = parse_message_str(json_str).unwrap();
        assert!(matches!(msg, ParsedMessage::User(_)));
    }

    #[test]
    fn test_roundtrip_serialization() {
        // Create a user message
        let original = UserMessage {
            id: Some("msg_123".to_string()),
            message_type: "message".to_string(),
            role: MessageRole::User,
            content: vec![ContentBlock::text("Hello")],
            created_at: "2025-01-01T00:00:00Z".to_string(),
        };

        // Serialize and deserialize
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: UserMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(original, deserialized);
    }
}
