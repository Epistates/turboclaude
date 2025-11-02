//! HTTP helper utilities for provider implementations
//!
//! Consolidates duplicate HTTP handling logic from multiple providers.

use crate::error::Result;
use crate::types::MessageRequest;
use serde::Serialize;

/// Deserialize a message request from serialized bytes
///
/// This logic was duplicated across Bedrock HTTP, Bedrock client, and Vertex HTTP providers.
/// Provides a single source of truth for request deserialization.
///
/// # Arguments
///
/// * `body` - The serializable request body
///
/// # Returns
///
/// A deserialized `MessageRequest`, or an error if serialization/deserialization fails
///
/// # Example
///
/// ```ignore
/// use turboclaude::providers::shared::deserialize_message_request;
///
/// let request = MessageRequest::builder()
///     .model("claude-3-5-sonnet-20241022")
///     .max_tokens(1024)
///     .messages(vec![])
///     .build()?;
///
/// let deserialized = deserialize_message_request(&request)?;
/// ```
pub fn deserialize_message_request<T: Serialize>(body: &T) -> Result<MessageRequest> {
    let json_bytes = serde_json::to_vec(body).map_err(crate::error::Error::Serialization)?;
    let request: MessageRequest =
        serde_json::from_slice(&json_bytes).map_err(crate::error::Error::Serialization)?;
    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ContentBlockParam, MessageParam, Role};

    #[test]
    fn test_deserialize_valid_request() {
        let request = MessageRequest::builder()
            .model("claude-3-5-sonnet-20241022")
            .max_tokens(1024u32)
            .messages(vec![MessageParam {
                role: Role::User,
                content: vec![ContentBlockParam::Text {
                    text: "Hello".to_string(),
                }],
            }])
            .build()
            .expect("Failed to build request");

        let result = deserialize_message_request(&request);
        assert!(result.is_ok());

        let deserialized = result.unwrap();
        assert_eq!(deserialized.model, request.model);
        assert_eq!(deserialized.max_tokens, request.max_tokens);
        assert_eq!(deserialized.messages.len(), 1);
    }

    #[test]
    fn test_deserialize_with_system_prompt() {
        let request = MessageRequest::builder()
            .model("claude-3-5-sonnet-20241022")
            .max_tokens(1024u32)
            .system(crate::types::SystemPrompt::String(
                "You are helpful".to_string(),
            ))
            .messages(vec![MessageParam {
                role: Role::User,
                content: vec![ContentBlockParam::Text {
                    text: "Hello".to_string(),
                }],
            }])
            .build()
            .expect("Failed to build request");

        let result = deserialize_message_request(&request);
        assert!(result.is_ok());

        let deserialized = result.unwrap();
        assert!(deserialized.system.is_some());
    }
}
