//! Property-based tests for turboclaude
//!
//! This module uses proptest to generate random inputs and verify invariants
//! about the SDK's behavior. Property-based testing helps catch edge cases
//! and ensure correctness across a wide range of inputs.

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    // ===== Strategy Generators =====

    #[allow(dead_code)]
    fn arb_model_id() -> impl Strategy<Value = String> {
        "claude-3-(5-)?[a-z]+-[0-9]{8}"
    }

    fn arb_token_count() -> impl Strategy<Value = u32> {
        1u32..200_000u32
    }

    fn arb_short_text() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9 ]{1,100}"
    }

    // ===== Message Request Properties =====

    proptest! {
        /// Property: MessageRequest serializes to valid JSON
        /// Invariant: Any valid request can be serialized
        #[test]
        fn prop_message_request_serializes(
            max_tokens in arb_token_count(),
        ) {
            use crate::types::{ContentBlockParam, MessageParam, MessageRequest, Role};

            let request = MessageRequest::builder()
                .model("claude-3-5-sonnet-20241022")
                .max_tokens(max_tokens)
                .messages(vec![MessageParam {
                    role: Role::User,
                    content: vec![ContentBlockParam::Text {
                        text: "test".to_string(),
                    }],
                }])
                .build()
                .expect("Failed to build request");

            // Must be serializable
            let _json = serde_json::to_string(&request)
                .expect("Failed to serialize");

            prop_assert!(true);
        }

        /// Property: MessageRequest validation accepts valid max_tokens
        /// Invariant: max_tokens in (0, 200000] always passes validation
        #[test]
        fn prop_validation_accepts_valid_max_tokens(
            max_tokens in arb_token_count(),
        ) {
            use crate::types::{ContentBlockParam, MessageParam, MessageRequest, Role};
            use crate::validation::validate_message_request;

            let request = MessageRequest::builder()
                .model("claude-3-5-sonnet-20241022")
                .max_tokens(max_tokens)
                .messages(vec![MessageParam {
                    role: Role::User,
                    content: vec![ContentBlockParam::Text {
                        text: "test".to_string(),
                    }],
                }])
                .build()
                .expect("Failed to build");

            prop_assert!(validate_message_request(&request).is_ok(),
                "max_tokens={} should be valid", max_tokens);
        }

        /// Property: MessageRequest validation rejects zero max_tokens
        /// Invariant: max_tokens = 0 always fails validation
        #[test]
        fn prop_validation_rejects_zero_max_tokens(_unused in Just(())) {
            use crate::types::{ContentBlockParam, MessageParam, MessageRequest, Role};
            use crate::validation::validate_message_request;

            let request = MessageRequest::builder()
                .model("claude-3-5-sonnet-20241022")
                .max_tokens(0u32)
                .messages(vec![MessageParam {
                    role: Role::User,
                    content: vec![ContentBlockParam::Text {
                        text: "test".to_string(),
                    }],
                }])
                .build()
                .expect("Failed to build");

            prop_assert!(validate_message_request(&request).is_err(),
                "max_tokens=0 should fail validation");
        }
    }

    // ===== Content Block Properties =====

    proptest! {
        /// Property: Text content blocks preserve content
        /// Invariant: Serialization round-trip preserves text exactly
        #[test]
        fn prop_text_block_preserves_content(
            text in arb_short_text()
        ) {
            use crate::types::ContentBlockParam;

            let block = ContentBlockParam::Text { text: text.clone() };

            let json = serde_json::to_string(&block)
                .expect("Failed to serialize");

            let deserialized: ContentBlockParam = serde_json::from_str(&json)
                .expect("Failed to deserialize");

            match deserialized {
                ContentBlockParam::Text { text: deserialized_text } => {
                    prop_assert_eq!(text, deserialized_text);
                }
                _ => prop_assert!(false, "Expected Text block"),
            }
        }

        /// Property: Tool results require non-empty IDs
        /// Invariant: Empty tool_use_id fails validation
        #[cfg(any(feature = "bedrock", feature = "vertex"))]
        #[test]
        fn prop_tool_result_requires_id(
            content in arb_short_text(),
        ) {
            use crate::providers::shared::transform_content_blocks;
            use crate::types::ContentBlockParam;

            let block_with_empty_id = vec![
                ContentBlockParam::ToolResult {
                    tool_use_id: String::new(),
                    content,
                    is_error: None,
                },
            ];

            let result = transform_content_blocks(&block_with_empty_id);
            prop_assert!(result.is_err(), "Should fail with empty tool_use_id");
        }

        /// Property: Tool results accept valid IDs and content
        /// Invariant: Non-empty ID and content passes validation
        #[cfg(any(feature = "bedrock", feature = "vertex"))]
        #[test]
        fn prop_tool_result_accepts_valid_data(
            id in "tool_[a-z0-9]{1,20}",
            content in arb_short_text(),
        ) {
            use crate::providers::shared::transform_content_blocks;
            use crate::types::ContentBlockParam;

            let block = vec![ContentBlockParam::ToolResult {
                tool_use_id: id,
                content,
                is_error: None,
            }];

            let result = transform_content_blocks(&block);
            prop_assert!(result.is_ok(), "Should accept valid tool result");
        }
    }

    // ===== System Prompt Properties =====

    proptest! {
        /// Property: System prompts preserve text
        /// Invariant: String and block prompts with same text produce same output
        #[cfg(any(feature = "bedrock", feature = "vertex"))]
        #[test]
        fn prop_system_prompt_text_preservation(
            text in arb_short_text()
        ) {
            use crate::providers::shared::extract_system_prompt_text;
            use crate::types::{SystemPrompt, SystemPromptBlock};

            let string_prompt = SystemPrompt::String(text.clone());
            let block_prompt = SystemPrompt::Blocks(vec![
                SystemPromptBlock::Text {
                    text: text.clone(),
                    cache_control: None,
                }
            ]);

            let string_text = extract_system_prompt_text(&string_prompt);
            let block_text = extract_system_prompt_text(&block_prompt);

            prop_assert_eq!(&string_text, &block_text,
                "Both formats should produce same text");
            prop_assert_eq!(&string_text, &text,
                "Text should be preserved exactly");
        }
    }

    // ===== Error Handling Properties =====

    proptest! {
        /// Property: Validation is idempotent
        /// Invariant: Running validation twice produces same result
        #[test]
        fn prop_validation_is_idempotent(
            max_tokens in arb_token_count(),
        ) {
            use crate::types::{ContentBlockParam, MessageParam, MessageRequest, Role};
            use crate::validation::validate_message_request;

            let request = MessageRequest::builder()
                .model("claude-3-5-sonnet-20241022")
                .max_tokens(max_tokens)
                .messages(vec![MessageParam {
                    role: Role::User,
                    content: vec![ContentBlockParam::Text {
                        text: "test".to_string(),
                    }],
                }])
                .build()
                .expect("Failed to build");

            let first = validate_message_request(&request);
            let second = validate_message_request(&request);

            // Both must have same result
            match (&first, &second) {
                (Ok(_), Ok(_)) => prop_assert!(true),
                (Err(_), Err(_)) => prop_assert!(true),
                _ => prop_assert!(false,
                    "Validation results differ between runs"),
            }
        }

        /// Property: Messages array validation is consistent
        /// Invariant: Non-empty messages always pass basic validation
        #[test]
        fn prop_messages_array_validation(
            text in arb_short_text(),
        ) {
            use crate::types::{ContentBlockParam, MessageParam, MessageRequest, Role};
            use crate::validation::validate_message_request;

            let request = MessageRequest::builder()
                .model("claude-3-5-sonnet-20241022")
                .max_tokens(1024u32)
                .messages(vec![MessageParam {
                    role: Role::User,
                    content: vec![ContentBlockParam::Text {
                        text,
                    }],
                }])
                .build()
                .expect("Failed to build");

            // Non-empty messages should pass
            prop_assert!(validate_message_request(&request).is_ok());
        }
    }
}
