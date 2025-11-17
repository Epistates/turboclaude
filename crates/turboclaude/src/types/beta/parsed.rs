//! Parsed message types for structured outputs
//!
//! This module provides types for working with structured outputs from the Claude API.
//! When using the structured outputs feature, responses can be automatically parsed
//! into strongly-typed Rust structs.

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::marker::PhantomData;

use crate::types::{ContentBlock, Message};
use crate::Error;

/// Type alias for beta message - currently using the same Message type
pub type BetaMessage = Message;

/// A beta message response with parsed structured output.
///
/// This type wraps a [`BetaMessage`] and provides type-safe access to structured
/// output parsed from the message's text content.
///
/// # Type Parameters
///
/// * `T` - The type to parse the output into. Must implement `Deserialize`.
///
/// # Example
///
/// ```rust,ignore
/// use serde::{Deserialize, Serialize};
/// use schemars::JsonSchema;
///
/// #[derive(Debug, Serialize, Deserialize, JsonSchema)]
/// struct Order {
///     product_name: String,
///     price: f64,
///     quantity: u32,
/// }
///
/// let parsed: ParsedBetaMessage<Order> = client.beta().messages()
///     .parse::<Order>()
///     .model(models::CLAUDE_SONNET_4_5_20250929_STRUCTURED_OUTPUTS)
///     .messages(messages)
///     .max_tokens(1024)
///     .send()
///     .await?;
///
/// let order = parsed.parsed_output()?;
/// println!("Order: {}", order.product_name);
/// ```
#[derive(Debug, Clone)]
pub struct ParsedBetaMessage<T> {
    /// The underlying beta message
    pub message: BetaMessage,

    /// Phantom data to hold the type parameter
    _phantom: PhantomData<T>,
}

impl<T> ParsedBetaMessage<T>
where
    T: DeserializeOwned,
{
    /// Create a new parsed message from a beta message.
    ///
    /// This does not perform parsing yet - parsing happens lazily when
    /// [`parsed_output`](Self::parsed_output) is called.
    pub fn new(message: BetaMessage) -> Self {
        Self {
            message,
            _phantom: PhantomData,
        }
    }

    /// Extract and parse the structured output from the message.
    ///
    /// This method:
    /// 1. Finds the first text content block in the message
    /// 2. Parses the text as JSON
    /// 3. Deserializes it into type `T`
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The message contains no text blocks
    /// - The text is not valid JSON
    /// - The JSON doesn't match the expected schema for type `T`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let order = parsed.parsed_output()?;
    /// println!("Ordered {} of {}", order.quantity, order.product_name);
    /// ```
    pub fn parsed_output(&self) -> Result<T, Error> {
        // Find the first text block
        let text = self
            .message
            .content
            .iter()
            .find_map(|block| match block {
                ContentBlock::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .ok_or_else(|| Error::ResponseValidation(
                "No text content block found in message for structured output parsing".to_string()
            ))?;

        // Parse the JSON text into type T
        serde_json::from_str(text).map_err(|e| Error::ResponseValidation(
            format!("Failed to parse structured output as JSON: {}", e)
        ))
    }

    /// Get a reference to the underlying beta message.
    ///
    /// Useful for accessing metadata like usage, stop_reason, etc.
    pub fn message(&self) -> &BetaMessage {
        &self.message
    }

    /// Consume self and return the underlying beta message.
    pub fn into_message(self) -> BetaMessage {
        self.message
    }
}

// Implement Serialize/Deserialize by delegating to the message
impl<T> Serialize for ParsedBetaMessage<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.message.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for ParsedBetaMessage<T>
where
    T: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let message = BetaMessage::deserialize(deserializer)?;
        Ok(Self::new(message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestOutput {
        name: String,
        count: u32,
    }

    #[test]
    fn test_parsed_message_basic() {
        use crate::types::{Role, Usage, StopReason};

        let text_json = r#"{"name": "test", "count": 42}"#;
        let message = BetaMessage {
            id: "msg_123".to_string(),
            message_type: "message".to_string(),
            role: Role::Assistant,
            content: vec![ContentBlock::Text {
                text: text_json.to_string(),
                citations: None,
            }],
            model: "claude-sonnet-4-5-20250929-structured-outputs".to_string(),
            stop_reason: Some(StopReason::EndTurn),
            stop_sequence: None,
            usage: Usage {
                input_tokens: 10,
                output_tokens: 20,
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
            },
        };

        let parsed = ParsedBetaMessage::<TestOutput>::new(message);
        let output = parsed.parsed_output().unwrap();

        assert_eq!(output.name, "test");
        assert_eq!(output.count, 42);
    }

    #[test]
    fn test_parsed_message_no_text_block() {
        use crate::types::{Role, Usage};

        let message = BetaMessage {
            id: "msg_123".to_string(),
            message_type: "message".to_string(),
            role: Role::Assistant,
            content: vec![],  // No content blocks
            model: "claude-sonnet-4-5-20250929-structured-outputs".to_string(),
            stop_reason: None,
            stop_sequence: None,
            usage: Usage {
                input_tokens: 10,
                output_tokens: 0,
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
            },
        };

        let parsed = ParsedBetaMessage::<TestOutput>::new(message);
        let result = parsed.parsed_output();

        assert!(result.is_err());
        if let Err(Error::ResponseValidation(message)) = result {
            assert!(message.contains("No text content block found"));
        } else {
            panic!("Expected ParseResponse error");
        }
    }

    #[test]
    fn test_parsed_message_invalid_json() {
        use crate::types::{Role, Usage, StopReason};

        let message = BetaMessage {
            id: "msg_123".to_string(),
            message_type: "message".to_string(),
            role: Role::Assistant,
            content: vec![ContentBlock::Text {
                text: "not valid json".to_string(),
                citations: None,
            }],
            model: "claude-sonnet-4-5-20250929-structured-outputs".to_string(),
            stop_reason: Some(StopReason::EndTurn),
            stop_sequence: None,
            usage: Usage {
                input_tokens: 10,
                output_tokens: 20,
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
            },
        };

        let parsed = ParsedBetaMessage::<TestOutput>::new(message);
        let result = parsed.parsed_output();

        assert!(result.is_err());
        if let Err(Error::ResponseValidation(message)) = result {
            assert!(message.contains("Failed to parse structured output"));
        } else {
            panic!("Expected ParseResponse error");
        }
    }

    #[test]
    fn test_parsed_message_accessors() {
        use crate::types::{Role, Usage, StopReason};

        let text_json = r#"{"name": "test", "count": 42}"#;
        let message = BetaMessage {
            id: "msg_123".to_string(),
            message_type: "message".to_string(),
            role: Role::Assistant,
            content: vec![ContentBlock::Text {
                text: text_json.to_string(),
                citations: None,
            }],
            model: "claude-sonnet-4-5-20250929-structured-outputs".to_string(),
            stop_reason: Some(StopReason::EndTurn),
            stop_sequence: None,
            usage: Usage {
                input_tokens: 10,
                output_tokens: 20,
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
            },
        };

        let parsed = ParsedBetaMessage::<TestOutput>::new(message.clone());

        // Test message() accessor
        assert_eq!(parsed.message().id, "msg_123");
        assert_eq!(parsed.message().usage.input_tokens, 10);

        // Test into_message()
        let extracted_message = parsed.into_message();
        assert_eq!(extracted_message.id, message.id);
    }
}
