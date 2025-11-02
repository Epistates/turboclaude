//! Message-related types

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{ContentBlock, ContentBlockParam, SystemPromptBlock, Tool, ToolChoice, Usage};

/// A message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique identifier for the message
    pub id: String,

    /// Type of the object (always "message")
    #[serde(rename = "type")]
    pub message_type: String,

    /// Role of the message sender
    pub role: Role,

    /// Content of the message
    pub content: Vec<ContentBlock>,

    /// Model that generated the message
    pub model: String,

    /// Stop reason if the message generation stopped
    pub stop_reason: Option<StopReason>,

    /// Stop sequence that triggered the stop
    pub stop_sequence: Option<String>,

    /// Usage statistics for the message
    pub usage: Usage,
}

impl Message {
    /// Create a user message with text content.
    pub fn user(content: impl Into<String>) -> MessageParam {
        MessageParam {
            role: Role::User,
            content: vec![ContentBlockParam::Text {
                text: content.into(),
            }],
        }
    }

    /// Create an assistant message with text content.
    pub fn assistant(content: impl Into<String>) -> MessageParam {
        MessageParam {
            role: Role::Assistant,
            content: vec![ContentBlockParam::Text {
                text: content.into(),
            }],
        }
    }

    /// Extract text content from the message.
    pub fn text(&self) -> String {
        self.content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

/// Parameters for creating a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageParam {
    /// Role of the message
    pub role: Role,

    /// Content blocks
    pub content: Vec<ContentBlockParam>,
}

/// Request parameters for creating a message.
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[builder(setter(into, strip_option))]
pub struct MessageRequest {
    /// Model to use
    pub model: String,

    /// Messages in the conversation
    pub messages: Vec<MessageParam>,

    /// Maximum tokens to generate
    pub max_tokens: u32,

    /// System prompt (string or structured blocks with cache control)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option, into))]
    pub system: Option<SystemPrompt>,

    /// Metadata for the request
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub metadata: Option<Metadata>,

    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub stop_sequences: Option<Vec<String>>,

    /// Whether to stream the response
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub stream: Option<bool>,

    /// Temperature for sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub temperature: Option<f32>,

    /// Tools available to the model
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub tools: Option<Vec<Tool>>,

    /// Tool choice preference
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub tool_choice: Option<ToolChoice>,

    /// Top-k sampling parameter
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub top_k: Option<u32>,

    /// Top-p (nucleus) sampling parameter
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub top_p: Option<f32>,

    /// User identifier for rate limiting
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub user_id: Option<String>,

    /// Extended thinking configuration (beta feature)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub thinking: Option<crate::types::beta::ThinkingConfig>,
}

impl MessageRequest {
    /// Create a builder for constructing a MessageRequest.
    pub fn builder() -> MessageRequestBuilder {
        MessageRequestBuilder::default()
    }
}

/// Role of a message sender.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// User message
    User,
    /// Assistant message
    Assistant,
}

/// Reason for stopping message generation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// Reached end of message
    EndTurn,
    /// Hit maximum token limit
    MaxTokens,
    /// Hit a stop sequence
    StopSequence,
    /// Tool use requested
    ToolUse,
}

impl StopReason {
    /// Get the string representation of this stop reason.
    pub fn as_str(&self) -> &str {
        match self {
            StopReason::EndTurn => "end_turn",
            StopReason::MaxTokens => "max_tokens",
            StopReason::StopSequence => "stop_sequence",
            StopReason::ToolUse => "tool_use",
        }
    }
}

/// System prompt (string or structured blocks with cache control).
///
/// Can be either a simple string or a vector of blocks with cache control.
/// See [Prompt Caching](https://docs.anthropic.com/en/docs/build-with-claude/prompt-caching).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum SystemPrompt {
    /// Simple string system prompt
    String(String),
    /// Structured blocks with cache control
    Blocks(Vec<SystemPromptBlock>),
}

impl From<String> for SystemPrompt {
    fn from(s: String) -> Self {
        SystemPrompt::String(s)
    }
}

impl From<&str> for SystemPrompt {
    fn from(s: &str) -> Self {
        SystemPrompt::String(s.to_string())
    }
}

impl From<Vec<SystemPromptBlock>> for SystemPrompt {
    fn from(blocks: Vec<SystemPromptBlock>) -> Self {
        SystemPrompt::Blocks(blocks)
    }
}

/// Metadata for a request.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Metadata {
    /// User-defined metadata
    #[serde(flatten)]
    pub data: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CacheControl, SystemPromptBlock};

    #[test]
    fn test_message_user_creation() {
        let msg = Message::user("Hello, Claude!");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.content.len(), 1);

        match &msg.content[0] {
            ContentBlockParam::Text { text } => {
                assert_eq!(text, "Hello, Claude!");
            }
            _ => panic!("Expected text content block"),
        }
    }

    #[test]
    fn test_message_assistant_creation() {
        let msg = Message::assistant("Hello! How can I help?");
        assert_eq!(msg.role, Role::Assistant);
        assert_eq!(msg.content.len(), 1);

        match &msg.content[0] {
            ContentBlockParam::Text { text } => {
                assert_eq!(text, "Hello! How can I help?");
            }
            _ => panic!("Expected text content block"),
        }
    }

    #[test]
    fn test_content_block_text() {
        let block = ContentBlockParam::Text {
            text: "Test message".to_string(),
        };

        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "Test message");
    }

    #[test]
    fn test_content_block_tool_use() {
        use serde_json::json;

        let block = ContentBlockParam::ToolResult {
            tool_use_id: "tool_123".to_string(),
            content: json!({"result": 42}).to_string(),
            is_error: None,
        };

        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["type"], "tool_result");
        assert_eq!(json["tool_use_id"], "tool_123");
        assert!(json.get("is_error").is_none());
    }

    #[test]
    fn test_content_block_tool_result() {
        let block_success = ContentBlockParam::ToolResult {
            tool_use_id: "tool_456".to_string(),
            content: "Success result".to_string(),
            is_error: Some(false),
        };

        let json = serde_json::to_value(&block_success).unwrap();
        assert_eq!(json["type"], "tool_result");
        assert_eq!(json["content"], "Success result");
        assert_eq!(json["is_error"], false);

        let block_error = ContentBlockParam::ToolResult {
            tool_use_id: "tool_789".to_string(),
            content: "Error occurred".to_string(),
            is_error: Some(true),
        };

        let json_error = serde_json::to_value(&block_error).unwrap();
        assert_eq!(json_error["is_error"], true);
        assert_eq!(json_error["content"], "Error occurred");
    }

    #[test]
    fn test_cache_control_serialization() {
        use crate::types::CacheTTL;

        // Default ephemeral (no TTL)
        let cache = CacheControl::ephemeral();
        let json = serde_json::to_value(&cache).unwrap();
        assert_eq!(json["type"], "ephemeral");
        assert!(json.get("ttl").is_none());

        // With 5m TTL
        let cache_5m = CacheControl::ephemeral_with_ttl(CacheTTL::FiveMinutes);
        let json = serde_json::to_value(&cache_5m).unwrap();
        assert_eq!(json["type"], "ephemeral");
        assert_eq!(json["ttl"], "5m");

        // With 1h TTL
        let cache_1h = CacheControl::ephemeral_with_ttl(CacheTTL::OneHour);
        let json = serde_json::to_value(&cache_1h).unwrap();
        assert_eq!(json["ttl"], "1h");
    }

    #[test]
    fn test_system_prompt_string() {
        let prompt = SystemPrompt::from("You are a helpful assistant.");

        let json = serde_json::to_value(&prompt).unwrap();
        assert_eq!(json, "You are a helpful assistant.");

        // Test from &str
        let prompt_ref = SystemPrompt::from("Another prompt");
        let json_ref = serde_json::to_value(&prompt_ref).unwrap();
        assert_eq!(json_ref, "Another prompt");
    }

    #[test]
    fn test_system_prompt_blocks() {
        let blocks = vec![
            SystemPromptBlock::text("Part 1: Instructions"),
            SystemPromptBlock::text_cached("Part 2: Cached context"),
        ];

        let prompt = SystemPrompt::from(blocks);
        let json = serde_json::to_value(&prompt).unwrap();

        assert!(json.is_array());
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), 2);

        // First block - no cache control
        assert_eq!(arr[0]["type"], "text");
        assert_eq!(arr[0]["text"], "Part 1: Instructions");
        assert!(arr[0].get("cache_control").is_none());

        // Second block - with cache control
        assert_eq!(arr[1]["type"], "text");
        assert_eq!(arr[1]["text"], "Part 2: Cached context");
        assert_eq!(arr[1]["cache_control"]["type"], "ephemeral");
    }

    #[test]
    fn test_stop_reason_serialization() {
        let end_turn = StopReason::EndTurn;
        let json = serde_json::to_string(&end_turn).unwrap();
        assert_eq!(json, "\"end_turn\"");
        assert_eq!(end_turn.as_str(), "end_turn");

        let max_tokens = StopReason::MaxTokens;
        let json = serde_json::to_string(&max_tokens).unwrap();
        assert_eq!(json, "\"max_tokens\"");
        assert_eq!(max_tokens.as_str(), "max_tokens");

        let stop_seq = StopReason::StopSequence;
        let json = serde_json::to_string(&stop_seq).unwrap();
        assert_eq!(json, "\"stop_sequence\"");

        let tool_use = StopReason::ToolUse;
        let json = serde_json::to_string(&tool_use).unwrap();
        assert_eq!(json, "\"tool_use\"");
    }

    #[test]
    fn test_role_serialization() {
        let user = Role::User;
        let json = serde_json::to_string(&user).unwrap();
        assert_eq!(json, "\"user\"");

        let assistant = Role::Assistant;
        let json = serde_json::to_string(&assistant).unwrap();
        assert_eq!(json, "\"assistant\"");

        // Test deserialization
        let user_deser: Role = serde_json::from_str("\"user\"").unwrap();
        assert_eq!(user_deser, Role::User);

        let assistant_deser: Role = serde_json::from_str("\"assistant\"").unwrap();
        assert_eq!(assistant_deser, Role::Assistant);
    }

    #[test]
    fn test_message_request_with_system_prompt() {
        use crate::types::Models;

        let request = MessageRequest::builder()
            .model(Models::CLAUDE_3_5_SONNET)
            .max_tokens(1024u32)
            .messages(vec![Message::user("Hello")])
            .system("You are a helpful assistant")
            .build()
            .unwrap();

        assert!(request.system.is_some());
        match request.system.unwrap() {
            SystemPrompt::String(s) => assert_eq!(s, "You are a helpful assistant"),
            _ => panic!("Expected string system prompt"),
        }
    }

    #[test]
    fn test_message_request_with_tools() {
        use crate::types::{Models, Tool};
        use serde_json::json;

        let tool = Tool {
            name: "calculator".to_string(),
            description: "A calculator tool".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "Math expression"
                    }
                },
                "required": ["expression"]
            }),
        };

        let request = MessageRequest::builder()
            .model(Models::CLAUDE_3_5_SONNET)
            .max_tokens(1024u32)
            .messages(vec![Message::user("Calculate 2+2")])
            .tools(vec![tool])
            .build()
            .unwrap();

        assert!(request.tools.is_some());
        let tools = request.tools.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "calculator");
    }

    #[test]
    fn test_message_request_stream_flag() {
        use crate::types::Models;

        let request = MessageRequest::builder()
            .model(Models::CLAUDE_3_5_SONNET)
            .max_tokens(1024u32)
            .messages(vec![Message::user("Hello")])
            .stream(true)
            .build()
            .unwrap();

        assert_eq!(request.stream, Some(true));

        // Test serialization includes stream
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["stream"], true);
    }

    #[test]
    fn test_metadata_creation() {
        use serde_json::json;
        use std::collections::HashMap;

        let mut data = HashMap::new();
        data.insert("user_id".to_string(), json!("user_123"));
        data.insert("request_type".to_string(), json!("analysis"));

        let metadata = Metadata { data };

        let json = serde_json::to_value(&metadata).unwrap();
        assert_eq!(json["user_id"], "user_123");
        assert_eq!(json["request_type"], "analysis");
    }
}
