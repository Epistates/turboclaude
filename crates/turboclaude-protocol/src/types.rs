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
///
/// This module provides constants for all available Claude models,
/// organized by generation and capability tier.
pub mod models {
    // ========================================================================
    // LATEST GENERATION - RECOMMENDED FOR PRODUCTION
    // ========================================================================

    /// Claude Sonnet 4.5 (September 2025) - **RECOMMENDED**
    ///
    /// Best coding model in the world. Strongest for building complex agents,
    /// best at computer use. Excellent at following instructions and writing code.
    ///
    /// **Pricing:** $3 per million input tokens, $15 per million output tokens
    ///
    /// **Use cases:** Complex coding, agent workflows, computer use, instruction following
    pub const CLAUDE_SONNET_4_5_20250929: &str = "claude-sonnet-4-5-20250929";

    /// Claude Sonnet 4.5 with Structured Outputs (September 2025)
    ///
    /// Same capabilities as CLAUDE_SONNET_4_5_20250929 but optimized for
    /// returning structured JSON outputs that conform to provided schemas.
    ///
    /// **Requires:** Beta header `structured-outputs-2025-09-17`
    ///
    /// **Use cases:** Type-safe API responses, data extraction, form filling
    pub const CLAUDE_SONNET_4_5_20250929_STRUCTURED_OUTPUTS: &str =
        "claude-sonnet-4-5-20250929-structured-outputs";

    /// Claude Haiku 4.5 (October 2025) - **RECOMMENDED FOR SPEED**
    ///
    /// Small, fast model optimized for low latency. Near-frontier coding quality,
    /// matches Sonnet 4 on coding, surpasses Sonnet 4 on some computer-use tasks.
    ///
    /// **Pricing:** $1 per million input tokens, $5 per million output tokens
    ///
    /// **Use cases:** Fast responses, cost optimization, high-volume requests
    pub const CLAUDE_HAIKU_4_5_20251001: &str = "claude-haiku-4-5-20251001";

    /// Claude Opus 4.1 (August 2025)
    ///
    /// Most capable model for complex reasoning and analysis. Improved agentic tasks,
    /// coding, and reasoning. Scores 74.5% on SWE-bench Verified.
    ///
    /// **Pricing:** $15 per million input tokens, $75 per million output tokens
    ///
    /// **Use cases:** Complex analysis, research, highest-quality output
    pub const CLAUDE_OPUS_4_1_20250805: &str = "claude-opus-4-1-20250805";

    // ========================================================================
    // CONVENIENCE ALIASES
    // ========================================================================

    /// Alias for the latest Sonnet model (currently 4.5)
    ///
    /// **Note:** This alias may point to different models over time as new versions release.
    /// Use specific version constants for reproducible behavior.
    pub const SONNET_LATEST: &str = CLAUDE_SONNET_4_5_20250929;

    /// Alias for the latest Haiku model (currently 4.5)
    pub const HAIKU_LATEST: &str = CLAUDE_HAIKU_4_5_20251001;

    /// Alias for the latest Opus model (currently 4.1)
    pub const OPUS_LATEST: &str = CLAUDE_OPUS_4_1_20250805;

    /// Default model for new applications
    pub const DEFAULT: &str = CLAUDE_SONNET_4_5_20250929;

    // ========================================================================
    // LEGACY GENERATION (backward compatibility)
    // ========================================================================

    /// Claude 3.5 Sonnet (October 2024)
    ///
    /// **Deprecated:** Use [`CLAUDE_SONNET_4_5_20250929`] instead for better performance.
    #[deprecated(since = "0.2.0", note = "Use CLAUDE_SONNET_4_5_20250929 instead")]
    pub const CLAUDE_3_5_SONNET_20241022: &str = "claude-3-5-sonnet-20241022";

    /// Claude 3.5 Haiku (October 2024)
    ///
    /// **Deprecated:** Use [`CLAUDE_HAIKU_4_5_20251001`] instead for better performance.
    #[deprecated(since = "0.2.0", note = "Use CLAUDE_HAIKU_4_5_20251001 instead")]
    pub const CLAUDE_3_5_HAIKU_20241022: &str = "claude-3-5-haiku-20241022";

    /// Claude Sonnet 4.5 (May 2025)
    ///
    /// **Deprecated:** Use [`CLAUDE_SONNET_4_5_20250929`] instead for latest improvements.
    #[deprecated(since = "0.2.0", note = "Use CLAUDE_SONNET_4_5_20250929 instead")]
    pub const CLAUDE_SONNET_4_5_20250514: &str = "claude-sonnet-4-5-20250514";

    /// Claude 3 Opus (February 2024)
    ///
    /// **Deprecated:** Use [`CLAUDE_OPUS_4_1_20250805`] instead.
    #[deprecated(since = "0.2.0", note = "Use CLAUDE_OPUS_4_1_20250805 instead")]
    pub const CLAUDE_3_OPUS_20240229: &str = "claude-3-opus-20240229";

    /// Claude 3 Sonnet (February 2024)
    ///
    /// **Deprecated:** Use [`CLAUDE_SONNET_4_5_20250929`] instead.
    #[deprecated(since = "0.2.0", note = "Use CLAUDE_SONNET_4_5_20250929 instead")]
    pub const CLAUDE_3_SONNET_20240229: &str = "claude-3-sonnet-20240229";

    /// Claude 3 Haiku (March 2024)
    ///
    /// **Deprecated:** Use [`CLAUDE_HAIKU_4_5_20251001`] instead.
    #[deprecated(since = "0.2.0", note = "Use CLAUDE_HAIKU_4_5_20251001 instead")]
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
