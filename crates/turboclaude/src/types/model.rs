//! Model-related types

use serde::{Deserialize, Serialize};

/// Information about a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    /// Unique identifier for the model
    pub id: String,

    /// Type of the object (always "model")
    #[serde(rename = "type")]
    pub model_type: String,

    /// Display name of the model
    pub display_name: String,

    /// When the model was created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Predefined model identifiers.
pub struct Models;

impl Models {
    /// Claude 3.5 Sonnet (October 2024)
    pub const CLAUDE_3_5_SONNET: &'static str = "claude-3-5-sonnet-20241022";

    /// Claude 3.5 Haiku (October 2024)
    pub const CLAUDE_3_5_HAIKU: &'static str = "claude-3-5-haiku-20241022";

    /// Claude 3 Opus (February 2024)
    pub const CLAUDE_3_OPUS: &'static str = "claude-3-opus-20240229";

    /// Claude 3 Sonnet (February 2024)
    pub const CLAUDE_3_SONNET: &'static str = "claude-3-sonnet-20240229";

    /// Claude 3 Haiku (March 2024)
    pub const CLAUDE_3_HAIKU: &'static str = "claude-3-haiku-20240307";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_enum_variants() {
        // Verify all model constants exist and are non-empty
        assert!(!Models::CLAUDE_3_5_SONNET.is_empty());
        assert!(!Models::CLAUDE_3_5_HAIKU.is_empty());
        assert!(!Models::CLAUDE_3_OPUS.is_empty());
        assert!(!Models::CLAUDE_3_SONNET.is_empty());
        assert!(!Models::CLAUDE_3_HAIKU.is_empty());

        // Verify they follow the expected format
        assert!(Models::CLAUDE_3_5_SONNET.starts_with("claude-"));
        assert!(Models::CLAUDE_3_5_HAIKU.starts_with("claude-"));
        assert!(Models::CLAUDE_3_OPUS.starts_with("claude-"));
        assert!(Models::CLAUDE_3_SONNET.starts_with("claude-"));
        assert!(Models::CLAUDE_3_HAIKU.starts_with("claude-"));

        // Verify they're distinct
        let models = vec![
            Models::CLAUDE_3_5_SONNET,
            Models::CLAUDE_3_5_HAIKU,
            Models::CLAUDE_3_OPUS,
            Models::CLAUDE_3_SONNET,
            Models::CLAUDE_3_HAIKU,
        ];
        let unique_count = models
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len();
        assert_eq!(unique_count, 5, "All model constants should be distinct");
    }

    #[test]
    fn test_model_serialization() {
        let model = Model {
            id: "claude-3-5-sonnet-20241022".to_string(),
            model_type: "model".to_string(),
            display_name: "Claude 3.5 Sonnet".to_string(),
            created_at: chrono::Utc::now(),
        };

        let json = serde_json::to_value(&model).unwrap();
        assert_eq!(json["id"], "claude-3-5-sonnet-20241022");
        assert_eq!(json["type"], "model");
        assert_eq!(json["display_name"], "Claude 3.5 Sonnet");
        assert!(json.get("created_at").is_some());
    }

    #[test]
    fn test_model_deserialization() {
        let json = r#"{
            "id": "claude-3-opus-20240229",
            "type": "model",
            "display_name": "Claude 3 Opus",
            "created_at": "2024-02-29T00:00:00Z"
        }"#;

        let model: Model = serde_json::from_str(json).unwrap();
        assert_eq!(model.id, "claude-3-opus-20240229");
        assert_eq!(model.model_type, "model");
        assert_eq!(model.display_name, "Claude 3 Opus");
    }
}
