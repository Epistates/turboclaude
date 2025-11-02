//! Message-related types

use serde::{Deserialize, Serialize};
use derive_builder::Builder;
use std::collections::HashMap;

use super::{ContentBlock, ContentBlockParam, Usage, SystemPromptBlock, Tool, ToolChoice};

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
                ContentBlock::Text { text } => Some(text.as_str()),
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
