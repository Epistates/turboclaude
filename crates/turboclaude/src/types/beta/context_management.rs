//! Context management types for controlling conversation history
//!
//! This module provides types for managing the conversation context, including
//! clearing old thinking blocks, clearing tool use blocks, and editing
//! conversation history.

use serde::{Deserialize, Serialize};

/// Union type for context management edits
///
/// Represents different types of edits that can be applied to the conversation context.
///
/// # Variants
///
/// * `ClearThinking` - Clear old thinking blocks from conversation history
/// * `ClearToolUses` - Clear old tool use/result blocks from conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContextManagementEdit {
    /// Clear thinking blocks from conversation history
    ClearThinking(super::thinking::BetaClearThinking20251015EditParam),

    /// Clear tool use/result blocks from conversation history
    ClearToolUses(super::thinking::BetaClearToolUses20250919EditParam),
}

impl ContextManagementEdit {
    /// Create a clear thinking edit
    pub fn clear_thinking(param: super::thinking::BetaClearThinking20251015EditParam) -> Self {
        Self::ClearThinking(param)
    }

    /// Create a clear tool uses edit
    pub fn clear_tool_uses(param: super::thinking::BetaClearToolUses20250919EditParam) -> Self {
        Self::ClearToolUses(param)
    }

    /// Get the edit type as a string
    pub fn edit_type(&self) -> &'static str {
        match self {
            Self::ClearThinking(_) => "clear_thinking_20251015",
            Self::ClearToolUses(_) => "clear_tool_uses_20250919",
        }
    }

    /// Returns true if this is a clear thinking edit
    pub fn is_clear_thinking(&self) -> bool {
        matches!(self, Self::ClearThinking(_))
    }

    /// Returns true if this is a clear tool uses edit
    pub fn is_clear_tool_uses(&self) -> bool {
        matches!(self, Self::ClearToolUses(_))
    }
}

/// Union type for context management edit responses
///
/// Represents the response from different types of context management edits.
///
/// # Variants
///
/// * `ClearThinking` - Response from clearing thinking blocks
/// * `ClearToolUses` - Response from clearing tool use blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContextManagementEditResponse {
    /// Response from clearing thinking blocks
    ClearThinking(super::thinking::BetaClearThinking20251015EditResponse),

    /// Response from clearing tool use blocks
    ClearToolUses(super::thinking::BetaClearToolUses20250919EditResponse),
}

impl ContextManagementEditResponse {
    /// Create a clear thinking response
    pub fn clear_thinking(
        response: super::thinking::BetaClearThinking20251015EditResponse,
    ) -> Self {
        Self::ClearThinking(response)
    }

    /// Create a clear tool uses response
    pub fn clear_tool_uses(
        response: super::thinking::BetaClearToolUses20250919EditResponse,
    ) -> Self {
        Self::ClearToolUses(response)
    }

    /// Get the response type as a string
    pub fn response_type(&self) -> &'static str {
        match self {
            Self::ClearThinking(_) => "clear_thinking_20251015",
            Self::ClearToolUses(_) => "clear_tool_uses_20250919",
        }
    }

    /// Returns true if this is a clear thinking response
    pub fn is_clear_thinking(&self) -> bool {
        matches!(self, Self::ClearThinking(_))
    }

    /// Returns true if this is a clear tool uses response
    pub fn is_clear_tool_uses(&self) -> bool {
        matches!(self, Self::ClearToolUses(_))
    }

    /// Get the number of cleared input tokens
    pub fn cleared_input_tokens(&self) -> u32 {
        match self {
            Self::ClearThinking(r) => r.cleared_input_tokens,
            Self::ClearToolUses(r) => r.cleared_input_tokens,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::beta::{
        BetaClearThinking20251015EditParam, BetaClearThinking20251015EditResponse,
        BetaClearToolUses20250919EditParam, BetaClearToolUses20250919EditResponse,
    };

    // ===== ContextManagementEdit Tests =====

    #[test]
    fn test_context_management_edit_clear_thinking() {
        let param = BetaClearThinking20251015EditParam::with_turns(5);
        let edit = ContextManagementEdit::clear_thinking(param);

        assert!(edit.is_clear_thinking());
        assert_eq!(edit.edit_type(), "clear_thinking_20251015");
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

        assert!(edit_response.is_clear_thinking());
        assert_eq!(edit_response.response_type(), "clear_thinking_20251015");
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

    // ===== ClearToolUses Edit Tests =====

    #[test]
    fn test_context_management_edit_clear_tool_uses() {
        let param = BetaClearToolUses20250919EditParam::new();
        let edit = ContextManagementEdit::clear_tool_uses(param);

        assert_eq!(edit.edit_type(), "clear_tool_uses_20250919");
        assert!(edit.is_clear_tool_uses());
        assert!(!edit.is_clear_thinking());
    }

    #[test]
    fn test_context_management_edit_clear_tool_uses_with_keep() {
        let param = BetaClearToolUses20250919EditParam::with_keep_turns(5);
        let edit = ContextManagementEdit::clear_tool_uses(param);

        assert_eq!(edit.edit_type(), "clear_tool_uses_20250919");
    }

    #[test]
    fn test_context_management_edit_clear_tool_uses_serialization() {
        let param = BetaClearToolUses20250919EditParam::with_keep_turns(3);
        let edit = ContextManagementEdit::clear_tool_uses(param);

        let json = serde_json::to_string(&edit).unwrap();
        assert!(json.contains("\"type\":\"clear_tool_uses_20250919\""));
        assert!(json.contains("\"keep_turns\":3"));
    }

    // ===== ClearToolUses Response Tests =====

    #[test]
    fn test_context_management_edit_response_clear_tool_uses() {
        let response = BetaClearToolUses20250919EditResponse::new(2048, 10);
        let edit_response = ContextManagementEditResponse::clear_tool_uses(response);

        assert_eq!(edit_response.response_type(), "clear_tool_uses_20250919");
        assert!(edit_response.is_clear_tool_uses());
        assert!(!edit_response.is_clear_thinking());
    }

    #[test]
    fn test_context_management_edit_response_cleared_tokens() {
        let thinking_response = BetaClearThinking20251015EditResponse {
            cleared_input_tokens: 1024,
            cleared_thinking_turns: 3,
            response_type: "clear_thinking_20251015".to_string(),
        };
        let thinking_wrapper = ContextManagementEditResponse::clear_thinking(thinking_response);
        assert_eq!(thinking_wrapper.cleared_input_tokens(), 1024);

        let tool_uses_response = BetaClearToolUses20250919EditResponse::new(2048, 10);
        let tool_uses_wrapper = ContextManagementEditResponse::clear_tool_uses(tool_uses_response);
        assert_eq!(tool_uses_wrapper.cleared_input_tokens(), 2048);
    }

    #[test]
    fn test_context_management_clear_tool_uses_workflow() {
        // Scenario: Clear all tool uses
        let clear_param = BetaClearToolUses20250919EditParam::new();
        let edit = ContextManagementEdit::clear_tool_uses(clear_param);

        assert_eq!(edit.edit_type(), "clear_tool_uses_20250919");

        // Simulate response
        let response = BetaClearToolUses20250919EditResponse::new(5000, 20);
        let response_wrapper = ContextManagementEditResponse::clear_tool_uses(response);

        assert_eq!(response_wrapper.response_type(), "clear_tool_uses_20250919");
        assert_eq!(response_wrapper.cleared_input_tokens(), 5000);
    }

    #[test]
    fn test_is_helper_methods() {
        let thinking_edit = ContextManagementEdit::clear_thinking(
            BetaClearThinking20251015EditParam::clear_all()
        );
        assert!(thinking_edit.is_clear_thinking());
        assert!(!thinking_edit.is_clear_tool_uses());

        let tool_uses_edit = ContextManagementEdit::clear_tool_uses(
            BetaClearToolUses20250919EditParam::new()
        );
        assert!(!tool_uses_edit.is_clear_thinking());
        assert!(tool_uses_edit.is_clear_tool_uses());
    }
}
