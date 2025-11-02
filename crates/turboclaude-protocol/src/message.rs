//! Message types for the protocol
//!
//! Defines the message structures used in both REST API and Agent protocol.
//! Matches the Anthropic Messages API format.

use crate::content::ContentBlock;
use crate::types::{CacheUsage, StopReason, Usage};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    /// Unique identifier for the message
    pub id: String,

    /// Type of message (always "message")
    #[serde(rename = "type")]
    pub message_type: String,

    /// The role that produced the message
    pub role: MessageRole,

    /// The content blocks in the message
    pub content: Vec<ContentBlock>,

    /// The model used to generate this message
    pub model: String,

    /// Why the model stopped generating
    pub stop_reason: StopReason,

    /// Additional stop sequences if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,

    /// When the message was created
    pub created_at: String,

    /// Token usage information
    pub usage: Usage,

    /// Cache usage information
    #[serde(default, skip_serializing_if = "is_zero_cache")]
    pub cache_usage: CacheUsage,
}

/// A user message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserMessage {
    /// Unique identifier for the message
    pub id: Option<String>,

    /// Type of message (always "message")
    #[serde(rename = "type", default = "default_user_type")]
    pub message_type: String,

    /// The role (always "user")
    pub role: MessageRole,

    /// The content blocks in the message
    pub content: Vec<ContentBlock>,

    /// When the message was created
    #[serde(default = "default_timestamp")]
    pub created_at: String,
}

/// An assistant message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssistantMessage {
    /// Unique identifier for the message
    pub id: String,

    /// Type of message (always "message")
    #[serde(rename = "type", default = "default_assistant_type")]
    pub message_type: String,

    /// The role (always "assistant")
    pub role: MessageRole,

    /// The content blocks in the message
    pub content: Vec<ContentBlock>,

    /// The model used to generate this message
    pub model: String,

    /// Why the model stopped generating
    pub stop_reason: StopReason,

    /// When the message was created
    #[serde(default = "default_timestamp")]
    pub created_at: String,

    /// Token usage information
    pub usage: Usage,

    /// Cache usage information
    #[serde(default, skip_serializing_if = "is_zero_cache")]
    pub cache_usage: CacheUsage,
}

/// A system message from the Claude CLI
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemMessage {
    /// Subtype of the system message
    pub subtype: String,

    /// Raw data from the system message
    pub data: serde_json::Value,
}

/// A result message indicating query completion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResultMessage {
    /// Subtype of the result message
    pub subtype: String,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// API duration in milliseconds
    pub duration_api_ms: u64,

    /// Whether the result is an error
    pub is_error: bool,

    /// Number of turns in the conversation
    pub num_turns: u32,

    /// Session identifier
    pub session_id: String,

    /// Total cost in USD (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cost_usd: Option<f64>,

    /// Token usage information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<serde_json::Value>,

    /// Result data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
}

/// A stream event for partial message updates during streaming
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StreamEvent {
    /// Unique identifier for the event
    pub uuid: String,

    /// Session identifier
    pub session_id: String,

    /// The raw Anthropic API stream event
    pub event: serde_json::Value,

    /// Parent tool use ID (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_tool_use_id: Option<String>,
}

/// Message role
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// User message
    User,

    /// Assistant message
    Assistant,
}

/// Request to create a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRequest {
    /// The model to use
    pub model: String,

    /// Maximum tokens to generate
    pub max_tokens: u32,

    /// System prompt (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,

    /// Messages in the conversation
    pub messages: Vec<MessageParameter>,

    /// Tools available to the model (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<serde_json::Value>>,

    /// Tool choice (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<serde_json::Value>,

    /// Stop sequences (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,

    /// Temperature for sampling (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Top P for nucleus sampling (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Top K for sampling (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,

    /// Whether to use extended thinking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<serde_json::Value>,

    /// Metadata for the request
    #[serde(default, skip_serializing_if = "is_empty_metadata")]
    pub metadata: serde_json::Map<String, serde_json::Value>,
}

/// A message parameter (user or assistant message)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageParameter {
    /// Message role
    pub role: MessageRole,

    /// Message content
    pub content: Vec<ContentBlock>,
}

impl Message {
    /// Create a new message
    pub fn new(model: impl Into<String>, role: MessageRole, content: Vec<ContentBlock>) -> Self {
        Self {
            id: format!("msg_{}", Uuid::new_v4()),
            message_type: "message".to_string(),
            role,
            content,
            model: model.into(),
            stop_reason: StopReason::EndTurn,
            stop_sequence: None,
            created_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            usage: Usage::new(0, 0),
            cache_usage: CacheUsage::default(),
        }
    }

    /// Get all text content from the message
    pub fn get_text_content(&self) -> String {
        self.content
            .iter()
            .filter_map(|block| block.as_text())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get all tool uses from the message
    pub fn get_tool_uses(&self) -> Vec<(&str, &str, &serde_json::Value)> {
        self.content
            .iter()
            .filter_map(|block| block.as_tool_use())
            .collect()
    }

    /// Check if message ended due to tool use
    pub fn used_tools(&self) -> bool {
        self.stop_reason == StopReason::ToolUse
    }

    /// Check if message is complete
    pub fn is_complete(&self) -> bool {
        self.stop_reason == StopReason::EndTurn
    }
}

impl UserMessage {
    /// Create a new user message
    pub fn new(content: Vec<ContentBlock>) -> Self {
        Self {
            id: Some(format!("msg_{}", Uuid::new_v4())),
            message_type: "message".to_string(),
            role: MessageRole::User,
            content,
            created_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        }
    }

    /// Create a user message from text
    pub fn text(text: impl Into<String>) -> Self {
        Self::new(vec![ContentBlock::text(text)])
    }
}

impl AssistantMessage {
    /// Create a new assistant message
    pub fn new(model: impl Into<String>, content: Vec<ContentBlock>, usage: Usage) -> Self {
        Self {
            id: format!("msg_{}", Uuid::new_v4()),
            message_type: "message".to_string(),
            role: MessageRole::Assistant,
            content,
            model: model.into(),
            stop_reason: StopReason::EndTurn,
            created_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            usage,
            cache_usage: CacheUsage::default(),
        }
    }
}

impl SystemMessage {
    /// Create a new system message
    pub fn new(subtype: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            subtype: subtype.into(),
            data,
        }
    }
}

impl ResultMessage {
    /// Create a new result message
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        subtype: impl Into<String>,
        duration_ms: u64,
        duration_api_ms: u64,
        is_error: bool,
        num_turns: u32,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            subtype: subtype.into(),
            duration_ms,
            duration_api_ms,
            is_error,
            num_turns,
            session_id: session_id.into(),
            total_cost_usd: None,
            usage: None,
            result: None,
        }
    }

    /// Set the total cost
    pub fn with_cost(mut self, cost: f64) -> Self {
        self.total_cost_usd = Some(cost);
        self
    }

    /// Set the usage information
    pub fn with_usage(mut self, usage: serde_json::Value) -> Self {
        self.usage = Some(usage);
        self
    }

    /// Set the result data
    pub fn with_result(mut self, result: impl Into<String>) -> Self {
        self.result = Some(result.into());
        self
    }
}

impl StreamEvent {
    /// Create a new stream event
    pub fn new(
        uuid: impl Into<String>,
        session_id: impl Into<String>,
        event: serde_json::Value,
    ) -> Self {
        Self {
            uuid: uuid.into(),
            session_id: session_id.into(),
            event,
            parent_tool_use_id: None,
        }
    }

    /// Set the parent tool use ID
    pub fn with_parent_tool_use_id(mut self, parent_tool_use_id: impl Into<String>) -> Self {
        self.parent_tool_use_id = Some(parent_tool_use_id.into());
        self
    }
}

// Helper functions for serde defaults
fn default_user_type() -> String {
    "message".to_string()
}

fn default_assistant_type() -> String {
    "message".to_string()
}

fn default_timestamp() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn is_zero_cache(cache: &CacheUsage) -> bool {
    cache.cache_creation_input_tokens == 0 && cache.cache_read_input_tokens == 0
}

fn is_empty_metadata(meta: &serde_json::Map<String, serde_json::Value>) -> bool {
    meta.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_message_creation() {
        let msg = UserMessage::text("Hello");
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content.len(), 1);
    }

    #[test]
    fn test_message_get_text() {
        let msg = Message::new(
            "claude-3-5-sonnet",
            MessageRole::Assistant,
            vec![ContentBlock::text("Hello world")],
        );
        assert_eq!(msg.get_text_content(), "Hello world");
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::new(
            "claude-3-5-sonnet",
            MessageRole::Assistant,
            vec![ContentBlock::text("Test")],
        );
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, deserialized);
    }
}
