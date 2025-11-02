//! Common type definitions used across the protocol
//!
//! Includes types for models, usage statistics, cache information, and other
//! common structures used in both REST and Agent protocols.

use serde::{Deserialize, Serialize};

/// Information about token usage in a message or batch
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Usage {
    /// Number of tokens in the input
    pub input_tokens: u32,

    /// Number of tokens in the output
    pub output_tokens: u32,
}

impl Usage {
    /// Create a new usage structure
    pub fn new(input_tokens: u32, output_tokens: u32) -> Self {
        Self {
            input_tokens,
            output_tokens,
        }
    }

    /// Get total tokens (input + output)
    pub fn total_tokens(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }
}

/// Cache usage information
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CacheUsage {
    /// Tokens read from cache
    #[serde(default)]
    pub cache_read_input_tokens: u32,

    /// Tokens written to cache
    #[serde(default)]
    pub cache_creation_input_tokens: u32,
}

impl CacheUsage {
    /// Create new cache usage
    pub fn new(cache_read_input_tokens: u32, cache_creation_input_tokens: u32) -> Self {
        Self {
            cache_read_input_tokens,
            cache_creation_input_tokens,
        }
    }
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Model {
    /// The model identifier
    pub id: String,

    /// The type of model (usually "model")
    pub r#type: String,

    /// When the model was created (ISO 8601 format)
    pub created_at: String,

    /// Display name for the model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Additional metadata about the model
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub metadata: serde_json::Map<String, serde_json::Value>,
}

impl Model {
    /// Create a new model
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            r#type: "model".to_string(),
            created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            display_name: None,
            metadata: serde_json::Map::new(),
        }
    }

    /// Set the display name
    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }
}

/// Common model constants matching Anthropic's model IDs
pub mod models {
    /// Claude 3.5 Sonnet, released in October 2024.
    pub const CLAUDE_3_5_SONNET_20241022: &str = "claude-3-5-sonnet-20241022";
    /// Claude 3.5 Haiku, released in October 2024.
    pub const CLAUDE_3_5_HAIKU_20241022: &str = "claude-3-5-haiku-20241022";
    /// Claude Sonnet 4.5, released in May 2025.
    pub const CLAUDE_SONNET_4_5_20250514: &str = "claude-sonnet-4-5-20250514";
    /// Claude 3 Opus, released in February 2025.
    pub const CLAUDE_3_OPUS_20250219: &str = "claude-3-opus-20250219";
    /// Claude 3 Sonnet, released in February 2024.
    pub const CLAUDE_3_SONNET_20240229: &str = "claude-3-sonnet-20240229";
    /// Claude 3 Haiku, released in March 2024.
    pub const CLAUDE_3_HAIKU_20240307: &str = "claude-3-haiku-20240307";
}

/// Stop reason for a message completion
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StopReason {
    /// The model finished naturally (sent complete message)
    #[serde(rename = "end_turn")]
    EndTurn,

    /// The model hit the max tokens limit
    #[serde(rename = "max_tokens")]
    MaxTokens,

    /// The model is requesting to use a tool
    #[serde(rename = "tool_use")]
    ToolUse,

    /// Stop sequence was encountered
    #[serde(rename = "stop_sequence")]
    StopSequence,
}

/// Permission mode for tool use in agent sessions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionMode {
    /// Default: ask for permission for each tool use
    Default,

    /// Automatically accept edits without asking
    AcceptEdits,

    /// Bypass permission checks entirely
    BypassPermissions,
}

/// Tool definition for agent queries
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolDefinition {
    /// Name of the tool
    pub name: String,

    /// Description of what the tool does
    pub description: String,

    /// JSON schema for the tool's input
    pub input_schema: serde_json::Value,
}

impl ToolDefinition {
    /// Create a new tool definition
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_total() {
        let usage = Usage::new(100, 50);
        assert_eq!(usage.total_tokens(), 150);
    }

    #[test]
    fn test_model_creation() {
        let model = Model::new("claude-3-5-sonnet");
        assert_eq!(model.id, "claude-3-5-sonnet");
        assert_eq!(model.r#type, "model");
    }

    #[test]
    fn test_stop_reason_serialization() {
        let reasons = vec![
            StopReason::EndTurn,
            StopReason::MaxTokens,
            StopReason::ToolUse,
        ];

        for reason in reasons {
            let json = serde_json::to_string(&reason).unwrap();
            let deserialized: StopReason = serde_json::from_str(&json).unwrap();
            assert_eq!(reason, deserialized);
        }
    }
}
