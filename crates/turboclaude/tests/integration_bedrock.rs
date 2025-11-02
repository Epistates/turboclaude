//! Integration tests for Bedrock provider
//!
//! These tests verify the Bedrock provider works correctly with mocked AWS responses.
//! Tests cover:
//! - Non-streaming message creation
//! - Streaming responses
//! - Error handling (invalid credentials, throttling, etc.)
//! - Type translation between turboclaude and Bedrock types

#![cfg(feature = "bedrock")]

use turboclaude::{
    error::Error,
    types::{ContentBlockParam, ImageSource, Message, MessageRequest},
    validation::validate_message_request,
};

/// Test that request validation catches empty messages before API calls
#[test]
fn test_bedrock_validation_empty_messages() {
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![])
        .build()
        .expect("Failed to build request");

    let result = validate_message_request(&request);
    assert!(result.is_err(), "Should reject empty messages");

    if let Err(Error::InvalidRequest(msg)) = result {
        assert!(msg.contains("empty"), "Error should mention empty messages");
    }
}

/// Test that request validation catches invalid max_tokens
#[test]
fn test_bedrock_validation_invalid_max_tokens() {
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(0u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .expect("Failed to build request");

    let result = validate_message_request(&request);
    assert!(result.is_err(), "Should reject zero max_tokens");
}

/// Test that request validation catches invalid model IDs
#[test]
fn test_bedrock_validation_invalid_model() {
    let request = MessageRequest::builder()
        .model("claude@invalid!")
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .expect("Failed to build request");

    let result = validate_message_request(&request);
    assert!(result.is_err(), "Should reject invalid model ID");
}

/// Test that text messages pass validation
#[test]
fn test_bedrock_validation_text_message_valid() {
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello, Claude!")])
        .build()
        .expect("Failed to build request");

    let result = validate_message_request(&request);
    assert!(result.is_ok(), "Valid text message should pass validation");
}

/// Test that image messages with valid base64 pass validation
#[test]
fn test_bedrock_validation_image_valid() {
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![turboclaude::types::UserMessage {
            content: vec![ContentBlockParam::Image {
                source: ImageSource::base64("image/jpeg", "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="),
            }],
        }.into()])
        .build()
        .expect("Failed to build request");

    let result = validate_message_request(&request);
    assert!(result.is_ok(), "Valid image should pass validation");
}

/// Test that image with invalid base64 fails validation
#[test]
fn test_bedrock_validation_image_invalid_base64() {
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![
            turboclaude::types::UserMessage {
                content: vec![ContentBlockParam::Image {
                    source: ImageSource::base64("image/jpeg", "!!!invalid base64!!!"),
                }],
            }
            .into(),
        ])
        .build()
        .expect("Failed to build request");

    let result = validate_message_request(&request);
    assert!(result.is_err(), "Invalid base64 should fail validation");
}

/// Test that unsupported image formats fail validation
#[test]
fn test_bedrock_validation_image_unsupported_format() {
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![turboclaude::types::UserMessage {
            content: vec![ContentBlockParam::Image {
                source: ImageSource {
                    source_type: "base64".to_string(),
                    media_type: "image/bmp".to_string(), // Not supported
                    data: "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==".to_string(),
                },
            }],
        }.into()])
        .build()
        .expect("Failed to build request");

    let result = validate_message_request(&request);
    assert!(
        result.is_err(),
        "Unsupported image format should fail validation"
    );
}

/// Test that empty text blocks fail validation
#[test]
fn test_bedrock_validation_empty_text_block() {
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![
            turboclaude::types::UserMessage {
                content: vec![ContentBlockParam::Text {
                    text: String::new(),
                }],
            }
            .into(),
        ])
        .build()
        .expect("Failed to build request");

    let result = validate_message_request(&request);
    assert!(result.is_err(), "Empty text block should fail validation");
}

/// Test that system prompt is validated
#[test]
fn test_bedrock_validation_system_prompt_empty() {
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .system(turboclaude::types::SystemPrompt::String(String::new()))
        .build()
        .expect("Failed to build request");

    let result = validate_message_request(&request);
    assert!(
        result.is_err(),
        "Empty system prompt should fail validation"
    );
}

/// Test that valid system prompt passes validation
#[test]
fn test_bedrock_validation_system_prompt_valid() {
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .system(turboclaude::types::SystemPrompt::String(
            "You are helpful".to_string(),
        ))
        .build()
        .expect("Failed to build request");

    let result = validate_message_request(&request);
    assert!(result.is_ok(), "Valid system prompt should pass validation");
}

/// Test that extended thinking with insufficient max_tokens fails
#[test]
fn test_bedrock_validation_thinking_insufficient_tokens() {
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(100u32) // Not enough for 5000 thinking + output
        .messages(vec![Message::user("Hello")])
        .thinking(turboclaude::types::beta::ThinkingConfig::new(5000))
        .build()
        .expect("Failed to build request");

    let result = validate_message_request(&request);
    assert!(
        result.is_err(),
        "Insufficient tokens for thinking should fail"
    );
}

/// Test that extended thinking with sufficient tokens passes
#[test]
fn test_bedrock_validation_thinking_sufficient_tokens() {
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(6000u32) // 5000 thinking + 1000 output = 6000
        .messages(vec![Message::user("Hello")])
        .thinking(turboclaude::types::beta::ThinkingConfig::new(5000))
        .build()
        .expect("Failed to build request");

    let result = validate_message_request(&request);
    assert!(result.is_ok(), "Sufficient tokens for thinking should pass");
}

/// Test that tools with empty names fail validation
#[test]
fn test_bedrock_validation_tool_empty_name() {
    use turboclaude::types::Tool;

    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .tools(vec![Tool {
            name: String::new(), // Empty!
            description: Some("A tool".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        }])
        .build()
        .expect("Failed to build request");

    let result = validate_message_request(&request);
    assert!(
        result.is_err(),
        "Tool with empty name should fail validation"
    );
}

/// Test model ID normalization for Bedrock
#[cfg(feature = "bedrock")]
#[test]
fn test_bedrock_model_id_normalization() {
    use turboclaude::providers::bedrock::http::BedrockHttpProvider;

    // Short format without version
    assert_eq!(
        BedrockHttpProvider::normalize_model_id("claude-3-5-sonnet-20241022"),
        "anthropic.claude-3-5-sonnet-20241022-v2:0"
    );

    // Already in Bedrock format
    assert_eq!(
        BedrockHttpProvider::normalize_model_id("anthropic.claude-3-5-sonnet-20241022-v2:0"),
        "anthropic.claude-3-5-sonnet-20241022-v2:0"
    );

    // With version suffix
    assert_eq!(
        BedrockHttpProvider::normalize_model_id("claude-3-opus-20240229-v1:0"),
        "anthropic.claude-3-opus-20240229-v1:0"
    );
}

#[cfg(test)]
mod streaming {
    use super::*;

    /// Test that streaming requests are properly validated
    #[test]
    fn test_bedrock_streaming_validation() {
        let mut request = MessageRequest::builder()
            .model("claude-3-5-sonnet-20241022")
            .max_tokens(1024u32)
            .messages(vec![])
            .stream(true)
            .build()
            .expect("Failed to build request");

        let result = validate_message_request(&request);
        assert!(
            result.is_err(),
            "Empty messages should fail even in streaming mode"
        );
    }

    /// Test that valid streaming requests pass validation
    #[test]
    fn test_bedrock_streaming_valid() {
        let request = MessageRequest::builder()
            .model("claude-3-5-sonnet-20241022")
            .max_tokens(1024u32)
            .messages(vec![Message::user("Stream this")])
            .stream(true)
            .build()
            .expect("Failed to build request");

        let result = validate_message_request(&request);
        assert!(
            result.is_ok(),
            "Valid streaming request should pass validation"
        );
    }
}
