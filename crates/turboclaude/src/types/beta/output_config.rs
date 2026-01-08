//! Output Configuration Types
//!
//! Types for controlling output generation behavior including effort levels.

use serde::{Deserialize, Serialize};

/// Effort level for output generation
///
/// Controls how much effort the model expends on response generation,
/// affecting quality/length and computation cost.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputEffort {
    /// Low effort - faster, shorter responses
    Low,
    /// Medium effort - balanced responses (default)
    #[default]
    Medium,
    /// High effort - more detailed, higher quality responses
    High,
}

/// Output configuration parameters
///
/// Controls various aspects of how the model generates output.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OutputConfig {
    /// Effort level for output generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<OutputEffort>,
}

impl OutputConfig {
    /// Create a new output config with low effort
    pub fn low_effort() -> Self {
        Self {
            effort: Some(OutputEffort::Low),
        }
    }

    /// Create a new output config with medium effort
    pub fn medium_effort() -> Self {
        Self {
            effort: Some(OutputEffort::Medium),
        }
    }

    /// Create a new output config with high effort
    pub fn high_effort() -> Self {
        Self {
            effort: Some(OutputEffort::High),
        }
    }

    /// Set effort level
    pub fn with_effort(mut self, effort: OutputEffort) -> Self {
        self.effort = Some(effort);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_effort_serialization() {
        assert_eq!(
            serde_json::to_value(OutputEffort::Low).unwrap(),
            serde_json::json!("low")
        );
        assert_eq!(
            serde_json::to_value(OutputEffort::Medium).unwrap(),
            serde_json::json!("medium")
        );
        assert_eq!(
            serde_json::to_value(OutputEffort::High).unwrap(),
            serde_json::json!("high")
        );
    }

    #[test]
    fn test_output_config_creation() {
        let config = OutputConfig::high_effort();
        let json = serde_json::to_value(&config).unwrap();

        assert_eq!(json["effort"], "high");
    }

    #[test]
    fn test_output_config_deserialization() {
        let json = r#"{"effort": "low"}"#;
        let config: OutputConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.effort, Some(OutputEffort::Low));
    }

    #[test]
    fn test_output_effort_default() {
        assert_eq!(OutputEffort::default(), OutputEffort::Medium);
    }
}
