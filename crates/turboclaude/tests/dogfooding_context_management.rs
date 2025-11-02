//! Dogfooding Tests: Context Management Strategies
//!
//! Real-world integration tests that exercise context management functionality
//! across multiple turns of conversation with realistic token usage patterns.
//!
//! These tests validate:
//! - Clear thinking type serialization/deserialization
//! - Context management response handling
//! - Token usage tracking
//! - Real-world conversation patterns
//!
//! Run with: cargo test --test dogfooding_context_management -- --nocapture

#[cfg(test)]
mod tests {
    use turboclaude::types::beta::context_management::{
        ContextManagementEdit, ContextManagementEditResponse,
    };
    use turboclaude::types::beta::thinking::{
        BetaAllThinkingTurnsParam, BetaClearThinking20251015EditParam,
        BetaClearThinking20251015EditResponse, BetaThinkingTurnsParam, Keep,
    };

    /// Test creating clear thinking edit request for keeping specific turns
    #[test]
    fn test_context_management_keep_turns() {
        let edit = BetaClearThinking20251015EditParam {
            param_type: "clear_thinking".to_string(),
            keep: Some(Keep::ThinkingTurns(BetaThinkingTurnsParam {
                param_type: "thinking_turns".to_string(),
                value: 3,
            })),
        };

        assert_eq!(edit.param_type, "clear_thinking");
        assert!(edit.keep.is_some());
    }

    /// Test creating clear thinking edit request for keeping all thinking
    #[test]
    fn test_context_management_keep_all() {
        let edit = BetaClearThinking20251015EditParam {
            param_type: "clear_thinking".to_string(),
            keep: Some(Keep::AllThinking(BetaAllThinkingTurnsParam {
                param_type: "all".to_string(),
            })),
        };

        assert_eq!(edit.param_type, "clear_thinking");
        assert!(edit.keep.is_some());
    }

    /// Test creating clear thinking edit request with no keep (clear all)
    #[test]
    fn test_context_management_clear_all() {
        let edit = BetaClearThinking20251015EditParam {
            param_type: "clear_thinking".to_string(),
            keep: None,
        };

        assert_eq!(edit.param_type, "clear_thinking");
        assert!(edit.keep.is_none());
    }

    /// Test response creation for context management operations
    #[test]
    fn test_context_management_response() {
        let response = BetaClearThinking20251015EditResponse {
            cleared_input_tokens: 2048,
            cleared_thinking_turns: 5,
            response_type: "clear_thinking_edit_result".to_string(),
        };

        assert_eq!(response.cleared_input_tokens, 2048);
        assert_eq!(response.cleared_thinking_turns, 5);
        assert_eq!(response.response_type, "clear_thinking_edit_result");
    }

    /// Test context management edit enum with thinking variant
    #[test]
    fn test_context_management_edit_enum() {
        let edit_param = BetaClearThinking20251015EditParam {
            param_type: "clear_thinking".to_string(),
            keep: Some(Keep::ThinkingTurns(BetaThinkingTurnsParam {
                param_type: "thinking_turns".to_string(),
                value: 2,
            })),
        };

        let edit = ContextManagementEdit::ClearThinking(edit_param);
        match edit {
            ContextManagementEdit::ClearThinking(p) => {
                assert_eq!(p.param_type, "clear_thinking");
            }
        }
    }

    /// Test context management response enum
    #[test]
    fn test_context_management_response_enum() {
        let response_data = BetaClearThinking20251015EditResponse {
            cleared_input_tokens: 1024,
            cleared_thinking_turns: 3,
            response_type: "clear_thinking_edit_result".to_string(),
        };

        let response = ContextManagementEditResponse::ClearThinking(response_data);
        match response {
            ContextManagementEditResponse::ClearThinking(r) => {
                assert_eq!(r.cleared_input_tokens, 1024);
                assert_eq!(r.cleared_thinking_turns, 3);
            }
        }
    }

    /// Test serialization of context management edit
    #[test]
    fn test_context_edit_serialization() {
        let edit = BetaClearThinking20251015EditParam {
            param_type: "clear_thinking".to_string(),
            keep: Some(Keep::ThinkingTurns(BetaThinkingTurnsParam {
                param_type: "thinking_turns".to_string(),
                value: 2,
            })),
        };

        let json = serde_json::to_string(&edit).expect("serialization failed");
        assert!(json.contains("clear_thinking"));
        assert!(json.contains("thinking_turns"));
        assert!(json.contains("\"value\":2"));
    }

    /// Test serialization of context management response
    #[test]
    fn test_context_response_serialization() {
        let response = BetaClearThinking20251015EditResponse {
            cleared_input_tokens: 512,
            cleared_thinking_turns: 2,
            response_type: "clear_thinking_edit_result".to_string(),
        };

        let json = serde_json::to_string(&response).expect("serialization failed");
        assert!(json.contains("clear_thinking_edit_result"));
        assert!(json.contains("512"));
        assert!(json.contains("2"));
    }

    /// Test deserialization of context management edit
    #[test]
    fn test_context_edit_deserialization() {
        let json = r#"{
            "type": "clear_thinking",
            "keep": {
                "type": "thinking_turns",
                "value": 3
            }
        }"#;

        let edit: BetaClearThinking20251015EditParam =
            serde_json::from_str(json).expect("deserialization failed");

        assert_eq!(edit.param_type, "clear_thinking");
        assert!(edit.keep.is_some());
    }

    /// Test deserialization of context management response
    #[test]
    fn test_context_response_deserialization() {
        let json = r#"{
            "cleared_input_tokens": 1024,
            "cleared_thinking_turns": 4,
            "type": "clear_thinking_edit_result"
        }"#;

        let response: BetaClearThinking20251015EditResponse =
            serde_json::from_str(json).expect("deserialization failed");

        assert_eq!(response.cleared_input_tokens, 1024);
        assert_eq!(response.cleared_thinking_turns, 4);
    }

    /// Test multi-turn conversation with context clearing
    /// Simulates a realistic conversation pattern where context is cleared
    #[test]
    fn test_multi_turn_context_clearing_scenario() {
        // Turn 1: Initial question
        let turn_1_tokens_used = 150;

        // Turn 2: Follow-up with context preserved
        let turn_2_tokens_used = 200;

        // Turn 3: Another question
        let turn_3_tokens_used = 180;

        // Total after 3 turns
        let total_before_clearing = turn_1_tokens_used + turn_2_tokens_used + turn_3_tokens_used;
        assert_eq!(total_before_clearing, 530);

        // Clear thinking and check tokens freed
        let clear_response = BetaClearThinking20251015EditResponse {
            cleared_input_tokens: 300, // Context clearing saves 300 tokens
            cleared_thinking_turns: 2, // Removed 2 thinking turns
            response_type: "clear_thinking_edit_result".to_string(),
        };

        let tokens_after_clearing = total_before_clearing - clear_response.cleared_input_tokens;
        assert_eq!(tokens_after_clearing, 230);
    }

    /// Test strategy: Keep recent turns only
    #[test]
    fn test_strategy_keep_recent_turns() {
        // Strategy: Keep last 2 thinking turns
        let edit = BetaClearThinking20251015EditParam {
            param_type: "clear_thinking".to_string(),
            keep: Some(Keep::ThinkingTurns(BetaThinkingTurnsParam {
                param_type: "thinking_turns".to_string(),
                value: 2, // Keep only last 2 turns
            })),
        };

        // Simulate response
        let response = BetaClearThinking20251015EditResponse {
            cleared_input_tokens: 1500, // Large savings by removing old turns
            cleared_thinking_turns: 8,  // Removed 8 old thinking turns
            response_type: "clear_thinking_edit_result".to_string(),
        };

        assert_eq!(edit.param_type, "clear_thinking");
        assert_eq!(response.cleared_input_tokens, 1500);
        assert_eq!(response.cleared_thinking_turns, 8);
    }

    /// Test strategy: Keep all thinking (preserve full context)
    #[test]
    fn test_strategy_keep_all_thinking() {
        // Strategy: Keep all thinking for interconnected topics
        let edit = BetaClearThinking20251015EditParam {
            param_type: "clear_thinking".to_string(),
            keep: Some(Keep::AllThinking(BetaAllThinkingTurnsParam {
                param_type: "all".to_string(),
            })),
        };

        // Response shows minimal clearing
        let response = BetaClearThinking20251015EditResponse {
            cleared_input_tokens: 0, // No thinking removed
            cleared_thinking_turns: 0,
            response_type: "clear_thinking_edit_result".to_string(),
        };

        assert_eq!(edit.param_type, "clear_thinking");
        assert_eq!(response.cleared_input_tokens, 0); // Full preservation
    }

    /// Test strategy: Clear all thinking (fresh start)
    #[test]
    fn test_strategy_clear_all_thinking() {
        // Strategy: Clear all thinking when switching topics entirely
        let edit = BetaClearThinking20251015EditParam {
            param_type: "clear_thinking".to_string(),
            keep: None, // Keep nothing
        };

        // Response shows full clearing
        let response = BetaClearThinking20251015EditResponse {
            cleared_input_tokens: 2000, // All thinking removed
            cleared_thinking_turns: 10, // All thinking turns removed
            response_type: "clear_thinking_edit_result".to_string(),
        };

        assert!(edit.keep.is_none()); // No turns kept
        assert_eq!(response.cleared_input_tokens, 2000);
        assert_eq!(response.cleared_thinking_turns, 10);
    }

    /// Test round-trip serialization for edit
    #[test]
    fn test_context_edit_round_trip() {
        let original = BetaClearThinking20251015EditParam {
            param_type: "clear_thinking".to_string(),
            keep: Some(Keep::ThinkingTurns(BetaThinkingTurnsParam {
                param_type: "thinking_turns".to_string(),
                value: 5,
            })),
        };

        let json = serde_json::to_string(&original).expect("serialize failed");
        let deserialized: BetaClearThinking20251015EditParam =
            serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(original.param_type, deserialized.param_type);
        assert_eq!(original.keep.is_some(), deserialized.keep.is_some());
    }

    /// Test round-trip serialization for response
    #[test]
    fn test_context_response_round_trip() {
        let original = BetaClearThinking20251015EditResponse {
            cleared_input_tokens: 888,
            cleared_thinking_turns: 6,
            response_type: "clear_thinking_edit_result".to_string(),
        };

        let json = serde_json::to_string(&original).expect("serialize failed");
        let deserialized: BetaClearThinking20251015EditResponse =
            serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(
            original.cleared_input_tokens,
            deserialized.cleared_input_tokens
        );
        assert_eq!(
            original.cleared_thinking_turns,
            deserialized.cleared_thinking_turns
        );
        assert_eq!(original.response_type, deserialized.response_type);
    }

    /// Test realistic token usage monitoring scenario
    #[test]
    fn test_token_usage_monitoring() {
        // Simulate conversation with token limits
        let max_tokens = 8000;
        let mut current_tokens = 0;

        // Turn 1
        current_tokens += 500; // Query costs 500 tokens
        assert!(current_tokens < max_tokens);

        // Turn 2
        current_tokens += 600;
        assert!(current_tokens < max_tokens);

        // Turn 3
        current_tokens += 550;
        assert!(current_tokens < max_tokens);

        // Check if we need to clear
        if current_tokens > max_tokens / 2 {
            // Clear old thinking
            let clear_response = BetaClearThinking20251015EditResponse {
                cleared_input_tokens: 700,
                cleared_thinking_turns: 2,
                response_type: "clear_thinking_edit_result".to_string(),
            };

            current_tokens -= clear_response.cleared_input_tokens;
        }

        assert!(current_tokens < max_tokens);
    }
}
