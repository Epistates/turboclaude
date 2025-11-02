//! Extended thinking types and context management
//!
//! Supports extended thinking configuration, thinking blocks, and context management
//! including the ability to clear old thinking blocks from conversation history.

use serde::{Deserialize, Serialize};

/// Thinking block returned in message content when extended thinking is enabled
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThinkingBlock {
    /// Signature identifying the thinking block
    pub signature: String,

    /// The model's reasoning/thinking process
    pub thinking: String,

    /// Block type (always "thinking")
    #[serde(rename = "type")]
    pub block_type: String,
}

/// Configuration for enabling extended thinking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingConfig {
    /// Token budget allocated for reasoning (must be ≥ 1024 and < max_tokens)
    pub budget_tokens: u32,

    /// Config type (always "enabled")
    #[serde(rename = "type")]
    pub config_type: String,
}

/// Parameter specifying number of recent thinking turns to keep
///
/// Used with `BetaClearThinking20251015EditParam` to preserve recent thinking
/// while clearing older thinking blocks from the conversation.
///
/// # Example
/// ```json
/// {
///   "type": "thinking_turns",
///   "value": 5
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetaThinkingTurnsParam {
    /// Parameter type (always "thinking_turns")
    #[serde(rename = "type")]
    pub param_type: String,

    /// Number of recent assistant turns to keep thinking blocks for
    pub value: u32,
}

impl BetaThinkingTurnsParam {
    /// Create a new thinking turns parameter
    ///
    /// # Arguments
    /// * `value` - Number of recent turns to keep thinking blocks for
    ///
    /// # Example
    /// ```rust
    /// use turboclaude::types::beta::BetaThinkingTurnsParam;
    ///
    /// let param = BetaThinkingTurnsParam::new(5);
    /// ```
    pub fn new(value: u32) -> Self {
        Self {
            param_type: "thinking_turns".to_string(),
            value,
        }
    }
}

/// Parameter to keep all thinking blocks
///
/// Used with `BetaClearThinking20251015EditParam` to preserve all thinking
/// blocks when clearing context.
///
/// # Example
/// ```json
/// {
///   "type": "all"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetaAllThinkingTurnsParam {
    /// Parameter type (always "all")
    #[serde(rename = "type")]
    pub param_type: String,
}

impl BetaAllThinkingTurnsParam {
    /// Create a new all thinking turns parameter
    pub fn new() -> Self {
        Self {
            param_type: "all".to_string(),
        }
    }
}

impl Default for BetaAllThinkingTurnsParam {
    fn default() -> Self {
        Self::new()
    }
}

/// Union type for specifying which thinking turns to keep during clearing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Keep {
    /// Keep specific number of recent thinking turns
    ThinkingTurns(BetaThinkingTurnsParam),
    /// Keep all thinking turns
    AllThinking(BetaAllThinkingTurnsParam),
    /// Keep all (string shorthand)
    All(String),
}

/// Request to clear old thinking blocks from conversation history
///
/// Allows selective removal of thinking blocks while preserving recent reasoning.
/// This is useful for managing token usage in long-running conversations.
///
/// # Example
/// ```rust
/// use turboclaude::types::beta::{BetaClearThinking20251015EditParam, BetaThinkingTurnsParam, Keep};
///
/// let clear_param = BetaClearThinking20251015EditParam {
///     param_type: "clear_thinking_20251015".to_string(),
///     keep: Some(Keep::ThinkingTurns(BetaThinkingTurnsParam::new(5))),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetaClearThinking20251015EditParam {
    /// Edit type (always "clear_thinking_20251015")
    #[serde(rename = "type")]
    pub param_type: String,

    /// Which thinking turns to keep (older turns will be cleared)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep: Option<Keep>,
}

impl BetaClearThinking20251015EditParam {
    /// Create a new clear thinking parameter
    ///
    /// # Arguments
    /// * `keep` - Which thinking turns to preserve
    pub fn new(keep: Option<Keep>) -> Self {
        Self {
            param_type: "clear_thinking_20251015".to_string(),
            keep,
        }
    }

    /// Create a clear thinking parameter keeping specific number of turns
    pub fn with_turns(turns: u32) -> Self {
        Self::new(Some(Keep::ThinkingTurns(BetaThinkingTurnsParam::new(
            turns,
        ))))
    }

    /// Create a clear thinking parameter keeping all thinking turns
    pub fn keep_all() -> Self {
        Self::new(Some(Keep::AllThinking(BetaAllThinkingTurnsParam::new())))
    }

    /// Create a clear thinking parameter clearing all thinking blocks
    pub fn clear_all() -> Self {
        Self::new(None)
    }
}

/// Response indicating successful clearing of thinking blocks
///
/// Provides feedback on tokens and turns that were cleared.
///
/// # Example
/// ```json
/// {
///   "type": "clear_thinking_20251015",
///   "cleared_input_tokens": 1024,
///   "cleared_thinking_turns": 3
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetaClearThinking20251015EditResponse {
    /// Number of input tokens cleared
    pub cleared_input_tokens: u32,

    /// Number of thinking turns cleared
    pub cleared_thinking_turns: u32,

    /// Response type (always "clear_thinking_20251015")
    #[serde(rename = "type")]
    pub response_type: String,
}

impl ThinkingConfig {
    /// Create a new thinking configuration with the specified token budget
    ///
    /// # Arguments
    ///
    /// * `budget_tokens` - Number of tokens to allocate for thinking (minimum 1024)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use turboclaude::types::beta::ThinkingConfig;
    ///
    /// let config = ThinkingConfig::new(1600);
    /// ```
    pub fn new(budget_tokens: u32) -> Self {
        Self {
            budget_tokens,
            config_type: "enabled".to_string(),
        }
    }

    /// Validate that budget_tokens meets minimum requirement
    pub fn validate(&self) -> Result<(), String> {
        if self.budget_tokens < 1024 {
            return Err(format!(
                "budget_tokens must be at least 1024, got {}",
                self.budget_tokens
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== ThinkingConfig Tests =====

    #[test]
    fn test_thinking_config_new() {
        let config = ThinkingConfig::new(1600);
        assert_eq!(config.budget_tokens, 1600);
        assert_eq!(config.config_type, "enabled");
    }

    #[test]
    fn test_thinking_config_validation_pass() {
        let config = ThinkingConfig::new(1024);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_thinking_config_validation_fail() {
        let config = ThinkingConfig::new(1023);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_thinking_config_serialization() {
        let config = ThinkingConfig::new(2000);
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"budget_tokens\":2000"));
        assert!(json.contains("\"type\":\"enabled\""));
    }

    // ===== BetaThinkingTurnsParam Tests =====

    #[test]
    fn test_beta_thinking_turns_param_new() {
        let param = BetaThinkingTurnsParam::new(5);
        assert_eq!(param.value, 5);
        assert_eq!(param.param_type, "thinking_turns");
    }

    #[test]
    fn test_beta_thinking_turns_param_serialization() {
        let param = BetaThinkingTurnsParam::new(3);
        let json = serde_json::to_string(&param).unwrap();
        assert!(json.contains("\"type\":\"thinking_turns\""));
        assert!(json.contains("\"value\":3"));
    }

    #[test]
    fn test_beta_thinking_turns_param_deserialization() {
        let json = r#"{"type":"thinking_turns","value":10}"#;
        let param: BetaThinkingTurnsParam = serde_json::from_str(json).unwrap();
        assert_eq!(param.value, 10);
        assert_eq!(param.param_type, "thinking_turns");
    }

    // ===== BetaAllThinkingTurnsParam Tests =====

    #[test]
    fn test_beta_all_thinking_turns_param_new() {
        let param = BetaAllThinkingTurnsParam::new();
        assert_eq!(param.param_type, "all");
    }

    #[test]
    fn test_beta_all_thinking_turns_param_default() {
        let param = BetaAllThinkingTurnsParam::default();
        assert_eq!(param.param_type, "all");
    }

    #[test]
    fn test_beta_all_thinking_turns_param_serialization() {
        let param = BetaAllThinkingTurnsParam::new();
        let json = serde_json::to_string(&param).unwrap();
        assert!(json.contains("\"type\":\"all\""));
    }

    #[test]
    fn test_beta_all_thinking_turns_param_deserialization() {
        let json = r#"{"type":"all"}"#;
        let param: BetaAllThinkingTurnsParam = serde_json::from_str(json).unwrap();
        assert_eq!(param.param_type, "all");
    }

    // ===== BetaClearThinking20251015EditParam Tests =====

    #[test]
    fn test_beta_clear_thinking_new() {
        let param = BetaClearThinking20251015EditParam::new(None);
        assert_eq!(param.param_type, "clear_thinking_20251015");
        assert!(param.keep.is_none());
    }

    #[test]
    fn test_beta_clear_thinking_with_turns() {
        let param = BetaClearThinking20251015EditParam::with_turns(5);
        assert_eq!(param.param_type, "clear_thinking_20251015");
        assert!(param.keep.is_some());
    }

    #[test]
    fn test_beta_clear_thinking_keep_all() {
        let param = BetaClearThinking20251015EditParam::keep_all();
        assert_eq!(param.param_type, "clear_thinking_20251015");
        assert!(param.keep.is_some());
    }

    #[test]
    fn test_beta_clear_thinking_clear_all() {
        let param = BetaClearThinking20251015EditParam::clear_all();
        assert_eq!(param.param_type, "clear_thinking_20251015");
        assert!(param.keep.is_none());
    }

    #[test]
    fn test_beta_clear_thinking_serialization_with_turns() {
        let param = BetaClearThinking20251015EditParam::with_turns(7);
        let json = serde_json::to_string(&param).unwrap();
        assert!(json.contains("\"type\":\"clear_thinking_20251015\""));
    }

    #[test]
    fn test_beta_clear_thinking_serialization_clear_all() {
        let param = BetaClearThinking20251015EditParam::clear_all();
        let json = serde_json::to_string(&param).unwrap();
        assert!(json.contains("\"type\":\"clear_thinking_20251015\""));
        // keep should not be present when None
        assert!(!json.contains("\"keep\""));
    }

    // ===== BetaClearThinking20251015EditResponse Tests =====

    #[test]
    fn test_beta_clear_thinking_response_creation() {
        let response = BetaClearThinking20251015EditResponse {
            cleared_input_tokens: 1024,
            cleared_thinking_turns: 3,
            response_type: "clear_thinking_20251015".to_string(),
        };
        assert_eq!(response.cleared_input_tokens, 1024);
        assert_eq!(response.cleared_thinking_turns, 3);
    }

    #[test]
    fn test_beta_clear_thinking_response_serialization() {
        let response = BetaClearThinking20251015EditResponse {
            cleared_input_tokens: 2048,
            cleared_thinking_turns: 5,
            response_type: "clear_thinking_20251015".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"cleared_input_tokens\":2048"));
        assert!(json.contains("\"cleared_thinking_turns\":5"));
        assert!(json.contains("\"type\":\"clear_thinking_20251015\""));
    }

    #[test]
    fn test_beta_clear_thinking_response_deserialization() {
        let json = r#"{"cleared_input_tokens":512,"cleared_thinking_turns":2,"type":"clear_thinking_20251015"}"#;
        let response: BetaClearThinking20251015EditResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.cleared_input_tokens, 512);
        assert_eq!(response.cleared_thinking_turns, 2);
        assert_eq!(response.response_type, "clear_thinking_20251015");
    }

    // ===== Integration Tests =====

    #[test]
    fn test_keep_enum_thinking_turns_variant() {
        let param = BetaThinkingTurnsParam::new(4);
        let keep = Keep::ThinkingTurns(param);
        let json = serde_json::to_string(&keep).unwrap();
        assert!(json.contains("\"type\":\"thinking_turns\""));
        assert!(json.contains("\"value\":4"));
    }

    #[test]
    fn test_keep_enum_all_variant() {
        let param = BetaAllThinkingTurnsParam::new();
        let keep = Keep::AllThinking(param);
        let json = serde_json::to_string(&keep).unwrap();
        assert!(json.contains("\"type\":\"all\""));
    }

    #[test]
    fn test_clear_thinking_workflow() {
        // Scenario: Clear thinking but keep last 3 turns
        let clear_param = BetaClearThinking20251015EditParam::with_turns(3);
        assert_eq!(clear_param.param_type, "clear_thinking_20251015");

        // Simulate response
        let response = BetaClearThinking20251015EditResponse {
            cleared_input_tokens: 1500,
            cleared_thinking_turns: 7,
            response_type: "clear_thinking_20251015".to_string(),
        };

        assert!(response.cleared_input_tokens > 0);
        assert!(response.cleared_thinking_turns > 0);
    }
}
