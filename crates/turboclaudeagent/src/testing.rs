//! Testing utilities for integration tests
//!
//! Provides mock transport and fixtures for testing AgentSession
//! without requiring a real Claude CLI process.

use std::sync::Arc;
use tokio::sync::Mutex;
use turboclaude_protocol::ProtocolMessage;

/// Configuration for mock CLI behavior
#[derive(Debug, Clone, Default)]
pub struct MockConfig {
    /// Whether to simulate hook requests during query execution
    pub simulate_hooks: bool,

    /// Whether to simulate permission checks during query execution
    pub simulate_permissions: bool,

    /// Optional delay to simulate network latency
    pub response_delay: Option<std::time::Duration>,

    /// If set, fail after receiving N messages
    pub fail_after_n_messages: Option<usize>,
}

/// Mock transport for testing that simulates CLI behavior
///
/// Allows tests to:
/// - Queue responses for queries
/// - Track sent messages
/// - Simulate hooks and permissions
/// - Simulate network delays and failures
#[derive(Clone)]
pub struct MockCliTransport {
    /// Queued responses to send
    response_queue: Arc<Mutex<Vec<ProtocolMessage>>>,

    /// Messages that have been sent
    sent_messages: Arc<Mutex<Vec<ProtocolMessage>>>,

    /// Configuration for mock behavior
    config: MockConfig,

    /// Counter for message tracking
    message_count: Arc<Mutex<usize>>,
}

impl MockCliTransport {
    /// Create a new mock transport with default configuration
    pub fn new() -> Self {
        Self {
            response_queue: Arc::new(Mutex::new(Vec::new())),
            sent_messages: Arc::new(Mutex::new(Vec::new())),
            config: MockConfig::default(),
            message_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Create a new mock transport with custom configuration
    pub fn with_config(config: MockConfig) -> Self {
        Self {
            response_queue: Arc::new(Mutex::new(Vec::new())),
            sent_messages: Arc::new(Mutex::new(Vec::new())),
            config,
            message_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Enqueue a response to be returned by recv_message()
    pub async fn enqueue_response(&self, message: ProtocolMessage) {
        self.response_queue.lock().await.push(message);
    }

    /// Get all messages that were sent via send_message()
    pub async fn sent_messages(&self) -> Vec<ProtocolMessage> {
        self.sent_messages.lock().await.clone()
    }

    /// Clear sent messages tracking
    pub async fn clear_sent_messages(&self) {
        self.sent_messages.lock().await.clear();
    }

    /// Get the count of messages received so far
    pub async fn message_count(&self) -> usize {
        *self.message_count.lock().await
    }
}

impl Default for MockCliTransport {
    fn default() -> Self {
        Self::new()
    }
}

// Implement the transport interface
impl MockCliTransport {
    /// Send a message (simulates sending to CLI)
    pub async fn send_message(
        &self,
        message: serde_json::Value,
    ) -> turboclaude_transport::Result<()> {
        // Check if we should fail
        let mut count = self.message_count.lock().await;
        *count += 1;

        if let Some(fail_after) = self.config.fail_after_n_messages
            && *count > fail_after
        {
            return Err(turboclaude_transport::TransportError::Other(
                "Mock transport configured to fail".to_string(),
            ));
        }

        // Track the message
        if let Ok(json_str) = serde_json::to_string(&message)
            && let Ok(parsed) = ProtocolMessage::from_json(&json_str)
        {
            self.sent_messages.lock().await.push(parsed);
        }

        // Simulate delay if configured
        if let Some(delay) = self.config.response_delay {
            tokio::time::sleep(delay).await;
        }

        Ok(())
    }

    /// Receive a message (returns queued responses)
    pub async fn recv_message(&self) -> turboclaude_transport::Result<Option<serde_json::Value>> {
        // Simulate delay if configured
        if let Some(delay) = self.config.response_delay {
            tokio::time::sleep(delay).await;
        }

        // Get next response from queue
        let mut queue = self.response_queue.lock().await;
        if let Some(message) = queue.pop() {
            let json = message.to_json().map_err(|e| {
                turboclaude_transport::TransportError::Serialization(format!("{}", e))
            })?;
            Ok(Some(serde_json::from_str(&json).map_err(|e| {
                turboclaude_transport::TransportError::Serialization(format!("{}", e))
            })?))
        } else {
            Ok(None)
        }
    }

    /// Check if transport is alive (always true for mock)
    pub async fn is_alive(&self) -> bool {
        true
    }

    /// Kill the transport (no-op for mock)
    pub async fn kill(&self) -> turboclaude_transport::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use turboclaude_protocol::ProtocolErrorMessage;

    #[tokio::test]
    async fn test_mock_transport_send_recv() {
        let mock = MockCliTransport::new();

        // Create a simple response
        let response = turboclaude_protocol::QueryResponse {
            message: turboclaude_protocol::Message {
                id: "msg_123".to_string(),
                message_type: "message".to_string(),
                role: turboclaude_protocol::message::MessageRole::Assistant,
                content: vec![turboclaude_protocol::ContentBlock::Text {
                    text: "test response".to_string(),
                }],
                model: "claude-3-5-sonnet".to_string(),
                stop_reason: turboclaude_protocol::types::StopReason::EndTurn,
                stop_sequence: None,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                usage: turboclaude_protocol::types::Usage {
                    input_tokens: 10,
                    output_tokens: 5,
                },
                cache_usage: Default::default(),
            },
            is_complete: true,
        };

        // Enqueue response
        mock.enqueue_response(turboclaude_protocol::ProtocolMessage::Response(response))
            .await;

        // Receive it
        let msg = mock.recv_message().await.unwrap();
        assert!(msg.is_some());
    }

    #[tokio::test]
    async fn test_mock_transport_tracking() {
        let mock = MockCliTransport::new();

        // Send a valid ProtocolMessage (error message)
        let error_msg = ProtocolMessage::Error(ProtocolErrorMessage {
            code: "test_error".to_string(),
            message: "Test error message".to_string(),
            details: None,
        });

        let json = error_msg.to_json().unwrap();
        let test_json: serde_json::Value = serde_json::from_str(&json).unwrap();

        mock.send_message(test_json).await.unwrap();

        // Verify it was tracked
        let sent = mock.sent_messages().await;
        assert!(!sent.is_empty());
    }

    #[tokio::test]
    async fn test_mock_transport_fail_after() {
        let config = MockConfig {
            fail_after_n_messages: Some(2),
            ..Default::default()
        };
        let mock = MockCliTransport::with_config(config);

        let test_json = serde_json::json!({"type": "test"});

        // First message should succeed
        assert!(mock.send_message(test_json.clone()).await.is_ok());

        // Second message should succeed
        assert!(mock.send_message(test_json.clone()).await.is_ok());

        // Third message should fail
        assert!(mock.send_message(test_json).await.is_err());
    }

    #[tokio::test]
    async fn test_mock_transport_delay() {
        use std::time::Instant;

        let config = MockConfig {
            response_delay: Some(std::time::Duration::from_millis(50)),
            ..Default::default()
        };
        let mock = MockCliTransport::with_config(config);

        let start = Instant::now();
        let test_json = serde_json::json!({"type": "test"});
        mock.send_message(test_json).await.unwrap();
        let elapsed = start.elapsed();

        // Should have delayed at least 50ms
        assert!(elapsed >= std::time::Duration::from_millis(50));
    }

    #[tokio::test]
    async fn test_mock_transport_is_alive() {
        let mock = MockCliTransport::new();
        assert!(mock.is_alive().await);
    }

    #[tokio::test]
    async fn test_mock_transport_kill() {
        let mock = MockCliTransport::new();
        assert!(mock.kill().await.is_ok());
        // Still alive for mock
        assert!(mock.is_alive().await);
    }
}
