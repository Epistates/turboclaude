//! Mock transport for testing without real subprocess
//!
//! Allows testing agent behavior in isolation by simulating
//! the Claude CLI subprocess. Supports message capture, response queuing,
//! error injection, and delay simulation.

use serde_json::Value as JsonValue;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::Mutex;
use turboclaude_transport::ProcessConfig;
use turboclaude_transport::error::{Result as TransportResult, TransportError};

/// A mock transport for testing agent behavior in isolation
///
/// Provides the same interface as `CliTransport` but without spawning
/// a real subprocess. All sent messages are captured for assertion,
/// and responses are queued and returned on receive.
///
/// # Examples
///
/// ```rust,no_run
/// use common::mock_transport::MockTransport;
/// use serde_json::json;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut transport = MockTransport::new();
///
/// // Queue a response
/// transport.queue_response(json!({
///     "type": "control_request",
///     "request_id": "test-1",
///     "request": {
///         "subtype": "can_use_tool",
///         "tool_name": "TestTool",
///         "input": {"param": "value"}
///     }
/// }));
///
/// // Send a message (will be captured)
/// transport.send_message(json!({
///     "type": "user",
///     "content": "Hello"
/// })).await?;
///
/// // Receive queued response
/// let response = transport.recv_message().await?;
/// assert!(response.is_some());
///
/// // Assert on captured messages
/// let sent = transport.sent_messages();
/// assert_eq!(sent.len(), 1);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct MockTransport {
    /// Messages sent through the transport (for assertion)
    sent_messages: Arc<Mutex<Vec<JsonValue>>>,

    /// Queued responses to return on recv_message()
    queued_responses: Arc<Mutex<VecDeque<JsonValue>>>,

    /// Optional error to return on next operation
    next_error: Arc<Mutex<Option<TransportError>>>,

    /// Delay to simulate network/processing time
    delay: Option<Duration>,

    /// Whether the mock process is "alive"
    is_alive: Arc<AtomicBool>,

    /// Process configuration
    config: ProcessConfig,
}

impl MockTransport {
    /// Create a new MockTransport with default configuration
    pub fn new() -> Self {
        Self {
            sent_messages: Arc::new(Mutex::new(Vec::new())),
            queued_responses: Arc::new(Mutex::new(VecDeque::new())),
            next_error: Arc::new(Mutex::new(None)),
            delay: None,
            is_alive: Arc::new(AtomicBool::new(true)),
            config: ProcessConfig::default(),
        }
    }

    /// Create a builder for ergonomic test setup
    pub fn builder() -> MockTransportBuilder {
        MockTransportBuilder::default()
    }

    /// Queue a response to be returned on next recv_message()
    ///
    /// Responses are returned in FIFO order.
    pub async fn queue_response(&self, response: JsonValue) {
        self.queued_responses.lock().await.push_back(response);
    }

    /// Queue multiple responses
    #[allow(dead_code)]
    pub async fn queue_responses(&self, responses: Vec<JsonValue>) {
        let mut queue = self.queued_responses.lock().await;
        queue.extend(responses);
    }

    /// Get all sent messages for assertion
    ///
    /// Returns a clone of all messages sent via send_message().
    pub async fn sent_messages(&self) -> Vec<JsonValue> {
        self.sent_messages.lock().await.clone()
    }

    /// Clear all sent messages
    pub async fn clear_sent(&self) {
        self.sent_messages.lock().await.clear();
    }

    /// Clear all queued responses
    pub async fn clear_queued(&self) {
        self.queued_responses.lock().await.clear();
    }

    /// Inject an error to be returned on next operation
    ///
    /// The error will be returned once and then cleared.
    /// Applies to both send_message() and recv_message().
    pub async fn inject_error(&self, error: TransportError) {
        *self.next_error.lock().await = Some(error);
    }

    /// Set delay to simulate processing time
    ///
    /// If set, both send and receive operations will sleep for
    /// the specified duration before executing.
    pub fn set_delay(&mut self, delay: Duration) {
        self.delay = Some(delay);
    }

    /// Send a message through the mock transport
    ///
    /// Messages are captured and can be retrieved via sent_messages().
    /// If an error was injected, it will be returned instead.
    pub async fn send_message(&self, message: JsonValue) -> TransportResult<()> {
        // Check for injected error
        if let Some(error) = self.next_error.lock().await.take() {
            return Err(error);
        }

        // Simulate delay if configured
        if let Some(delay) = self.delay {
            tokio::time::sleep(delay).await;
        }

        // Check if alive
        if !self.is_alive.load(Ordering::SeqCst) {
            return Err(TransportError::Connection(
                "Transport is not alive".to_string(),
            ));
        }

        // Capture sent message
        self.sent_messages.lock().await.push(message);

        Ok(())
    }

    /// Receive a message from the mock transport
    ///
    /// Returns the next queued response, or None if the queue is empty.
    /// If an error was injected, it will be returned instead.
    pub async fn recv_message(&self) -> TransportResult<Option<JsonValue>> {
        // Check for injected error
        if let Some(error) = self.next_error.lock().await.take() {
            return Err(error);
        }

        // Simulate delay if configured
        if let Some(delay) = self.delay {
            tokio::time::sleep(delay).await;
        }

        // Check if alive
        if !self.is_alive.load(Ordering::SeqCst) {
            return Ok(None);
        }

        // Return queued response
        Ok(self.queued_responses.lock().await.pop_front())
    }

    /// Check if the mock process is alive
    pub async fn is_alive(&self) -> bool {
        self.is_alive.load(Ordering::SeqCst)
    }

    /// Kill the mock process
    ///
    /// Sets is_alive to false and clears all queues.
    pub async fn kill(&self) -> TransportResult<()> {
        self.is_alive.store(false, Ordering::SeqCst);
        self.sent_messages.lock().await.clear();
        self.queued_responses.lock().await.clear();
        Ok(())
    }

    /// Get the process configuration
    pub async fn config(&self) -> ProcessConfig {
        self.config.clone()
    }
}

impl Default for MockTransport {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for MockTransport with ergonomic test setup
///
/// # Examples
///
/// ```rust,no_run
/// use common::mock_transport::MockTransport;
/// use serde_json::json;
/// use std::time::Duration;
///
/// let transport = MockTransport::builder()
///     .with_response(json!({"type": "response", "data": "test"}))
///     .with_delay(Duration::from_millis(10))
///     .build();
/// ```
#[derive(Default)]
pub struct MockTransportBuilder {
    responses: Vec<JsonValue>,
    delay: Option<Duration>,
    config: Option<ProcessConfig>,
}

impl MockTransportBuilder {
    /// Add a single response to the queue
    pub fn with_response(mut self, response: JsonValue) -> Self {
        self.responses.push(response);
        self
    }

    /// Add multiple responses to the queue
    #[allow(dead_code)]
    pub fn with_responses(mut self, responses: Vec<JsonValue>) -> Self {
        self.responses.extend(responses);
        self
    }

    /// Set the delay for simulating processing time
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = Some(delay);
        self
    }

    /// Set the process configuration
    pub fn with_config(mut self, config: ProcessConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Build the MockTransport
    pub fn build(self) -> MockTransport {
        let mut transport = MockTransport {
            sent_messages: Arc::new(Mutex::new(Vec::new())),
            queued_responses: Arc::new(Mutex::new(VecDeque::from(self.responses))),
            next_error: Arc::new(Mutex::new(None)),
            delay: self.delay,
            is_alive: Arc::new(AtomicBool::new(true)),
            config: self.config.unwrap_or_default(),
        };

        if let Some(delay) = self.delay {
            transport.set_delay(delay);
        }

        transport
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_mock_transport_send_receive() {
        let transport = MockTransport::new();

        // Queue a response
        let response = json!({"type": "response", "data": "hello"});
        transport.queue_response(response.clone()).await;

        // Send a message
        let request = json!({"type": "request", "query": "hi"});
        transport.send_message(request.clone()).await.unwrap();

        // Verify sent
        let sent = transport.sent_messages().await;
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0], request);

        // Receive queued response
        let received = transport.recv_message().await.unwrap();
        assert_eq!(received, Some(response));
    }

    #[tokio::test]
    async fn test_mock_transport_error_injection() {
        let transport = MockTransport::new();

        // Inject error
        transport.inject_error(TransportError::Timeout).await;

        // Should fail
        let result = transport.send_message(json!({"test": "data"})).await;
        assert!(matches!(result, Err(TransportError::Timeout)));
    }

    #[tokio::test]
    async fn test_mock_transport_empty_queue() {
        let transport = MockTransport::new();

        // No responses queued
        let result = transport.recv_message().await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_mock_transport_kill() {
        let transport = MockTransport::new();

        // Queue some data
        transport.queue_response(json!({"test": "data"})).await;
        transport
            .send_message(json!({"test": "msg"}))
            .await
            .unwrap();

        // Kill the transport
        transport.kill().await.unwrap();

        // Should not be alive
        assert!(!transport.is_alive().await);

        // Queues should be cleared
        assert!(transport.sent_messages().await.is_empty());
        assert_eq!(transport.recv_message().await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_mock_transport_builder() {
        let transport = MockTransport::builder()
            .with_response(json!({"id": 1}))
            .with_response(json!({"id": 2}))
            .with_delay(Duration::from_millis(1))
            .build();

        assert!(transport.is_alive().await);

        // Should receive responses in order
        let msg1 = transport.recv_message().await.unwrap();
        assert_eq!(msg1, Some(json!({"id": 1})));

        let msg2 = transport.recv_message().await.unwrap();
        assert_eq!(msg2, Some(json!({"id": 2})));

        // Queue should be empty now
        let msg3 = transport.recv_message().await.unwrap();
        assert_eq!(msg3, None);
    }

    #[tokio::test]
    async fn test_mock_transport_clear_operations() {
        let transport = MockTransport::new();

        // Add some data
        transport.queue_response(json!({"test": "data"})).await;
        transport
            .send_message(json!({"test": "msg"}))
            .await
            .unwrap();

        // Clear sent
        transport.clear_sent().await;
        assert!(transport.sent_messages().await.is_empty());

        // Clear queued
        transport.clear_queued().await;
        let result = transport.recv_message().await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_mock_transport_delay() {
        use std::time::Instant;

        let mut transport = MockTransport::new();
        transport.set_delay(Duration::from_millis(50));
        transport.queue_response(json!({"test": "data"})).await;

        let start = Instant::now();
        transport
            .send_message(json!({"test": "msg"}))
            .await
            .unwrap();
        let send_elapsed = start.elapsed();

        let start = Instant::now();
        transport.recv_message().await.unwrap();
        let recv_elapsed = start.elapsed();

        // Both should take at least 50ms
        assert!(send_elapsed >= Duration::from_millis(50));
        assert!(recv_elapsed >= Duration::from_millis(50));
    }

    #[tokio::test]
    async fn test_mock_transport_multiple_errors() {
        let transport = MockTransport::new();

        // Inject error for send
        transport
            .inject_error(TransportError::Connection("test".to_string()))
            .await;
        let result = transport.send_message(json!({"test": "data"})).await;
        assert!(result.is_err());

        // Error should be consumed, next send should succeed
        transport
            .send_message(json!({"test": "data"}))
            .await
            .unwrap();

        // Inject different error for recv
        transport.inject_error(TransportError::Timeout).await;
        let result = transport.recv_message().await;
        assert!(matches!(result, Err(TransportError::Timeout)));
    }

    #[tokio::test]
    async fn test_mock_transport_config() {
        let custom_config = ProcessConfig {
            cli_path: "/custom/path/claude".into(),
            ..Default::default()
        };

        let transport = MockTransport::builder()
            .with_config(custom_config.clone())
            .build();

        let config = transport.config().await;
        assert_eq!(config.cli_path, custom_config.cli_path);
    }
}
