//! Streaming support for the Messages API
//!
//! This module provides streaming functionality similar to the Python SDK's
//! message streaming capabilities, using Server-Sent Events (SSE).

use futures::{Stream, StreamExt};
use pin_project::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};
use eventsource_stream::Eventsource;
use bytes::Bytes;

use crate::{
    error::{Error, Result},
    types::{Message, ContentBlock, Usage, StopReason},
};

/// A stream of message events.
///
/// This provides high-level streaming similar to the Python SDK's MessageStream.
#[pin_project]
pub struct MessageStream {
    #[pin]
    inner: Box<dyn Stream<Item = Result<StreamEvent>> + Send + Unpin>,
    /// Accumulated message being built from stream events
    message_builder: MessageBuilder,
}

impl MessageStream {
    /// Create a new message stream from an SSE response.
    pub(crate) fn new(response: impl Stream<Item = Result<Bytes>> + Send + Unpin + 'static) -> Self {
        let event_stream = response
            .eventsource()
            .map(|result| match result {
                Ok(event) => Self::parse_event(event),
                Err(e) => Err(Error::Streaming(e.to_string())),
            });

        Self {
            inner: Box::new(event_stream),
            message_builder: MessageBuilder::new(),
        }
    }

    /// Parse an SSE event into a StreamEvent.
    fn parse_event(event: eventsource_stream::Event) -> Result<StreamEvent> {
        // Parse based on event type
        match event.event.as_str() {
            "message_start" => {
                let data: MessageStartEvent = serde_json::from_str(&event.data)
                    .map_err(|e| Error::ResponseValidation(e.to_string()))?;
                Ok(StreamEvent::MessageStart(data))
            }
            "content_block_start" => {
                let data: ContentBlockStartEvent = serde_json::from_str(&event.data)
                    .map_err(|e| Error::ResponseValidation(e.to_string()))?;
                Ok(StreamEvent::ContentBlockStart(data))
            }
            "content_block_delta" => {
                let data: ContentBlockDeltaEvent = serde_json::from_str(&event.data)
                    .map_err(|e| Error::ResponseValidation(e.to_string()))?;
                Ok(StreamEvent::ContentBlockDelta(data))
            }
            "content_block_stop" => {
                let data: ContentBlockStopEvent = serde_json::from_str(&event.data)
                    .map_err(|e| Error::ResponseValidation(e.to_string()))?;
                Ok(StreamEvent::ContentBlockStop(data))
            }
            "message_delta" => {
                let data: MessageDeltaEvent = serde_json::from_str(&event.data)
                    .map_err(|e| Error::ResponseValidation(e.to_string()))?;
                Ok(StreamEvent::MessageDelta(data))
            }
            "message_stop" => {
                Ok(StreamEvent::MessageStop)
            }
            "ping" => {
                Ok(StreamEvent::Ping)
            }
            "error" => {
                let error: StreamError = serde_json::from_str(&event.data)
                    .map_err(|e| Error::ResponseValidation(e.to_string()))?;
                Err(Error::Streaming(format!("{}: {}", error.error_type, error.message)))
            }
            _ => {
                // Unknown event type, skip
                Ok(StreamEvent::Unknown)
            }
        }
    }

    /// Get a stream of just the text content.
    ///
    /// This is a convenience method similar to the Python SDK's text_stream.
    pub fn text_stream(self) -> impl Stream<Item = Result<String>> {
        self.filter_map(|result| async move {
            match result {
                Ok(StreamEvent::ContentBlockDelta(delta)) => {
                    delta.delta.text.map(Ok)
                }
                Err(e) => Some(Err(e)),
                _ => None,
            }
        })
    }

    /// Collect all events and reconstruct the final message.
    ///
    /// This is similar to the Python SDK's get_final_message().
    pub async fn get_final_message(mut self) -> Result<Message> {
        while let Some(event) = self.next().await {
            match event? {
                StreamEvent::MessageStart(start) => {
                    self.message_builder.set_message_start(start);
                }
                StreamEvent::ContentBlockStart(start) => {
                    self.message_builder.add_content_block_start(start);
                }
                StreamEvent::ContentBlockDelta(delta) => {
                    self.message_builder.add_content_block_delta(delta);
                }
                StreamEvent::ContentBlockStop(_) => {
                    self.message_builder.finalize_current_block();
                }
                StreamEvent::MessageDelta(delta) => {
                    self.message_builder.set_message_delta(delta);
                }
                StreamEvent::MessageStop => {
                    break;
                }
                _ => {}
            }
        }

        self.message_builder.build()
    }
}

impl Stream for MessageStream {
    type Item = Result<StreamEvent>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.inner.poll_next(cx)
    }
}

/// Events that can be received from a message stream.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// Start of a new message
    MessageStart(MessageStartEvent),
    /// Start of a content block
    ContentBlockStart(ContentBlockStartEvent),
    /// Delta update for a content block
    ContentBlockDelta(ContentBlockDeltaEvent),
    /// End of a content block
    ContentBlockStop(ContentBlockStopEvent),
    /// Delta update for the message
    MessageDelta(MessageDeltaEvent),
    /// End of the message
    MessageStop,
    /// Ping event to keep connection alive
    Ping,
    /// Unknown event type
    Unknown,
}

/// Message start event.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct MessageStartEvent {
    /// The message being started
    pub message: PartialMessage,
}

/// Content block start event.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ContentBlockStartEvent {
    /// Index of the content block
    pub index: usize,
    /// The content block being started
    pub content_block: PartialContentBlock,
}

/// Content block delta event.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ContentBlockDeltaEvent {
    /// Index of the content block
    pub index: usize,
    /// Delta update for the content block
    pub delta: ContentDelta,
}

/// Content block stop event.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ContentBlockStopEvent {
    /// Index of the content block that stopped
    pub index: usize,
}

/// Message delta event.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct MessageDeltaEvent {
    /// Delta update for the message
    pub delta: MessageDelta,
    /// Updated usage statistics (delta only, may contain only output_tokens)
    pub usage: Option<DeltaUsage>,
}

/// Usage statistics in delta events (may be partial).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct DeltaUsage {
    /// Number of output tokens (usually only this is present in deltas)
    pub output_tokens: u32,
}

/// Partial message during streaming.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PartialMessage {
    /// Message ID
    pub id: String,
    /// Message type
    #[serde(rename = "type")]
    pub message_type: String,
    /// Role of the message
    pub role: String,
    /// Model used
    pub model: String,
    /// Content blocks
    pub content: Vec<ContentBlock>,
    /// Stop reason if finished
    pub stop_reason: Option<StopReason>,
    /// Stop sequence that triggered stop
    pub stop_sequence: Option<String>,
    /// Token usage statistics
    pub usage: Option<Usage>,
}

/// Partial content block during streaming.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type")]
pub enum PartialContentBlock {
    /// Text content block
    #[serde(rename = "text")]
    Text {
        /// Text content
        text: String
    },
    /// Tool use block
    #[serde(rename = "tool_use")]
    ToolUse {
        /// Tool use ID
        id: String,
        /// Tool name
        name: String,
        /// Input JSON (as object - will be accumulated from deltas)
        input: serde_json::Value,
    },
}

/// Delta for content blocks.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ContentDelta {
    /// Text delta if this is a text block
    pub text: Option<String>,
    /// JSON string delta if this is a tool use block
    pub partial_json: Option<String>,
}

/// Delta for messages.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct MessageDelta {
    /// Stop reason if message finished
    pub stop_reason: Option<StopReason>,
    /// Stop sequence that triggered the stop
    pub stop_sequence: Option<String>,
}

/// Error from the stream.
#[derive(Debug, serde::Deserialize)]
struct StreamError {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

/// Builder for reconstructing a message from stream events.
struct MessageBuilder {
    id: Option<String>,
    model: Option<String>,
    content_blocks: Vec<ContentBlock>,
    current_block: Option<(usize, String)>,
    stop_reason: Option<StopReason>,
    stop_sequence: Option<String>,
    usage: Option<Usage>,
}

impl MessageBuilder {
    fn new() -> Self {
        Self {
            id: None,
            model: None,
            content_blocks: Vec::new(),
            current_block: None,
            stop_reason: None,
            stop_sequence: None,
            usage: None,
        }
    }

    fn set_message_start(&mut self, start: MessageStartEvent) {
        self.id = Some(start.message.id);
        self.model = Some(start.message.model);
        self.usage = start.message.usage;
    }

    fn add_content_block_start(&mut self, start: ContentBlockStartEvent) {
        match start.content_block {
            PartialContentBlock::Text { text } => {
                self.current_block = Some((start.index, text));
            }
            PartialContentBlock::ToolUse { .. } => {
                // Handle tool use blocks
                self.current_block = Some((start.index, String::new()));
            }
        }
    }

    fn add_content_block_delta(&mut self, delta: ContentBlockDeltaEvent) {
        if let Some((idx, ref mut text)) = self.current_block {
            if idx == delta.index {
                if let Some(delta_text) = delta.delta.text {
                    text.push_str(&delta_text);
                } else if let Some(json) = delta.delta.partial_json {
                    text.push_str(&json);
                }
            }
        }
    }

    fn finalize_current_block(&mut self) {
        if let Some((_, text)) = self.current_block.take() {
            self.content_blocks.push(ContentBlock::Text { text });
        }
    }

    fn set_message_delta(&mut self, delta: MessageDeltaEvent) {
        if delta.delta.stop_reason.is_some() {
            self.stop_reason = delta.delta.stop_reason;
        }
        if delta.delta.stop_sequence.is_some() {
            self.stop_sequence = delta.delta.stop_sequence;
        }
        // Update usage with delta (only output_tokens changes in deltas)
        if let Some(delta_usage) = delta.usage {
            if let Some(ref mut usage) = self.usage {
                usage.output_tokens = delta_usage.output_tokens;
            }
        }
    }

    fn build(mut self) -> Result<Message> {
        // Finalize any pending block
        self.finalize_current_block();

        Ok(Message {
            id: self.id.ok_or_else(|| Error::Streaming("Missing message ID".to_string()))?,
            message_type: "message".to_string(),
            role: crate::types::Role::Assistant,
            content: self.content_blocks,
            model: self.model.ok_or_else(|| Error::Streaming("Missing model".to_string()))?,
            stop_reason: self.stop_reason,
            stop_sequence: self.stop_sequence,
            usage: self.usage.ok_or_else(|| Error::Streaming("Missing usage".to_string()))?,
        })
    }
}