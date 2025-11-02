//! Context management types for controlling conversation history
//!
//! This module provides types for managing the conversation context, including
//! clearing old thinking blocks and editing conversation history.

use serde::{Deserialize, Serialize};

/// Union type for context management edits
///
/// Represents different types of edits that can be applied to the conversation context.
///
/// # Variants
///
/// * `ClearThinking` - Clear old thinking blocks from conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContextManagementEdit {
    /// Clear thinking blocks from conversation history
    ClearThinking(super::thinking::BetaClearThinking20251015EditParam),
}

impl ContextManagementEdit {
    /// Create a clear thinking edit
    pub fn clear_thinking(param: super::thinking::BetaClearThinking20251015EditParam) -> Self {
        Self::ClearThinking(param)
    }

    /// Get the edit type as a string
    pub fn edit_type(&self) -> &'static str {
        match self {
            Self::ClearThinking(_) => "clear_thinking_20251015",
        }
    }
}

/// Union type for context management edit responses
///
/// Represents the response from different types of context management edits.
///
/// # Variants
///
/// * `ClearThinking` - Response from clearing thinking blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContextManagementEditResponse {
    /// Response from clearing thinking blocks
    ClearThinking(super::thinking::BetaClearThinking20251015EditResponse),
}

impl ContextManagementEditResponse {
    /// Create a clear thinking response
    pub fn clear_thinking(
        response: super::thinking::BetaClearThinking20251015EditResponse,
    ) -> Self {
        Self::ClearThinking(response)
    }

    /// Get the response type as a string
    pub fn response_type(&self) -> &'static str {
        match self {
            Self::ClearThinking(_) => "clear_thinking_20251015",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::beta::{
        BetaClearThinking20251015EditParam, BetaClearThinking20251015EditResponse,
    };

    // ===== ContextManagementEdit Tests =====

    #[test]
    fn test_context_management_edit_clear_thinking() {
        let param = BetaClearThinking20251015EditParam::with_turns(5);
        let edit = ContextManagementEdit::clear_thinking(param);

        match edit {
            ContextManagementEdit::ClearThinking(_) => {
                assert_eq!(edit.edit_type(), "clear_thinking_20251015");
            }
        }
    }

    #[test]
    fn test_context_management_edit_type_string() {
        let param = BetaClearThinking20251015EditParam::clear_all();
        let edit = ContextManagementEdit::ClearThinking(param);

        assert_eq!(edit.edit_type(), "clear_thinking_20251015");
    }

    #[test]
    fn test_context_management_edit_serialization() {
        let param = BetaClearThinking20251015EditParam::with_turns(3);
        let edit = ContextManagementEdit::clear_thinking(param);

        let json = serde_json::to_string(&edit).unwrap();
        assert!(json.contains("\"type\":\"clear_thinking_20251015\""));
    }

    #[test]
    fn test_context_management_edit_deserialization() {
        let json = r#"{"type":"clear_thinking_20251015"}"#;
        let edit: ContextManagementEdit = serde_json::from_str(json).unwrap();

        assert_eq!(edit.edit_type(), "clear_thinking_20251015");
    }

    // ===== ContextManagementEditResponse Tests =====

    #[test]
    fn test_context_management_edit_response_clear_thinking() {
        let response = BetaClearThinking20251015EditResponse {
            cleared_input_tokens: 1024,
            cleared_thinking_turns: 3,
            response_type: "clear_thinking_20251015".to_string(),
        };

        let edit_response = ContextManagementEditResponse::clear_thinking(response);

        match edit_response {
            ContextManagementEditResponse::ClearThinking(_) => {
                assert_eq!(edit_response.response_type(), "clear_thinking_20251015");
            }
        }
    }

    #[test]
    fn test_context_management_edit_response_type_string() {
        let response = BetaClearThinking20251015EditResponse {
            cleared_input_tokens: 512,
            cleared_thinking_turns: 2,
            response_type: "clear_thinking_20251015".to_string(),
        };

        let edit_response = ContextManagementEditResponse::ClearThinking(response);

        assert_eq!(edit_response.response_type(), "clear_thinking_20251015");
    }

    #[test]
    fn test_context_management_edit_response_serialization() {
        let response = BetaClearThinking20251015EditResponse {
            cleared_input_tokens: 2048,
            cleared_thinking_turns: 5,
            response_type: "clear_thinking_20251015".to_string(),
        };

        let edit_response = ContextManagementEditResponse::clear_thinking(response);
        let json = serde_json::to_string(&edit_response).unwrap();

        assert!(json.contains("\"type\":\"clear_thinking_20251015\""));
        assert!(json.contains("\"cleared_input_tokens\":2048"));
    }

    #[test]
    fn test_context_management_edit_response_deserialization() {
        let json = r#"{"type":"clear_thinking_20251015","cleared_input_tokens":1024,"cleared_thinking_turns":3,"response_type":"clear_thinking_20251015"}"#;
        let response: ContextManagementEditResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.response_type(), "clear_thinking_20251015");
    }

    // ===== Integration Tests =====

    #[test]
    fn test_context_management_workflow() {
        // Create an edit
        let param = BetaClearThinking20251015EditParam::with_turns(10);
        let edit = ContextManagementEdit::clear_thinking(param);

        // Verify edit type
        assert_eq!(edit.edit_type(), "clear_thinking_20251015");

        // Simulate receiving response
        let response = BetaClearThinking20251015EditResponse {
            cleared_input_tokens: 3000,
            cleared_thinking_turns: 8,
            response_type: "clear_thinking_20251015".to_string(),
        };

        let response_wrapper = ContextManagementEditResponse::clear_thinking(response);

        // Verify response type
        assert_eq!(response_wrapper.response_type(), "clear_thinking_20251015");
    }

    #[test]
    fn test_context_management_round_trip() {
        let param = BetaClearThinking20251015EditParam::keep_all();
        let edit = ContextManagementEdit::clear_thinking(param);

        // Serialize
        let json = serde_json::to_string(&edit).unwrap();

        // Deserialize
        let restored: ContextManagementEdit = serde_json::from_str(&json).unwrap();

        // Verify
        assert_eq!(edit.edit_type(), restored.edit_type());
    }
}
