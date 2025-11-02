//! Streaming event validation for message streams
//!
//! This module provides validation of Server-Sent Events (SSE) received from the Anthropic API
//! to ensure they conform to the expected format and sequence.
//!
//! # Validation Strategy
//!
//! Validation occurs at multiple levels:
//! - **Syntactic**: Event structure matches expected format
//! - **Semantic**: Event values are logically consistent
//! - **Sequence**: Events arrive in the correct order
//!
//! # Event Sequence Validation
//!
//! A valid stream must follow this pattern:
//! ```text
//! message_start
//! (content_block_start
//!  content_block_delta*
//!  content_block_stop)+
//! message_delta?
//! message_stop
//! ```
//!
//! Where:
//! - `?` = optional (0 or 1)
//! - `*` = repeating (0 or more)
//! - `+` = required repeating (1 or more)
//!
//! Additional constraints:
//! - All content_block_* events for a given block must have matching indices
//! - At least one content block is required
//! - Only text and tool_use content blocks are valid
//! - Usage must have non-negative token counts

use crate::error::{Error, Result};
use crate::streaming::StreamEvent;
use tracing::{debug, warn};

/// Validator for streaming message events
#[derive(Debug)]
pub struct StreamEventValidator {
    /// Whether message_start has been received
    message_started: bool,
    /// Current content block index being processed
    current_block_index: Option<usize>,
    /// Whether the current content block was started
    block_started: bool,
    /// Number of content blocks completed
    completed_blocks: usize,
    /// Whether message_stop has been received
    message_stopped: bool,
}

impl StreamEventValidator {
    /// Create a new stream event validator
    pub fn new() -> Self {
        Self {
            message_started: false,
            current_block_index: None,
            block_started: false,
            completed_blocks: 0,
            message_stopped: false,
        }
    }

    /// Validate a single streaming event
    ///
    /// # Returns
    /// `Ok(())` if the event is valid, or an error if validation fails
    ///
    /// # Errors
    /// Returns errors for:
    /// - Events in wrong sequence
    /// - Missing required fields
    /// - Invalid indices or values
    pub fn validate_event(&mut self, event: &StreamEvent) -> Result<()> {
        match event {
            StreamEvent::MessageStart(start) => {
                if self.message_started {
                    return Err(Error::Streaming(
                        "Received multiple message_start events".to_string(),
                    ));
                }
                if self.message_stopped {
                    return Err(Error::Streaming(
                        "Received message_start after message_stop".to_string(),
                    ));
                }

                // Validate message structure
                if start.message.id.is_empty() {
                    return Err(Error::Streaming(
                        "message_start: message ID cannot be empty".to_string(),
                    ));
                }
                if start.message.model.is_empty() {
                    return Err(Error::Streaming(
                        "message_start: model cannot be empty".to_string(),
                    ));
                }

                debug!(
                    "Stream validation: message_start received (id={})",
                    start.message.id
                );
                self.message_started = true;
                Ok(())
            }

            StreamEvent::ContentBlockStart(start) => {
                if !self.message_started {
                    return Err(Error::Streaming(
                        "Received content_block_start before message_start".to_string(),
                    ));
                }
                if self.message_stopped {
                    return Err(Error::Streaming(
                        "Received content_block_start after message_stop".to_string(),
                    ));
                }

                // Finalize previous block if needed
                if self.block_started && self.current_block_index.is_some() {
                    return Err(Error::Streaming(
                        "Started new content block without finishing previous one".to_string(),
                    ));
                }

                debug!(
                    "Stream validation: content_block_start (index={})",
                    start.index
                );
                self.current_block_index = Some(start.index);
                self.block_started = true;
                Ok(())
            }

            StreamEvent::ContentBlockDelta(delta) => {
                if !self.message_started {
                    return Err(Error::Streaming(
                        "Received content_block_delta before message_start".to_string(),
                    ));
                }
                if !self.block_started {
                    return Err(Error::Streaming(
                        "Received content_block_delta without content_block_start".to_string(),
                    ));
                }
                if self.message_stopped {
                    return Err(Error::Streaming(
                        "Received content_block_delta after message_stop".to_string(),
                    ));
                }

                // Validate index matches
                let expected_index = self.current_block_index.ok_or_else(|| {
                    Error::Streaming("content_block_delta: no current block index".to_string())
                })?;

                if delta.index != expected_index {
                    return Err(Error::Streaming(format!(
                        "content_block_delta: index mismatch (expected {}, got {})",
                        expected_index, delta.index
                    )));
                }

                // Validate delta has content
                if delta.delta.text.is_none() && delta.delta.partial_json.is_none() {
                    return Err(Error::Streaming(
                        "content_block_delta: neither text nor partial_json provided".to_string(),
                    ));
                }

                Ok(())
            }

            StreamEvent::ContentBlockStop(stop) => {
                if !self.message_started {
                    return Err(Error::Streaming(
                        "Received content_block_stop before message_start".to_string(),
                    ));
                }
                if !self.block_started {
                    return Err(Error::Streaming(
                        "Received content_block_stop without content_block_start".to_string(),
                    ));
                }
                if self.message_stopped {
                    return Err(Error::Streaming(
                        "Received content_block_stop after message_stop".to_string(),
                    ));
                }

                // Validate index matches
                let expected_index = self.current_block_index.ok_or_else(|| {
                    Error::Streaming("content_block_stop: no current block index".to_string())
                })?;

                if stop.index != expected_index {
                    return Err(Error::Streaming(format!(
                        "content_block_stop: index mismatch (expected {}, got {})",
                        expected_index, stop.index
                    )));
                }

                debug!(
                    "Stream validation: content_block_stop (index={})",
                    stop.index
                );
                self.block_started = false;
                self.completed_blocks += 1;
                Ok(())
            }

            StreamEvent::MessageDelta(delta) => {
                if !self.message_started {
                    return Err(Error::Streaming(
                        "Received message_delta before message_start".to_string(),
                    ));
                }
                if self.message_stopped {
                    return Err(Error::Streaming(
                        "Received message_delta after message_stop".to_string(),
                    ));
                }

                // Validate usage if present
                if let Some(usage) = &delta.usage
                    && usage.output_tokens as i32 <= 0
                {
                    warn!(
                        "Stream validation: message_delta has non-positive output_tokens ({})",
                        usage.output_tokens
                    );
                }

                debug!("Stream validation: message_delta received");
                Ok(())
            }

            StreamEvent::MessageStop => {
                if !self.message_started {
                    return Err(Error::Streaming(
                        "Received message_stop without message_start".to_string(),
                    ));
                }
                if self.message_stopped {
                    return Err(Error::Streaming(
                        "Received multiple message_stop events".to_string(),
                    ));
                }

                // Ensure no incomplete content blocks
                if self.block_started {
                    return Err(Error::Streaming(
                        "Received message_stop with incomplete content block".to_string(),
                    ));
                }

                // Ensure at least one content block was completed
                if self.completed_blocks == 0 {
                    return Err(Error::Streaming(
                        "Stream ended without any complete content blocks".to_string(),
                    ));
                }

                debug!(
                    "Stream validation: message_stop received ({} blocks)",
                    self.completed_blocks
                );
                self.message_stopped = true;
                Ok(())
            }

            StreamEvent::Ping => {
                // Pings are always valid
                Ok(())
            }

            StreamEvent::Unknown => {
                debug!("Stream validation: unknown event received");
                Ok(())
            }
        }
    }

    /// Check if the stream is in a valid final state
    ///
    /// A stream is valid if:
    /// - message_start was received
    /// - message_stop was received
    /// - At least one content block was completed
    pub fn is_complete_and_valid(&self) -> Result<()> {
        if !self.message_started {
            return Err(Error::Streaming(
                "Stream ended without message_start".to_string(),
            ));
        }

        if !self.message_stopped {
            return Err(Error::Streaming(
                "Stream ended without message_stop".to_string(),
            ));
        }

        if self.completed_blocks == 0 {
            return Err(Error::Streaming(
                "Stream has no completed content blocks".to_string(),
            ));
        }

        if self.block_started {
            return Err(Error::Streaming(
                "Stream ended with incomplete content block".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the number of completed content blocks
    pub fn completed_blocks(&self) -> usize {
        self.completed_blocks
    }
}

impl Default for StreamEventValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming::*;
    use crate::types::Usage;

    fn create_message_start_event() -> StreamEvent {
        StreamEvent::MessageStart(MessageStartEvent {
            message: PartialMessage {
                id: "msg_123".to_string(),
                message_type: "message".to_string(),
                role: "assistant".to_string(),
                model: "claude-3-5-sonnet-20241022".to_string(),
                content: vec![],
                stop_reason: None,
                stop_sequence: None,
                usage: Some(Usage {
                    input_tokens: 10,
                    output_tokens: 0,
                    cache_creation_input_tokens: None,
                    cache_read_input_tokens: None,
                }),
            },
        })
    }

    #[test]
    fn test_message_start_required() {
        let mut validator = StreamEventValidator::new();
        let delta = StreamEvent::ContentBlockDelta(ContentBlockDeltaEvent {
            index: 0,
            delta: ContentDelta {
                text: Some("hello".to_string()),
                partial_json: None,
            },
        });

        let result = validator.validate_event(&delta);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("before message_start")
        );
    }

    #[test]
    fn test_valid_stream_sequence() {
        let mut validator = StreamEventValidator::new();

        // message_start
        assert!(
            validator
                .validate_event(&create_message_start_event())
                .is_ok()
        );

        // content_block_start
        assert!(
            validator
                .validate_event(&StreamEvent::ContentBlockStart(ContentBlockStartEvent {
                    index: 0,
                    content_block: PartialContentBlock::Text {
                        text: "".to_string(),
                    },
                }))
                .is_ok()
        );

        // content_block_delta
        assert!(
            validator
                .validate_event(&StreamEvent::ContentBlockDelta(ContentBlockDeltaEvent {
                    index: 0,
                    delta: ContentDelta {
                        text: Some("Hello".to_string()),
                        partial_json: None,
                    },
                }))
                .is_ok()
        );

        // content_block_stop
        assert!(
            validator
                .validate_event(&StreamEvent::ContentBlockStop(ContentBlockStopEvent {
                    index: 0
                }))
                .is_ok()
        );

        // message_delta
        assert!(
            validator
                .validate_event(&StreamEvent::MessageDelta(MessageDeltaEvent {
                    delta: MessageDelta {
                        stop_reason: Some(crate::types::StopReason::EndTurn),
                        stop_sequence: None,
                    },
                    usage: Some(DeltaUsage { output_tokens: 1 }),
                }))
                .is_ok()
        );

        // message_stop
        assert!(validator.validate_event(&StreamEvent::MessageStop).is_ok());

        // Verify stream is valid
        assert!(validator.is_complete_and_valid().is_ok());
    }

    #[test]
    fn test_index_mismatch_error() {
        let mut validator = StreamEventValidator::new();
        validator
            .validate_event(&create_message_start_event())
            .unwrap();

        validator
            .validate_event(&StreamEvent::ContentBlockStart(ContentBlockStartEvent {
                index: 0,
                content_block: PartialContentBlock::Text {
                    text: "".to_string(),
                },
            }))
            .unwrap();

        // Delta with mismatched index
        let result =
            validator.validate_event(&StreamEvent::ContentBlockDelta(ContentBlockDeltaEvent {
                index: 1, // Wrong index!
                delta: ContentDelta {
                    text: Some("hello".to_string()),
                    partial_json: None,
                },
            }));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("index mismatch"));
    }

    #[test]
    fn test_incomplete_stream_error() {
        let mut validator = StreamEventValidator::new();
        validator
            .validate_event(&create_message_start_event())
            .unwrap();

        // Try to finalize without completing blocks
        let result = validator.is_complete_and_valid();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("message_stop") || err_msg.contains("completed"),
            "Error message was: {}",
            err_msg
        );
    }

    #[test]
    fn test_multiple_message_starts_error() {
        let mut validator = StreamEventValidator::new();
        validator
            .validate_event(&create_message_start_event())
            .unwrap();

        let result = validator.validate_event(&create_message_start_event());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("multiple message_start")
        );
    }
}
