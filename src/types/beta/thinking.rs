//! Extended thinking types

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
}
