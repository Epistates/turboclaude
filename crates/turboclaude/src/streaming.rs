//! Streaming support for the Messages API
//!
//! This module provides streaming functionality similar to the Python SDK's
//! message streaming capabilities, using Server-Sent Events (SSE).

use bytes::Bytes;
use eventsource_stream::Eventsource;
use futures::{Stream, StreamExt};
use pin_project::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;
use tracing::{debug, info, warn};

use crate::{
    error::{Error, Result},
    observability::StreamContext,
    types::{ContentBlock, Message, StopReason, Usage},
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
    /// Streaming context for observability
    stream_context: StreamContext,
    /// Start time of stream for duration tracking
    start_time: Instant,
}

impl MessageStream {
    /// Create a new message stream from an SSE response.
    pub(crate) fn new(
        response: impl Stream<Item = Result<Bytes>> + Send + Unpin + 'static,
    ) -> Self {
        StreamContext::log_started("/v1/messages");

        let event_stream = response.eventsource().map(|result| match result {
            Ok(event) => Self::parse_event(event),
            Err(e) => {
                warn!("Stream error during event parsing: {}", e);
                Err(Error::Streaming(e.to_string()))
            }
        });

        Self {
            inner: Box::new(event_stream),
            message_builder: MessageBuilder::new(),
            stream_context: StreamContext::new(),
            start_time: Instant::now(),
        }
    }

    /// Parse an SSE event into a StreamEvent.
    fn parse_event(event: eventsource_stream::Event) -> Result<StreamEvent> {
        // Parse based on event type
        match event.event.as_str() {
            "message_start" => {
                debug!("Parsing message_start event");
                let data: MessageStartEvent = serde_json::from_str(&event.data).map_err(|e| {
                    warn!("Failed to parse message_start: {}", e);
                    Error::ResponseValidation(e.to_string())
                })?;
                debug!(message_id = %data.message.id, "Message started");
                Ok(StreamEvent::MessageStart(data))
            }
            "content_block_start" => {
                debug!(event_type = "content_block_start", "Parsing stream event");
                let data: ContentBlockStartEvent =
                    serde_json::from_str(&event.data).map_err(|e| {
                        warn!("Failed to parse content_block_start: {}", e);
                        Error::ResponseValidation(e.to_string())
                    })?;
                debug!(block_index = data.index, "Content block started");
                Ok(StreamEvent::ContentBlockStart(data))
            }
            "content_block_delta" => {
                let data: ContentBlockDeltaEvent =
                    serde_json::from_str(&event.data).map_err(|e| {
                        warn!("Failed to parse content_block_delta: {}", e);
                        Error::ResponseValidation(e.to_string())
                    })?;
                Ok(StreamEvent::ContentBlockDelta(data))
            }
            "content_block_stop" => {
                debug!(event_type = "content_block_stop", "Parsing stream event");
                let data: ContentBlockStopEvent =
                    serde_json::from_str(&event.data).map_err(|e| {
                        warn!("Failed to parse content_block_stop: {}", e);
                        Error::ResponseValidation(e.to_string())
                    })?;
                debug!(block_index = data.index, "Content block stopped");
                Ok(StreamEvent::ContentBlockStop(data))
            }
            "message_delta" => {
                let data: MessageDeltaEvent = serde_json::from_str(&event.data).map_err(|e| {
                    warn!("Failed to parse message_delta: {}", e);
                    Error::ResponseValidation(e.to_string())
                })?;
                if let Some(stop_reason) = &data.delta.stop_reason {
                    debug!(stop_reason = ?stop_reason, "Message stopping");
                }
                Ok(StreamEvent::MessageDelta(data))
            }
            "message_stop" => {
                debug!("Message stream completed");
                Ok(StreamEvent::MessageStop)
            }
            "ping" => {
                debug!("Received ping to keep connection alive");
                Ok(StreamEvent::Ping)
            }
            "error" => {
                let error: StreamError = serde_json::from_str(&event.data)
                    .map_err(|e| Error::ResponseValidation(e.to_string()))?;
                warn!(
                    error_type = %error.error_type,
                    error_message = %error.message,
                    "Stream error received"
                );
                Err(Error::Streaming(format!(
                    "{}: {}",
                    error.error_type, error.message
                )))
            }
            _ => {
                debug!(unknown_event = %event.event, "Received unknown event type");
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
                Ok(StreamEvent::ContentBlockDelta(delta)) => delta.delta.text.map(Ok),
                Err(e) => Some(Err(e)),
                _ => None,
            }
        })
    }

    /// Collect all events and reconstruct the final message.
    ///
    /// This is similar to the Python SDK's get_final_message().
    pub async fn get_final_message(mut self) -> Result<Message> {
        debug!("Starting message reconstruction from stream");

        while let Some(event) = self.next().await {
            match event {
                Ok(stream_event) => match &stream_event {
                    StreamEvent::MessageStart(start) => {
                        self.stream_context.log_event("MessageStart");
                        self.message_builder.set_message_start(start.clone());
                    }
                    StreamEvent::ContentBlockStart(start) => {
                        self.stream_context.log_event("ContentBlockStart");
                        self.message_builder.add_content_block_start(start.clone());
                    }
                    StreamEvent::ContentBlockDelta(_) => {
                        self.stream_context.log_event("ContentBlockDelta");
                        if let StreamEvent::ContentBlockDelta(delta) = stream_event {
                            self.message_builder.add_content_block_delta(delta);
                        }
                    }
                    StreamEvent::ContentBlockStop(_) => {
                        self.stream_context.log_event("ContentBlockStop");
                        self.message_builder.finalize_current_block();
                    }
                    StreamEvent::MessageDelta(delta) => {
                        self.stream_context.log_event("MessageDelta");
                        self.message_builder.set_message_delta(delta.clone());
                    }
                    StreamEvent::MessageStop => {
                        self.stream_context.log_event("MessageStop");
                        break;
                    }
                    StreamEvent::Ping => {
                        debug!("Ping event received, keeping connection alive");
                    }
                    StreamEvent::Unknown => {
                        debug!("Unknown stream event received");
                    }
                },
                Err(e) => {
                    let elapsed = self.start_time.elapsed();
                    warn!(
                        error = %e,
                        elapsed_ms = elapsed.as_millis(),
                        event_count = self.stream_context.event_count,
                        "Stream event processing failed"
                    );
                    self.stream_context
                        .log_error("/v1/messages", &e.to_string());
                    return Err(e);
                }
            }
        }

        let elapsed = self.start_time.elapsed();
        match self.message_builder.build() {
            Ok(message) => {
                info!(
                    message_id = %message.id,
                    event_count = self.stream_context.event_count,
                    elapsed_ms = elapsed.as_millis(),
                    output_tokens = message.usage.output_tokens,
                    "Stream message reconstruction complete"
                );
                Ok(message)
            }
            Err(e) => {
                warn!(
                    error = %e,
                    event_count = self.stream_context.event_count,
                    elapsed_ms = elapsed.as_millis(),
                    "Stream message reconstruction failed"
                );
                self.stream_context
                    .log_error("/v1/messages", &e.to_string());
                Err(e)
            }
        }
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
        text: String,
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
        if let Some((idx, ref mut text)) = self.current_block
            && idx == delta.index
        {
            if let Some(delta_text) = delta.delta.text {
                text.push_str(&delta_text);
            } else if let Some(json) = delta.delta.partial_json {
                text.push_str(&json);
            }
        }
    }

    fn finalize_current_block(&mut self) {
        if let Some((_, text)) = self.current_block.take() {
            self.content_blocks.push(ContentBlock::Text {
                text,
                citations: None,
            });
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
        if let Some(delta_usage) = delta.usage
            && let Some(ref mut usage) = self.usage
        {
            usage.output_tokens = delta_usage.output_tokens;
        }
    }

    fn build(mut self) -> Result<Message> {
        // Finalize any pending block
        self.finalize_current_block();

        Ok(Message {
            id: self
                .id
                .ok_or_else(|| Error::Streaming("Missing message ID".to_string()))?,
            message_type: "message".to_string(),
            role: crate::types::Role::Assistant,
            content: self.content_blocks,
            model: self
                .model
                .ok_or_else(|| Error::Streaming("Missing model".to_string()))?,
            stop_reason: self.stop_reason,
            stop_sequence: self.stop_sequence,
            usage: self
                .usage
                .ok_or_else(|| Error::Streaming("Missing usage".to_string()))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use futures::{StreamExt, stream};

    /// Test 1: MessageStream::new() creates stream from SSE response
    #[tokio::test]
    async fn test_message_stream_from_sse() {
        let sse_data = vec![
            Ok(Bytes::from(
                "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_123\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"claude-3-5-sonnet-20241022\",\"content\":[],\"stop_reason\":null,\"stop_sequence\":null,\"usage\":{\"input_tokens\":10,\"output_tokens\":0}}}\n\n",
            )),
            Ok(Bytes::from(
                "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n",
            )),
        ];

        let byte_stream = stream::iter(sse_data);
        let mut msg_stream = MessageStream::new(byte_stream);

        // Should successfully receive message_start event
        let first_event = msg_stream.next().await;
        assert!(first_event.is_some());
        assert!(matches!(
            first_event.unwrap(),
            Ok(StreamEvent::MessageStart(_))
        ));

        // Should successfully receive message_stop event
        let second_event = msg_stream.next().await;
        assert!(second_event.is_some());
        assert!(matches!(
            second_event.unwrap(),
            Ok(StreamEvent::MessageStop)
        ));
    }

    /// Test 2: Parse message_start event correctly
    #[test]
    fn test_parse_event_message_start() {
        let event = eventsource_stream::Event {
            event: "message_start".to_string(),
            data: r#"{
                "type": "message_start",
                "message": {
                    "id": "msg_123",
                    "type": "message",
                    "role": "assistant",
                    "model": "claude-3-5-sonnet-20241022",
                    "content": [],
                    "stop_reason": null,
                    "stop_sequence": null,
                    "usage": {
                        "input_tokens": 10,
                        "output_tokens": 0
                    }
                }
            }"#
            .to_string(),
            id: String::new(),
            retry: None,
        };

        let result = MessageStream::parse_event(event);
        assert!(result.is_ok());
        match result.unwrap() {
            StreamEvent::MessageStart(start) => {
                assert_eq!(start.message.id, "msg_123");
                assert_eq!(start.message.model, "claude-3-5-sonnet-20241022");
                assert_eq!(start.message.role, "assistant");
            }
            _ => panic!("Expected MessageStart event"),
        }
    }

    /// Test 3: Parse content_block_start event
    #[test]
    fn test_parse_event_content_block_start() {
        let event = eventsource_stream::Event {
            event: "content_block_start".to_string(),
            data: r#"{
                "type": "content_block_start",
                "index": 0,
                "content_block": {
                    "type": "text",
                    "text": ""
                }
            }"#
            .to_string(),
            id: String::new(),
            retry: None,
        };

        let result = MessageStream::parse_event(event);
        assert!(result.is_ok());
        match result.unwrap() {
            StreamEvent::ContentBlockStart(start) => {
                assert_eq!(start.index, 0);
                match start.content_block {
                    PartialContentBlock::Text { text } => {
                        assert_eq!(text, "");
                    }
                    _ => panic!("Expected Text content block"),
                }
            }
            _ => panic!("Expected ContentBlockStart event"),
        }
    }

    /// Test 4: Parse content_block_delta event with text
    #[test]
    fn test_parse_event_content_block_delta() {
        let event = eventsource_stream::Event {
            event: "content_block_delta".to_string(),
            data: r#"{
                "type": "content_block_delta",
                "index": 0,
                "delta": {
                    "type": "text_delta",
                    "text": "Hello, "
                }
            }"#
            .to_string(),
            id: String::new(),
            retry: None,
        };

        let result = MessageStream::parse_event(event);
        assert!(result.is_ok());
        match result.unwrap() {
            StreamEvent::ContentBlockDelta(delta) => {
                assert_eq!(delta.index, 0);
                assert_eq!(delta.delta.text, Some("Hello, ".to_string()));
                assert_eq!(delta.delta.partial_json, None);
            }
            _ => panic!("Expected ContentBlockDelta event"),
        }
    }

    /// Test 5: Parse content_block_stop event
    #[test]
    fn test_parse_event_content_block_stop() {
        let event = eventsource_stream::Event {
            event: "content_block_stop".to_string(),
            data: r#"{
                "type": "content_block_stop",
                "index": 0
            }"#
            .to_string(),
            id: String::new(),
            retry: None,
        };

        let result = MessageStream::parse_event(event);
        assert!(result.is_ok());
        match result.unwrap() {
            StreamEvent::ContentBlockStop(stop) => {
                assert_eq!(stop.index, 0);
            }
            _ => panic!("Expected ContentBlockStop event"),
        }
    }

    /// Test 6: Parse message_delta event
    #[test]
    fn test_parse_event_message_delta() {
        let event = eventsource_stream::Event {
            event: "message_delta".to_string(),
            data: r#"{
                "type": "message_delta",
                "delta": {
                    "stop_reason": "end_turn",
                    "stop_sequence": null
                },
                "usage": {
                    "output_tokens": 15
                }
            }"#
            .to_string(),
            id: String::new(),
            retry: None,
        };

        let result = MessageStream::parse_event(event);
        assert!(result.is_ok());
        match result.unwrap() {
            StreamEvent::MessageDelta(delta) => {
                assert_eq!(delta.delta.stop_reason, Some(StopReason::EndTurn));
                assert!(delta.usage.is_some());
                assert_eq!(delta.usage.unwrap().output_tokens, 15);
            }
            _ => panic!("Expected MessageDelta event"),
        }
    }

    /// Test 7: Parse message_stop event
    #[test]
    fn test_parse_event_message_stop() {
        let event = eventsource_stream::Event {
            event: "message_stop".to_string(),
            data: r#"{"type": "message_stop"}"#.to_string(),
            id: String::new(),
            retry: None,
        };

        let result = MessageStream::parse_event(event);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), StreamEvent::MessageStop));
    }

    /// Test 8: Parse error event
    #[test]
    fn test_parse_event_error() {
        let event = eventsource_stream::Event {
            event: "error".to_string(),
            data: r#"{
                "type": "overloaded_error",
                "message": "Service temporarily overloaded"
            }"#
            .to_string(),
            id: String::new(),
            retry: None,
        };

        let result = MessageStream::parse_event(event);
        assert!(result.is_err());
        match result {
            Err(Error::Streaming(msg)) => {
                assert!(msg.contains("overloaded_error"));
                assert!(msg.contains("Service temporarily overloaded"));
            }
            _ => panic!("Expected Streaming error"),
        }
    }

    /// Test 9: Reconstruct final message from stream events
    #[tokio::test]
    async fn test_get_final_message_reconstruction() {
        let sse_data = vec![
            Ok(Bytes::from(
                "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_123\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"claude-3-5-sonnet-20241022\",\"content\":[],\"stop_reason\":null,\"stop_sequence\":null,\"usage\":{\"input_tokens\":10,\"output_tokens\":0}}}\n\n",
            )),
            Ok(Bytes::from(
                "event: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n",
            )),
            Ok(Bytes::from(
                "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}\n\n",
            )),
            Ok(Bytes::from(
                "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\" world\"}}\n\n",
            )),
            Ok(Bytes::from(
                "event: content_block_stop\ndata: {\"type\":\"content_block_stop\",\"index\":0}\n\n",
            )),
            Ok(Bytes::from(
                "event: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\",\"stop_sequence\":null},\"usage\":{\"output_tokens\":2}}\n\n",
            )),
            Ok(Bytes::from(
                "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n",
            )),
        ];

        let byte_stream = stream::iter(sse_data);
        let msg_stream = MessageStream::new(byte_stream);

        let final_message = msg_stream.get_final_message().await;
        assert!(final_message.is_ok());

        let message = final_message.unwrap();
        assert_eq!(message.id, "msg_123");
        assert_eq!(message.model, "claude-3-5-sonnet-20241022");
        assert_eq!(message.content.len(), 1);

        // Verify text was concatenated correctly
        match &message.content[0] {
            ContentBlock::Text { text, .. } => {
                assert_eq!(text, "Hello world");
            }
            _ => panic!("Expected Text content block"),
        }

        assert_eq!(message.stop_reason, Some(StopReason::EndTurn));
    }

    /// Test 10: text_stream() filters only text content
    #[tokio::test]
    async fn test_text_stream_filtering() {
        let sse_data = vec![
            Ok(Bytes::from(
                "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_123\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"claude-3-5-sonnet-20241022\",\"content\":[],\"stop_reason\":null,\"stop_sequence\":null,\"usage\":{\"input_tokens\":10,\"output_tokens\":0}}}\n\n",
            )),
            Ok(Bytes::from(
                "event: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n",
            )),
            Ok(Bytes::from(
                "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}\n\n",
            )),
            Ok(Bytes::from(
                "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\" world\"}}\n\n",
            )),
            Ok(Bytes::from("event: ping\ndata: {\"type\":\"ping\"}\n\n")),
            Ok(Bytes::from(
                "event: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\",\"stop_sequence\":null},\"usage\":{\"output_tokens\":2}}\n\n",
            )),
        ];

        let byte_stream = stream::iter(sse_data);
        let msg_stream = MessageStream::new(byte_stream);

        let text_stream = msg_stream.text_stream();
        let mut text_stream = Box::pin(text_stream);
        let mut collected_text = Vec::new();

        while let Some(result) = text_stream.next().await {
            if let Ok(text) = result {
                collected_text.push(text);
            }
        }

        // Should only get the text deltas, not other events
        assert_eq!(collected_text.len(), 2);
        assert_eq!(collected_text[0], "Hello");
        assert_eq!(collected_text[1], " world");
    }

    /// Test 11: MessageBuilder state machine transitions
    #[test]
    fn test_message_builder_state_machine() {
        let mut builder = MessageBuilder::new();

        // Initially empty
        assert!(builder.id.is_none());
        assert!(builder.model.is_none());
        assert_eq!(builder.content_blocks.len(), 0);

        // Set message start
        let start = MessageStartEvent {
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
        };
        builder.set_message_start(start);
        assert_eq!(builder.id, Some("msg_123".to_string()));
        assert_eq!(
            builder.model,
            Some("claude-3-5-sonnet-20241022".to_string())
        );

        // Start a content block
        let block_start = ContentBlockStartEvent {
            index: 0,
            content_block: PartialContentBlock::Text {
                text: String::new(),
            },
        };
        builder.add_content_block_start(block_start);
        assert!(builder.current_block.is_some());

        // Add deltas
        let delta1 = ContentBlockDeltaEvent {
            index: 0,
            delta: ContentDelta {
                text: Some("Hello".to_string()),
                partial_json: None,
            },
        };
        builder.add_content_block_delta(delta1);

        let delta2 = ContentBlockDeltaEvent {
            index: 0,
            delta: ContentDelta {
                text: Some(" world".to_string()),
                partial_json: None,
            },
        };
        builder.add_content_block_delta(delta2);

        // Verify delta was accumulated
        assert!(builder.current_block.is_some());
        let (_, text) = builder.current_block.as_ref().unwrap();
        assert_eq!(text, "Hello world");

        // Finalize block
        builder.finalize_current_block();
        assert!(builder.current_block.is_none());
        assert_eq!(builder.content_blocks.len(), 1);

        // Set message delta
        let msg_delta = MessageDeltaEvent {
            delta: MessageDelta {
                stop_reason: Some(StopReason::EndTurn),
                stop_sequence: None,
            },
            usage: Some(DeltaUsage { output_tokens: 2 }),
        };
        builder.set_message_delta(msg_delta);
        assert_eq!(builder.stop_reason, Some(StopReason::EndTurn));

        // Build final message
        let result = builder.build();
        assert!(result.is_ok());
        let message = result.unwrap();
        assert_eq!(message.id, "msg_123");
        assert_eq!(message.content.len(), 1);
    }

    /// Test 12: Unknown event types are handled gracefully
    #[test]
    fn test_streaming_unknown_event() {
        let event = eventsource_stream::Event {
            event: "unknown_future_event".to_string(),
            data: r#"{"type": "unknown", "data": "something"}"#.to_string(),
            id: String::new(),
            retry: None,
        };

        let result = MessageStream::parse_event(event);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), StreamEvent::Unknown));
    }
}
