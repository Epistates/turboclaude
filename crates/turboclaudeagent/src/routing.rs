//! Message router for protocol message handling
//!
//! Manages bidirectional protocol communication with the Claude Code CLI.
//! Routes incoming messages to hooks/permissions and manages request/response correlation.

use crate::error::Result as AgentResult;
use crate::hooks::HookRegistry;
use crate::permissions::PermissionEvaluator;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio::time::{Duration, timeout};
use turboclaude_protocol::{
    HookRequest, PermissionCheckRequest, ProtocolMessage, QueryResponse, RequestId,
};
use turboclaude_transport::CliTransport;

/// Waits for a response to a query request
///
/// Holds mutable state for receiving the response and notifies when it arrives.
#[derive(Debug, Clone)]
struct ResponseWaiter {
    response: Arc<Mutex<Option<QueryResponse>>>,
    notify: Arc<Notify>,
}

impl ResponseWaiter {
    /// Create a new response waiter
    fn new() -> Self {
        Self {
            response: Arc::new(Mutex::new(None)),
            notify: Arc::new(Notify::new()),
        }
    }

    /// Store a response and notify waiters
    #[allow(dead_code)]
    async fn store_response(&self, response: QueryResponse) {
        *self.response.lock().await = Some(response);
        self.notify.notify_one();
    }

    /// Wait for a response with timeout
    async fn wait_response(&self, timeout_duration: Duration) -> AgentResult<QueryResponse> {
        match timeout(timeout_duration, self.notify.notified()).await {
            Ok(_) => self
                .response
                .lock()
                .await
                .take()
                .ok_or(crate::error::AgentError::Protocol(
                    "Response lost after notification".into(),
                )),
            Err(_) => Err(crate::error::AgentError::Protocol(
                "Response timeout".into(),
            )),
        }
    }
}

/// Routes protocol messages between client and CLI
///
/// Manages:
/// - Request/response correlation via RequestId
/// - Hook event dispatching
/// - Permission request evaluation
/// - Background message loop
pub struct MessageRouter {
    transport: Arc<CliTransport>,
    _hooks: Arc<HookRegistry>,
    _permissions: Arc<PermissionEvaluator>,
    pending_requests: Arc<Mutex<HashMap<String, ResponseWaiter>>>,
    shutdown: Arc<AtomicBool>,
    message_loop_handle: JoinHandle<()>,
}

impl MessageRouter {
    /// Create and start a new message router
    pub async fn new(
        transport: Arc<CliTransport>,
        hooks: Arc<HookRegistry>,
        permissions: Arc<PermissionEvaluator>,
    ) -> AgentResult<Self> {
        let pending_requests = Arc::new(Mutex::new(HashMap::new()));
        let shutdown = Arc::new(AtomicBool::new(false));

        // Spawn background message loop
        let message_loop_handle = {
            let transport = Arc::clone(&transport);
            let hooks = Arc::clone(&hooks);
            let permissions = Arc::clone(&permissions);
            let pending_requests = Arc::clone(&pending_requests);
            let shutdown = Arc::clone(&shutdown);

            tokio::spawn(async move {
                Self::message_loop(transport, hooks, permissions, pending_requests, shutdown).await;
            })
        };

        Ok(Self {
            transport,
            _hooks: hooks,
            _permissions: permissions,
            pending_requests,
            shutdown,
            message_loop_handle,
        })
    }

    /// Send a query and wait for response
    ///
    /// # Arguments
    /// * `request_id` - Unique request ID
    /// * `query` - The query to send
    ///
    /// # Returns
    /// The response from the CLI, or error if timeout/failure
    pub async fn send_query(
        &self,
        request_id: RequestId,
        query: turboclaude_protocol::QueryRequest,
    ) -> AgentResult<QueryResponse> {
        let request_id_str = request_id.as_str().to_string();

        // Create waiter
        let waiter = ResponseWaiter::new();
        {
            let mut pending = self.pending_requests.lock().await;
            if pending.contains_key(&request_id_str) {
                return Err(crate::error::AgentError::Other(
                    "Request ID already in use".into(),
                ));
            }
            pending.insert(request_id_str.clone(), waiter.clone());
        }

        // Send query message
        let message = ProtocolMessage::Query(query);
        let json = message.to_json().map_err(|e| {
            crate::error::AgentError::Protocol(format!("Failed to serialize query: {}", e))
        })?;
        let json_value = serde_json::from_str(&json).map_err(|e| {
            crate::error::AgentError::Protocol(format!("Failed to parse JSON: {}", e))
        })?;

        self.transport.send_message(json_value).await.map_err(|e| {
            crate::error::AgentError::Transport(format!("Failed to send query: {}", e))
        })?;

        // Wait for response
        let response = waiter.wait_response(Duration::from_secs(300)).await?;

        // Clean up
        self.pending_requests.lock().await.remove(&request_id_str);

        Ok(response)
    }

    /// Background message loop that routes incoming messages
    ///
    /// Continuously receives messages from transport and routes them to:
    /// - Hook registry for hook_request messages
    /// - Permission evaluator for permission_check messages
    /// - Pending requests map for response messages
    async fn message_loop(
        transport: Arc<CliTransport>,
        hooks: Arc<HookRegistry>,
        permissions: Arc<PermissionEvaluator>,
        pending_requests: Arc<Mutex<HashMap<String, ResponseWaiter>>>,
        shutdown: Arc<AtomicBool>,
    ) {
        loop {
            if shutdown.load(Ordering::Relaxed) {
                break;
            }

            // Receive message
            match transport.recv_message().await {
                Ok(Some(json_value)) => {
                    // Try to parse as protocol message
                    match serde_json::to_string(&json_value) {
                        Ok(json_str) => {
                            match ProtocolMessage::from_json(&json_str) {
                                Ok(message) => {
                                    // Route message
                                    match message {
                                        ProtocolMessage::HookRequest(hook_req) => {
                                            if let Err(e) = Self::handle_hook_request(
                                                hook_req, &hooks, &transport,
                                            )
                                            .await
                                            {
                                                eprintln!("Error handling hook request: {}", e);
                                            }
                                        }
                                        ProtocolMessage::PermissionCheck(perm_req) => {
                                            if let Err(e) = Self::handle_permission_request(
                                                perm_req,
                                                &permissions,
                                                &transport,
                                            )
                                            .await
                                            {
                                                eprintln!(
                                                    "Error handling permission request: {}",
                                                    e
                                                );
                                            }
                                        }
                                        ProtocolMessage::Response(response) => {
                                            // For now, extract base request ID - in full impl would use response.request_id
                                            // This is a placeholder - responses need request_id field
                                            if let Err(e) =
                                                Self::handle_response(response, &pending_requests)
                                                    .await
                                            {
                                                eprintln!("Error handling response: {}", e);
                                            }
                                        }
                                        ProtocolMessage::Error(error) => {
                                            eprintln!(
                                                "Protocol error from CLI: {} - {}",
                                                error.code, error.message
                                            );
                                        }
                                        _ => {
                                            eprintln!("Unexpected message type in message loop");
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to parse protocol message: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to serialize JSON: {}", e);
                        }
                    }
                }
                Ok(None) => {
                    // Transport closed
                    eprintln!("Transport closed");
                    break;
                }
                Err(e) => {
                    eprintln!("Error receiving message: {}", e);
                    // Continue receiving despite error
                }
            }
        }
    }

    /// Handle incoming hook request
    async fn handle_hook_request(
        request: HookRequest,
        hooks: &Arc<HookRegistry>,
        transport: &Arc<CliTransport>,
    ) -> AgentResult<()> {
        // Dispatch to hook registry
        let response = hooks.dispatch(request.event_type.clone(), request).await?;

        // Send response back
        let message = ProtocolMessage::HookResponse(Box::new(response));
        let json = message.to_json().map_err(|e| {
            crate::error::AgentError::Protocol(format!("Failed to serialize hook response: {}", e))
        })?;
        let json_value = serde_json::from_str(&json).map_err(|e| {
            crate::error::AgentError::Protocol(format!("Failed to parse JSON: {}", e))
        })?;

        transport.send_message(json_value).await.map_err(|e| {
            crate::error::AgentError::Transport(format!("Failed to send hook response: {}", e))
        })?;

        Ok(())
    }

    /// Handle incoming permission check request
    async fn handle_permission_request(
        request: PermissionCheckRequest,
        permissions: &Arc<PermissionEvaluator>,
        transport: &Arc<CliTransport>,
    ) -> AgentResult<()> {
        // Evaluate permission
        let response = permissions.check(request).await?;

        // Send response back
        let message = ProtocolMessage::PermissionResponse(response);
        let json = message.to_json().map_err(|e| {
            crate::error::AgentError::Protocol(format!(
                "Failed to serialize permission response: {}",
                e
            ))
        })?;
        let json_value = serde_json::from_str(&json).map_err(|e| {
            crate::error::AgentError::Protocol(format!("Failed to parse JSON: {}", e))
        })?;

        transport.send_message(json_value).await.map_err(|e| {
            crate::error::AgentError::Transport(format!(
                "Failed to send permission response: {}",
                e
            ))
        })?;

        Ok(())
    }

    /// Handle incoming response - store in waiter
    async fn handle_response(
        _response: QueryResponse,
        _pending_requests: &Arc<Mutex<HashMap<String, ResponseWaiter>>>,
    ) -> AgentResult<()> {
        // In full implementation, would extract request_id from response
        // For now, this is a placeholder
        // We would match response.request_id against pending requests

        // This would be:
        // let pending = pending_requests.lock().await;
        // if let Some(waiter) = pending.get(response.request_id) {
        //     waiter.store_response(response).await;
        // }

        Ok(())
    }

    /// Shutdown the message router
    pub async fn shutdown(&mut self) -> AgentResult<()> {
        self.shutdown.store(true, Ordering::Relaxed);

        // Wait for message loop to finish (with timeout)
        match timeout(Duration::from_secs(5), &mut self.message_loop_handle).await {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(crate::error::AgentError::Other(format!(
                "Message loop join error: {}",
                e
            ))),
            Err(_) => {
                // Timeout - message loop didn't shut down cleanly
                eprintln!("Message loop shutdown timeout");
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use turboclaude_protocol::message::MessageRole;
    use turboclaude_protocol::types::{StopReason, Usage};

    #[test]
    fn test_response_waiter_creation() {
        let waiter = ResponseWaiter::new();
        let response = waiter.response.clone();
        let notify = waiter.notify.clone();

        assert!(response.blocking_lock().is_none());
        // notify exists
        drop(notify);
    }

    #[tokio::test]
    async fn test_response_waiter_store_and_wait() {
        let waiter = ResponseWaiter::new();
        let waiter_clone = waiter.clone();

        // Spawn a task that stores a response
        let store_task = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let response = QueryResponse {
                message: turboclaude_protocol::Message {
                    id: "msg_test".to_string(),
                    message_type: "message".to_string(),
                    role: MessageRole::Assistant,
                    content: vec![turboclaude_protocol::ContentBlock::Text {
                        text: "test response".to_string(),
                    }],
                    model: "claude-3-5-sonnet".to_string(),
                    stop_reason: StopReason::EndTurn,
                    stop_sequence: None,
                    created_at: "2024-01-01T00:00:00Z".to_string(),
                    usage: Usage {
                        input_tokens: 10,
                        output_tokens: 5,
                    },
                    cache_usage: Default::default(),
                },
                is_complete: true,
            };
            waiter_clone.store_response(response).await;
        });

        // Wait for response with timeout
        let result = waiter.wait_response(Duration::from_secs(1)).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_complete);

        store_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_response_waiter_timeout() {
        let waiter = ResponseWaiter::new();

        // Wait for response that never comes
        let result = waiter.wait_response(Duration::from_millis(50)).await;
        assert!(result.is_err());
        match result {
            Err(crate::error::AgentError::Protocol(msg)) => {
                assert!(msg.contains("timeout"));
            }
            _ => panic!("Expected Protocol error with timeout message"),
        }
    }

    #[tokio::test]
    async fn test_message_router_creation() {
        // Note: Can't easily test without real transport
        // This would require mock transport implementation
    }

    #[test]
    fn test_request_id_matching() {
        let id = RequestId::new();
        let base = id.base();

        assert!(!base.is_empty());
        let sequenced = id.with_sequence(1);

        // Sequenced should have a dot
        assert!(sequenced.as_str().contains('.'));
    }

    #[test]
    fn test_request_id_base_extraction() {
        let id = RequestId::new();
        let base_str = id.base();

        let seq1 = id.with_sequence(1);
        let base_from_seq = seq1.base();

        assert_eq!(base_str, base_from_seq);
    }

    #[test]
    fn test_request_id_sequence_multiple() {
        let id = RequestId::new();

        let seq1 = id.with_sequence(1);
        let seq2 = id.with_sequence(2);
        let seq3 = id.with_sequence(3);

        assert!(seq1.as_str().contains(".1"));
        assert!(seq2.as_str().contains(".2"));
        assert!(seq3.as_str().contains(".3"));

        assert_eq!(seq1.base(), seq2.base());
        assert_eq!(seq2.base(), seq3.base());
    }

    #[test]
    fn test_response_waiter_clone() {
        let waiter1 = ResponseWaiter::new();
        let waiter2 = waiter1.clone();

        // Both should share the same Arc
        let response1 = waiter1.response.clone();
        let response2 = waiter2.response.clone();

        assert_eq!(
            Arc::as_ptr(&response1),
            Arc::as_ptr(&response2),
            "Cloned waiters should share the same response Arc"
        );
    }

    #[tokio::test]
    async fn test_concurrent_response_waiters() {
        let mut tasks = vec![];

        // Create multiple waiters
        for i in 0..5 {
            let waiter = ResponseWaiter::new();
            let waiter_clone = waiter.clone();

            let task = tokio::spawn(async move {
                let response = QueryResponse {
                    message: turboclaude_protocol::Message {
                        id: format!("msg_{}", i),
                        message_type: "message".to_string(),
                        role: MessageRole::Assistant,
                        content: vec![turboclaude_protocol::ContentBlock::Text {
                            text: format!("response {}", i),
                        }],
                        model: "claude-3-5-sonnet".to_string(),
                        stop_reason: StopReason::EndTurn,
                        stop_sequence: None,
                        created_at: "2024-01-01T00:00:00Z".to_string(),
                        usage: Usage {
                            input_tokens: 10,
                            output_tokens: 5,
                        },
                        cache_usage: Default::default(),
                    },
                    is_complete: true,
                };

                // Store after a small delay
                tokio::time::sleep(Duration::from_millis(10 * i as u64)).await;
                waiter.store_response(response).await;

                // Wait for it
                waiter_clone
                    .wait_response(Duration::from_secs(1))
                    .await
                    .ok()
            });

            tasks.push(task);
        }

        // Wait for all to complete
        for task in tasks {
            let result = task.await;
            assert!(result.is_ok());
        }
    }
}
