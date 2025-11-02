//! Hook system for agent event handling
//!
//! Provides a registry for registering and dispatching hooks that fire during query execution.
//! Hooks can observe events (PreToolUse, PostToolUse, etc.) and optionally modify inputs.

use crate::error::Result as AgentResult;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;
use turboclaude_protocol::{HookRequest, HookResponse};

/// Type alias for async hook handlers
///
/// Handlers take a HookRequest and return a Future with a HookResponse.
pub type HookHandler = (
    String,
    Arc<
        dyn Fn(HookRequest) -> Pin<Box<dyn Future<Output = AgentResult<HookResponse>> + Send>>
            + Send
            + Sync,
    >,
);

/// Handle for a registered hook (allows deregistration)
#[derive(Debug, Clone)]
pub struct HookHandle {
    id: String,
    event_type: String,
}

/// Registry for hook handlers
///
/// Stores handlers for different hook event types and provides dispatch functionality.
/// Handlers are called sequentially, and responses are merged with AND logic (all must continue).
pub struct HookRegistry {
    /// Map of event type to list of handlers
    handlers: Arc<Mutex<HashMap<String, Vec<HookHandler>>>>,
}

impl HookRegistry {
    /// Create a new hook registry
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a handler for a specific hook event type
    ///
    /// Returns a handle that can be used to deregister the handler later.
    pub async fn register<F>(&self, event_type: impl Into<String>, handler: F) -> HookHandle
    where
        F: Fn(HookRequest) -> Pin<Box<dyn Future<Output = AgentResult<HookResponse>> + Send>>
            + Send
            + Sync
            + 'static,
    {
        let event_type = event_type.into();
        let handler = Arc::new(handler);
        let id = format!("{}-{}", event_type, uuid::Uuid::new_v4());

        let mut handlers = self.handlers.lock().await;
        handlers
            .entry(event_type.clone())
            .or_insert_with(Vec::new)
            .push((id.clone(), handler));

        HookHandle { id, event_type }
    }

    /// Dispatch a hook event to all registered handlers
    ///
    /// Calls handlers sequentially and merges responses:
    /// - ALL handlers must return continue=true for overall continue=true
    /// - Modified inputs from handlers are merged (later overrides earlier)
    /// - Contexts are accumulated
    pub async fn dispatch(
        &self,
        event_type: impl Into<String>,
        request: HookRequest,
    ) -> AgentResult<HookResponse> {
        let event_type = event_type.into();
        let handlers = self.handlers.lock().await;

        // Get handlers for this event type
        let event_handlers = match handlers.get(&event_type) {
            Some(h) => h.clone(),
            None => {
                // No handlers registered, return continue=true
                return Ok(HookResponse::continue_exec());
            }
        };

        // Call each handler and collect responses
        let mut responses = Vec::new();
        for (_id, handler) in event_handlers {
            let response = handler(request.clone()).await?;
            responses.push(response);
        }

        // Merge responses
        Ok(merge_hook_responses(responses))
    }

    /// Deregister a hook.
    pub async fn deregister(&self, handle: HookHandle) {
        let mut handlers = self.handlers.lock().await;
        if let Some(event_handlers) = handlers.get_mut(&handle.event_type) {
            event_handlers.retain(|(id, _)| id != &handle.id);
        }
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Merge multiple hook responses into a single response
///
/// Semantics:
/// - continue_: AND logic (all must say true for result to be true)
/// - modified_inputs: Later inputs override earlier ones
/// - context: Accumulated from all hooks
/// - permission_decision: First non-None value wins (deny > ask > allow)
/// - Additional fields: Later values override earlier ones
fn merge_hook_responses(responses: Vec<HookResponse>) -> HookResponse {
    if responses.is_empty() {
        return HookResponse {
            continue_: true,
            modified_inputs: None,
            context: None,
            permission_decision: None,
            permission_decision_reason: None,
            additional_context: None,
            continue_reason: None,
            stop_reason: None,
            system_message: None,
            reason: None,
            suppress_output: None,
        };
    }

    // ALL must say continue for result to be true
    let continue_ = responses.iter().all(|r| r.continue_);

    // Merge modified inputs (later overrides earlier)
    let modified_inputs = responses
        .iter()
        .filter_map(|r| r.modified_inputs.clone())
        .next_back();

    // Accumulate contexts
    let mut context = None;
    for response in &responses {
        if let Some(ctx) = &response.context {
            match context {
                None => context = Some(ctx.clone()),
                Some(ref mut existing) => {
                    // Merge into existing context
                    if let (serde_json::Value::Object(obj), serde_json::Value::Object(new)) =
                        (existing, ctx)
                    {
                        for (k, v) in new {
                            obj.insert(k.clone(), v.clone());
                        }
                    }
                }
            }
        }
    }

    // Permission decision: deny > ask > allow (most restrictive wins)
    let permission_decision = responses
        .iter()
        .filter_map(|r| r.permission_decision.as_ref())
        .find(|d| d.as_str() == "deny")
        .or_else(|| {
            responses
                .iter()
                .filter_map(|r| r.permission_decision.as_ref())
                .find(|d| d.as_str() == "ask")
        })
        .or_else(|| {
            responses
                .iter()
                .filter_map(|r| r.permission_decision.as_ref())
                .next()
        })
        .cloned();

    // Other fields: later overrides earlier
    let permission_decision_reason = responses
        .iter()
        .filter_map(|r| r.permission_decision_reason.clone())
        .next_back();

    let additional_context = responses
        .iter()
        .filter_map(|r| r.additional_context.clone())
        .next_back();

    let continue_reason = responses
        .iter()
        .filter_map(|r| r.continue_reason.clone())
        .next_back();

    let stop_reason = responses
        .iter()
        .filter_map(|r| r.stop_reason.clone())
        .next_back();

    let system_message = responses
        .iter()
        .filter_map(|r| r.system_message.clone())
        .next_back();

    let reason = responses
        .iter()
        .filter_map(|r| r.reason.clone())
        .next_back();

    let suppress_output = responses
        .iter()
        .filter_map(|r| r.suppress_output)
        .next_back();

    HookResponse {
        continue_,
        modified_inputs,
        context,
        permission_decision,
        permission_decision_reason,
        additional_context,
        continue_reason,
        stop_reason,
        system_message,
        reason,
        suppress_output,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hook_registry_creation() {
        let registry = HookRegistry::new();
        assert_eq!(registry.handlers.lock().await.len(), 0);
    }

    #[tokio::test]
    async fn test_hook_register() {
        let registry = HookRegistry::new();

        registry
            .register("PreToolUse", |_req| {
                Box::pin(async { Ok(HookResponse::continue_exec()) })
            })
            .await;

        assert_eq!(registry.handlers.lock().await.len(), 1);
    }

    #[tokio::test]
    async fn test_hook_dispatch_no_handlers() {
        let registry = HookRegistry::new();
        let request = HookRequest {
            event_type: "PreToolUse".to_string(),
            data: serde_json::json!({}),
        };

        let response = registry.dispatch("PreToolUse", request).await.unwrap();
        assert!(response.continue_);
        assert!(response.modified_inputs.is_none());
        assert!(response.permission_decision.is_none());
    }

    #[tokio::test]
    async fn test_hook_dispatch_single_handler() {
        let registry = HookRegistry::new();

        registry
            .register("PreToolUse", |_req| {
                Box::pin(async { Ok(HookResponse::continue_exec()) })
            })
            .await;

        let request = HookRequest {
            event_type: "PreToolUse".to_string(),
            data: serde_json::json!({}),
        };

        let response = registry.dispatch("PreToolUse", request).await.unwrap();
        assert!(response.continue_);
    }

    #[tokio::test]
    async fn test_hook_dispatch_multiple_handlers_all_continue() {
        let registry = HookRegistry::new();

        registry
            .register("PreToolUse", |_req| {
                Box::pin(async { Ok(HookResponse::continue_exec()) })
            })
            .await;

        registry
            .register("PreToolUse", |_req| {
                Box::pin(async { Ok(HookResponse::continue_exec()) })
            })
            .await;

        let request = HookRequest {
            event_type: "PreToolUse".to_string(),
            data: serde_json::json!({}),
        };

        let response = registry.dispatch("PreToolUse", request).await.unwrap();
        assert!(response.continue_);
    }

    #[tokio::test]
    async fn test_hook_dispatch_one_handler_stops() {
        let registry = HookRegistry::new();

        registry
            .register("PreToolUse", |_req| {
                Box::pin(async { Ok(HookResponse::continue_exec()) })
            })
            .await;

        registry
            .register("PreToolUse", |_req| {
                Box::pin(async { Ok(HookResponse::stop()) })
            })
            .await;

        let request = HookRequest {
            event_type: "PreToolUse".to_string(),
            data: serde_json::json!({}),
        };

        let response = registry.dispatch("PreToolUse", request).await.unwrap();
        assert!(!response.continue_); // One handler said stop
    }

    #[tokio::test]
    async fn test_hook_response_merge_continue() {
        let responses = vec![HookResponse::continue_exec(), HookResponse::continue_exec()];

        let merged = merge_hook_responses(responses);
        assert!(merged.continue_);
    }

    #[tokio::test]
    async fn test_hook_response_merge_stop() {
        let responses = vec![HookResponse::continue_exec(), HookResponse::stop()];

        let merged = merge_hook_responses(responses);
        assert!(!merged.continue_);
    }

    #[tokio::test]
    async fn test_hook_response_merge_inputs() {
        let mut resp1 = HookResponse::continue_exec();
        resp1.modified_inputs = Some(turboclaude_protocol::ModifiedInputs {
            tool_name: Some("tool1".to_string()),
            input: None,
        });

        let mut resp2 = HookResponse::continue_exec();
        resp2.modified_inputs = Some(turboclaude_protocol::ModifiedInputs {
            tool_name: Some("tool2".to_string()),
            input: None,
        });

        let responses = vec![resp1, resp2];

        let merged = merge_hook_responses(responses);
        assert!(merged.continue_);
        assert_eq!(
            merged.modified_inputs.unwrap().tool_name,
            Some("tool2".to_string())
        ); // Later overrides
    }

    #[tokio::test]
    async fn test_hook_response_merge_permission_decision() {
        // Test that "deny" wins over "allow"
        let resp1 = HookResponse::continue_exec().with_permission_decision("allow");
        let resp2 = HookResponse::continue_exec().with_permission_decision("deny");

        let merged = merge_hook_responses(vec![resp1, resp2]);
        assert_eq!(merged.permission_decision, Some("deny".to_string()));

        // Test that "ask" wins over "allow"
        let resp1 = HookResponse::continue_exec().with_permission_decision("allow");
        let resp2 = HookResponse::continue_exec().with_permission_decision("ask");

        let merged = merge_hook_responses(vec![resp1, resp2]);
        assert_eq!(merged.permission_decision, Some("ask".to_string()));

        // Test that "deny" wins over "ask"
        let resp1 = HookResponse::continue_exec().with_permission_decision("ask");
        let resp2 = HookResponse::continue_exec().with_permission_decision("deny");

        let merged = merge_hook_responses(vec![resp1, resp2]);
        assert_eq!(merged.permission_decision, Some("deny".to_string()));
    }

    #[tokio::test]
    async fn test_hook_response_merge_additional_fields() {
        let resp1 = HookResponse::continue_exec()
            .with_system_message("First message")
            .with_reason("First reason");

        let resp2 = HookResponse::continue_exec()
            .with_system_message("Second message")
            .with_additional_context(serde_json::json!({"key": "value"}));

        let merged = merge_hook_responses(vec![resp1, resp2]);
        assert_eq!(merged.system_message, Some("Second message".to_string()));
        // reason: Last non-None value wins (resp2 had no reason, so resp1's remains)
        assert_eq!(merged.reason, Some("First reason".to_string()));
        assert!(merged.additional_context.is_some());
    }
}
