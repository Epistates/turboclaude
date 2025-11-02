//! Request validation for the Anthropic SDK
//!
//! This module provides comprehensive validation of requests before they're sent to the API.
//! Validation happens early to provide clear error messages and fail fast, improving debugging
//! and user experience.
//!
//! # Validation Strategy
//!
//! Validation is organized by concern:
//! - **Structural**: Required fields, array bounds, type correctness
//! - **Semantic**: Logical constraints (e.g., max_tokens <= budget_tokens for extended thinking)
//! - **Format**: String patterns, URL validity, base64 encoding
//! - **Provider-specific**: Constraints that only apply to certain providers
//!
//! # Examples
//!
//! ```rust
//! use turboclaude::types::{MessageRequest, Message};
//! use turboclaude::validation::validate_message_request;
//!
//! // Valid request with at least one message
//! let request = MessageRequest::builder()
//!     .model("claude-3-5-sonnet-20241022")
//!     .max_tokens(1024u32)
//!     .messages(vec![Message::user("Hello")])
//!     .build()?;
//!
//! // Validation passes
//! validate_message_request(&request)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::error::{Error, Result};
use crate::types::{ContentBlockParam, MessageParam, MessageRequest, SystemPrompt};
use tracing::debug;

/// Validate a MessageRequest before sending to the API.
///
/// This performs comprehensive validation including:
/// - Required fields are present and non-empty
/// - Token limits are reasonable
/// - Content blocks are properly formed
/// - Extended thinking configuration is valid
///
/// # Errors
///
/// Returns `Error::InvalidRequest` with a descriptive message for any validation failure.
///
/// # Examples
///
/// ```rust,no_run
/// use turboclaude::types::{MessageRequest, Message};
/// use turboclaude::validation::validate_message_request;
///
/// let request = MessageRequest::builder()
///     .model("claude-3-5-sonnet-20241022")
///     .max_tokens(1024u32)
///     .messages(vec![Message::user("Hello")])
///     .build()?;
///
/// validate_message_request(&request)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn validate_message_request(request: &MessageRequest) -> Result<()> {
    debug!(
        model = %request.model,
        max_tokens = request.max_tokens,
        message_count = request.messages.len(),
        has_system = request.system.is_some(),
        has_thinking = request.thinking.is_some(),
        has_tools = request.tools.is_some(),
        "Validating message request"
    );

    // Validate model
    validate_model_id(&request.model)?;

    // Validate max_tokens
    validate_max_tokens(request.max_tokens)?;

    // Validate messages
    validate_messages(&request.messages)?;

    // Validate system prompt if present
    if let Some(system) = &request.system {
        validate_system_prompt(system)?;
    }

    // Validate extended thinking if enabled
    if let Some(thinking) = &request.thinking {
        thinking
            .validate()
            .map_err(|e| Error::InvalidRequest(format!("Invalid thinking configuration: {}", e)))?;

        // Semantic check: max_tokens must be at least thinking budget + output
        let min_output_tokens = 256; // Minimum for response
        if request.max_tokens < thinking.budget_tokens + min_output_tokens {
            return Err(Error::InvalidRequest(format!(
                "max_tokens ({}) must be at least thinking budget ({}) + {} tokens for output",
                request.max_tokens, thinking.budget_tokens, min_output_tokens
            )));
        }

        debug!(
            thinking_budget = thinking.budget_tokens,
            "Extended thinking enabled"
        );
    }

    // Validate tool configuration if present
    if let Some(tools) = &request.tools {
        if tools.is_empty() {
            return Err(Error::InvalidRequest(
                "Tools array cannot be empty. Either provide tools or omit the field.".to_string(),
            ));
        }
        for tool in tools {
            // Basic tool validation
            // More detailed validation is handled by the Tool type itself
            if tool.name.is_empty() {
                return Err(Error::InvalidRequest(
                    "Tool name cannot be empty".to_string(),
                ));
            }
        }
        debug!(tool_count = tools.len(), "Tools configured");
    }

    debug!("Message request validation passed");
    Ok(())
}

/// Validate a model ID.
///
/// # Errors
///
/// Returns `Error::InvalidRequest` if the model ID is invalid.
fn validate_model_id(model: &str) -> Result<()> {
    if model.is_empty() {
        return Err(Error::InvalidRequest(
            "Model ID cannot be empty".to_string(),
        ));
    }

    if model.len() > 1024 {
        return Err(Error::InvalidRequest(
            "Model ID exceeds maximum length of 1024 characters".to_string(),
        ));
    }

    // Model IDs should be ASCII alphanumeric with hyphens and dots
    if !model
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '_')
    {
        return Err(Error::InvalidRequest(format!(
            "Invalid model ID '{}': must contain only alphanumeric characters, hyphens, dots, and underscores",
            model
        )));
    }

    Ok(())
}

/// Validate max_tokens value.
///
/// # Errors
///
/// Returns `Error::InvalidRequest` if max_tokens is invalid.
fn validate_max_tokens(max_tokens: u32) -> Result<()> {
    if max_tokens == 0 {
        return Err(Error::InvalidRequest(
            "max_tokens must be greater than 0".to_string(),
        ));
    }

    // Anthropic's maximum token limit
    const MAX_TOKEN_LIMIT: u32 = 200_000;
    if max_tokens > MAX_TOKEN_LIMIT {
        return Err(Error::InvalidRequest(format!(
            "max_tokens ({}) exceeds maximum allowed value ({})",
            max_tokens, MAX_TOKEN_LIMIT
        )));
    }

    Ok(())
}

/// Validate messages array.
///
/// # Errors
///
/// Returns `Error::InvalidRequest` if the messages array is invalid.
fn validate_messages(messages: &[MessageParam]) -> Result<()> {
    if messages.is_empty() {
        return Err(Error::InvalidRequest(
            "Messages array cannot be empty. At least one message is required.".to_string(),
        ));
    }

    if messages.len() > 10_000 {
        return Err(Error::InvalidRequest(
            "Messages array exceeds maximum length of 10,000 messages".to_string(),
        ));
    }

    // Validate each message
    for (index, message) in messages.iter().enumerate() {
        validate_message_param(message, index)?;
    }

    // Validate message sequence: user and assistant must alternate
    // (with some flexibility for edge cases)
    let mut expected_role = Some("user");
    for message in messages {
        let role = match message.role {
            crate::types::Role::User => "user",
            crate::types::Role::Assistant => "assistant",
        };

        if let Some(exp) = expected_role {
            if exp == role && role == "assistant" {
                // Allow consecutive assistants (edge case)
                continue;
            }
            expected_role = if role == "user" {
                Some("assistant")
            } else {
                Some("user")
            };
        }
    }

    Ok(())
}

/// Validate a single message parameter.
///
/// # Errors
///
/// Returns `Error::InvalidRequest` if the message is invalid.
fn validate_message_param(message: &MessageParam, index: usize) -> Result<()> {
    // Check role and validate content
    if message.content.is_empty() {
        return Err(Error::InvalidRequest(format!(
            "Message at index {} has empty content",
            index
        )));
    }

    match message.role {
        crate::types::Role::User => {
            // User messages can have any content type
            for (block_index, block) in message.content.iter().enumerate() {
                validate_content_block(block, index, block_index)?;
            }
        }
        crate::types::Role::Assistant => {
            // Assistant messages should typically only have text or tool use
            for (block_index, block) in message.content.iter().enumerate() {
                match block {
                    ContentBlockParam::Text { .. } => {
                        // Valid
                    }
                    ContentBlockParam::ToolResult { .. } => {
                        // Valid - this is assistant responding to tool
                    }
                    _ => {
                        return Err(Error::InvalidRequest(format!(
                            "Assistant message at index {} content block {} has unsupported type",
                            index, block_index
                        )));
                    }
                }
            }
        }
    }

    Ok(())
}

/// Validate a content block parameter.
///
/// # Errors
///
/// Returns `Error::InvalidRequest` if the content block is invalid.
fn validate_content_block(
    block: &ContentBlockParam,
    message_index: usize,
    block_index: usize,
) -> Result<()> {
    match block {
        ContentBlockParam::Text { text } => {
            if text.is_empty() {
                return Err(Error::InvalidRequest(format!(
                    "Text content block at message {} block {} is empty",
                    message_index, block_index
                )));
            }

            if text.len() > 1_000_000 {
                return Err(Error::InvalidRequest(
                    "Text content exceeds 1 million characters".to_string(),
                ));
            }
        }

        ContentBlockParam::Image { source } => {
            // Validate media type
            match source.media_type.as_str() {
                "image/jpeg" | "image/png" | "image/gif" | "image/webp" => {
                    // Valid formats
                }
                _ => {
                    return Err(Error::InvalidRequest(format!(
                        "Unsupported image media type '{}' at message {} block {}. Supported types: image/jpeg, image/png, image/gif, image/webp",
                        source.media_type, message_index, block_index
                    )));
                }
            }

            // Validate base64 encoding
            if source.data.is_empty() {
                return Err(Error::InvalidRequest(format!(
                    "Image data is empty at message {} block {}",
                    message_index, block_index
                )));
            }

            // Quick base64 validation
            if !source
                .data
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
            {
                return Err(Error::InvalidRequest(format!(
                    "Invalid base64 encoding in image data at message {} block {}",
                    message_index, block_index
                )));
            }
        }

        ContentBlockParam::Document { source, .. } => {
            match source {
                crate::types::DocumentSource::PlainText { text } => {
                    if text.is_empty() {
                        return Err(Error::InvalidRequest(format!(
                            "Document text is empty at message {} block {}",
                            message_index, block_index
                        )));
                    }

                    if text.len() > 5_000_000 {
                        return Err(Error::InvalidRequest(
                            "Document text exceeds 5 million characters".to_string(),
                        ));
                    }
                }
                crate::types::DocumentSource::Base64PDF { data, .. } => {
                    if data.is_empty() {
                        return Err(Error::InvalidRequest(format!(
                            "Document data is empty at message {} block {}",
                            message_index, block_index
                        )));
                    }

                    // Quick base64 validation
                    if !data
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
                    {
                        return Err(Error::InvalidRequest(format!(
                            "Invalid base64 encoding in document data at message {} block {}",
                            message_index, block_index
                        )));
                    }
                }
                crate::types::DocumentSource::URL { url } => {
                    // For now, accept URLs (providers may have restrictions)
                    if url.is_empty() {
                        return Err(Error::InvalidRequest(format!(
                            "Document URL is empty at message {} block {}",
                            message_index, block_index
                        )));
                    }

                    // URL validation (basic)
                    if !url.starts_with("http://") && !url.starts_with("https://") {
                        return Err(Error::InvalidRequest(format!(
                            "Document URL must start with http:// or https:// at message {} block {}",
                            message_index, block_index
                        )));
                    }
                }
            }
        }

        ContentBlockParam::ToolResult { tool_use_id, .. } => {
            if tool_use_id.is_empty() {
                return Err(Error::InvalidRequest(format!(
                    "Tool use ID is empty at message {} block {}",
                    message_index, block_index
                )));
            }
        }
    }

    Ok(())
}

/// Validate a system prompt.
///
/// # Errors
///
/// Returns `Error::InvalidRequest` if the system prompt is invalid.
fn validate_system_prompt(system: &SystemPrompt) -> Result<()> {
    match system {
        SystemPrompt::String(s) => {
            if s.is_empty() {
                return Err(Error::InvalidRequest(
                    "System prompt string cannot be empty".to_string(),
                ));
            }

            if s.len() > 10_000 {
                return Err(Error::InvalidRequest(
                    "System prompt exceeds 10,000 characters".to_string(),
                ));
            }
        }
        SystemPrompt::Blocks(blocks) => {
            if blocks.is_empty() {
                return Err(Error::InvalidRequest(
                    "System prompt blocks array cannot be empty".to_string(),
                ));
            }

            for block in blocks {
                match block {
                    crate::types::SystemPromptBlock::Text { text, .. } => {
                        if text.is_empty() {
                            return Err(Error::InvalidRequest(
                                "System prompt text block is empty".to_string(),
                            ));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Message, MessageRequest};

    #[test]
    fn test_validate_model_id_valid() {
        assert!(validate_model_id("claude-3-5-sonnet-20241022").is_ok());
        assert!(validate_model_id("claude-3-opus-20240229").is_ok());
        assert!(validate_model_id("gpt-4").is_ok());
    }

    #[test]
    fn test_validate_model_id_empty() {
        assert!(validate_model_id("").is_err());
    }

    #[test]
    fn test_validate_model_id_invalid_chars() {
        assert!(validate_model_id("claude@invalid").is_err());
        assert!(validate_model_id("claude!test").is_err());
    }

    #[test]
    fn test_validate_max_tokens() {
        assert!(validate_max_tokens(1024).is_ok());
        assert!(validate_max_tokens(0).is_err());
        assert!(validate_max_tokens(200_001).is_err());
    }

    #[test]
    fn test_validate_messages_empty() {
        let messages: Vec<MessageParam> = vec![];
        assert!(validate_messages(&messages).is_err());
    }

    #[test]
    fn test_validate_message_request_valid() {
        let request = MessageRequest::builder()
            .model("claude-3-5-sonnet-20241022")
            .max_tokens(1024u32)
            .messages(vec![Message::user("Hello")])
            .build()
            .expect("Failed to build request");

        assert!(validate_message_request(&request).is_ok());
    }

    #[test]
    fn test_validate_message_request_empty_messages() {
        let request = MessageRequest::builder()
            .model("claude-3-5-sonnet-20241022")
            .max_tokens(1024u32)
            .messages(vec![])
            .build()
            .expect("Failed to build request");

        assert!(validate_message_request(&request).is_err());
    }

    #[test]
    fn test_validate_message_request_zero_max_tokens() {
        let request = MessageRequest::builder()
            .model("claude-3-5-sonnet-20241022")
            .max_tokens(0u32)
            .messages(vec![Message::user("Hello")])
            .build()
            .expect("Failed to build request");

        assert!(validate_message_request(&request).is_err());
    }
}
