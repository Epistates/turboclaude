//! Beta Models API types
//!
//! Types for listing and retrieving model information via the Beta API.

use serde::{Deserialize, Serialize};

pub use turboclaude_protocol::types::Model;

/// Paginated response for model listing.
///
/// The Models API returns models in descending order of creation date,
/// with the most recently released models listed first.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPage {
    /// List of models in this page.
    pub data: Vec<Model>,

    /// Whether there are more results available.
    pub has_more: bool,

    /// ID of the first model in this page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,

    /// ID of the last model in this page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_page_serialization() {
        let page = ModelPage {
            data: vec![],
            has_more: false,
            first_id: Some("model_1".to_string()),
            last_id: Some("model_2".to_string()),
        };

        let json = serde_json::to_value(&page).unwrap();
        assert_eq!(json["has_more"], false);
        assert_eq!(json["first_id"], "model_1");
        assert_eq!(json["last_id"], "model_2");
        assert!(json["data"].is_array());
    }

    #[test]
    fn test_model_page_deserialization() {
        let json = r#"{
            "data": [],
            "has_more": true,
            "first_id": "claude-3-5-sonnet-20241022",
            "last_id": "claude-3-opus-20240229"
        }"#;

        let page: ModelPage = serde_json::from_str(json).unwrap();
        assert_eq!(page.has_more, true);
        assert_eq!(
            page.first_id,
            Some("claude-3-5-sonnet-20241022".to_string())
        );
        assert_eq!(page.last_id, Some("claude-3-opus-20240229".to_string()));
        assert_eq!(page.data.len(), 0);
    }

    #[test]
    fn test_model_page_with_models() {
        let json = r#"{
            "data": [
                {
                    "id": "claude-3-5-sonnet-20241022",
                    "type": "model",
                    "display_name": "Claude 3.5 Sonnet",
                    "created_at": "2024-10-22T00:00:00Z"
                }
            ],
            "has_more": false,
            "first_id": "claude-3-5-sonnet-20241022",
            "last_id": "claude-3-5-sonnet-20241022"
        }"#;

        let page: ModelPage = serde_json::from_str(json).unwrap();
        assert_eq!(page.data.len(), 1);
        assert_eq!(page.data[0].id, "claude-3-5-sonnet-20241022");
        assert_eq!(page.data[0].display_name, Some("Claude 3.5 Sonnet".to_string()));
    }

    #[test]
    fn test_model_page_optional_fields_omitted() {
        let page = ModelPage {
            data: vec![],
            has_more: false,
            first_id: None,
            last_id: None,
        };

        let json = serde_json::to_value(&page).unwrap();
        assert!(!json.as_object().unwrap().contains_key("first_id"));
        assert!(!json.as_object().unwrap().contains_key("last_id"));
    }
}
