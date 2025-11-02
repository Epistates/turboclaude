//! Permission evaluation for tool execution
//!
//! Provides a permission system that controls whether tools are allowed to execute.
//! Supports fail-safe defaults (DENY), auto-approval modes, and audit trails.

use crate::error::Result as AgentResult;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, timeout};
use turboclaude_protocol::{
    PermissionBehavior, PermissionCheckRequest, PermissionMode, PermissionResponse,
    PermissionUpdate,
};

/// Type alias for async permission handlers
///
/// Handlers take a PermissionCheckRequest and return a Future with a PermissionResponse.
pub type PermissionHandler = Arc<
    dyn Fn(
            PermissionCheckRequest,
        ) -> Pin<Box<dyn Future<Output = AgentResult<PermissionResponse>> + Send>>
        + Send
        + Sync,
>;

/// Handle for a registered permission handler
#[derive(Debug, Clone)]
pub struct PermissionHandle {
    _id: String,
}

/// Permission state tracking rules and directories
#[derive(Debug, Clone, Default)]
struct PermissionState {
    /// Allow rules: tool name -> rule content
    allow_rules: HashMap<String, Option<String>>,

    /// Deny rules: tool name -> rule content
    deny_rules: HashMap<String, Option<String>>,

    /// Ask rules: tool name -> rule content
    ask_rules: HashMap<String, Option<String>>,

    /// Allowed directories
    allowed_directories: Vec<String>,
}

/// Permission evaluator
///
/// Evaluates permission requests with configurable behavior based on the permission mode.
/// Provides fail-safe defaults: denies permission unless explicitly approved.
pub struct PermissionEvaluator {
    /// Optional permission handler
    handler: Arc<Mutex<Option<PermissionHandler>>>,

    /// Current permission mode
    mode: Arc<Mutex<PermissionMode>>,

    /// Permission state (rules and directories)
    state: Arc<Mutex<PermissionState>>,
}

impl PermissionEvaluator {
    /// Create a new permission evaluator with the given default mode
    pub fn new(mode: PermissionMode) -> Self {
        Self {
            handler: Arc::new(Mutex::new(None)),
            mode: Arc::new(Mutex::new(mode)),
            state: Arc::new(Mutex::new(PermissionState::default())),
        }
    }

    /// Register a permission handler
    ///
    /// The handler is called when a tool needs permission.
    pub async fn register<F>(&self, handler: F) -> PermissionHandle
    where
        F: Fn(
                PermissionCheckRequest,
            ) -> Pin<Box<dyn Future<Output = AgentResult<PermissionResponse>> + Send>>
            + Send
            + Sync
            + 'static,
    {
        let handler = Arc::new(handler);
        let id = format!("permission-{}", uuid::Uuid::new_v4());

        *self.handler.lock().await = Some(handler);

        PermissionHandle { _id: id }
    }

    /// Check if a tool use is permitted
    ///
    /// Returns the permission response. Semantics depend on the permission mode:
    ///
    /// - `Default`: Requires handler to approve (fails closed - denies by default)
    /// - `AcceptEdits`: Auto-approves but allows handler to modify inputs
    /// - `BypassPermissions`: Always approves without consulting handler
    pub async fn check(&self, request: PermissionCheckRequest) -> AgentResult<PermissionResponse> {
        let mode = *self.mode.lock().await;

        match mode {
            PermissionMode::BypassPermissions => {
                // Auto-approve all
                Ok(PermissionResponse {
                    allow: true,
                    modified_input: None,
                    reason: Some("Permissions bypassed".to_string()),
                })
            }
            PermissionMode::AcceptEdits => {
                // Auto-approve but allow handler to modify inputs
                let handler = self.handler.lock().await;
                if let Some(handler) = handler.as_ref() {
                    let response = match timeout(Duration::from_secs(30), handler(request)).await {
                        Ok(Ok(resp)) => resp,
                        Ok(Err(e)) => return Err(e),
                        Err(_) => {
                            // Timeout - still auto-approve but log it
                            PermissionResponse {
                                allow: true,
                                modified_input: None,
                                reason: Some(
                                    "Accepted after handler timeout (AcceptEdits mode)".to_string(),
                                ),
                            }
                        }
                    };
                    return Ok(response);
                }

                // No handler registered, auto-approve with warning
                Ok(PermissionResponse {
                    allow: true,
                    modified_input: None,
                    reason: Some(
                        "Accepted without explicit approval (AcceptEdits mode)".to_string(),
                    ),
                })
            }
            PermissionMode::Default => {
                // Require explicit permission
                let handler = self.handler.lock().await;
                if let Some(handler) = handler.as_ref() {
                    let response = timeout(Duration::from_secs(30), handler(request)).await;

                    match response {
                        Ok(Ok(resp)) => return Ok(resp),
                        Ok(Err(e)) => return Err(e),
                        Err(_) => {
                            // Timeout - fail-safe DENY
                            return Ok(PermissionResponse {
                                allow: false,
                                modified_input: None,
                                reason: Some(
                                    "Permission check timeout (fail-safe deny)".to_string(),
                                ),
                            });
                        }
                    }
                }

                // No handler registered - fail-safe DENY
                Ok(PermissionResponse {
                    allow: false,
                    modified_input: None,
                    reason: Some("No permission handler registered (fail-safe deny)".to_string()),
                })
            }
        }
    }

    /// Change the permission mode
    pub async fn set_mode(&self, mode: PermissionMode) {
        *self.mode.lock().await = mode;
    }

    /// Get the current permission mode
    pub async fn get_mode(&self) -> PermissionMode {
        *self.mode.lock().await
    }

    /// Update permissions dynamically
    ///
    /// Applies a permission update to the evaluator state. Updates are atomic
    /// and thread-safe. Validates the update before applying.
    ///
    /// # Arguments
    ///
    /// * `update` - The permission update to apply
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the update was applied successfully, or an error
    /// if the update was invalid or could not be applied.
    ///
    /// # Thread Safety
    ///
    /// This method is thread-safe and can be called concurrently from multiple
    /// threads. Updates are applied atomically.
    pub async fn update_permissions(&self, update: PermissionUpdate) -> AgentResult<()> {
        // Validate the update first
        update.validate().map_err(|e| {
            crate::error::AgentError::Config(format!("Invalid permission update: {}", e))
        })?;

        let mut state = self.state.lock().await;

        match update {
            PermissionUpdate::AddRules(add_rules) => {
                // Add rules to the appropriate behavior map
                let target_map = match add_rules.behavior {
                    PermissionBehavior::Allow => &mut state.allow_rules,
                    PermissionBehavior::Deny => &mut state.deny_rules,
                    PermissionBehavior::Ask => &mut state.ask_rules,
                };

                for rule in add_rules.rules {
                    target_map.insert(rule.tool_name, rule.rule_content);
                }
            }
            PermissionUpdate::ReplaceRules(replace_rules) => {
                // Clear existing rules for this behavior and add new ones
                let target_map = match replace_rules.behavior {
                    PermissionBehavior::Allow => &mut state.allow_rules,
                    PermissionBehavior::Deny => &mut state.deny_rules,
                    PermissionBehavior::Ask => &mut state.ask_rules,
                };

                target_map.clear();
                for rule in replace_rules.rules {
                    target_map.insert(rule.tool_name, rule.rule_content);
                }
            }
            PermissionUpdate::RemoveRules(remove_rules) => {
                // Remove rules from all behavior maps
                for rule in remove_rules.rules {
                    state.allow_rules.remove(&rule.tool_name);
                    state.deny_rules.remove(&rule.tool_name);
                    state.ask_rules.remove(&rule.tool_name);
                }
            }
            PermissionUpdate::SetMode(set_mode) => {
                // Update the permission mode
                *self.mode.lock().await = set_mode.mode;
            }
            PermissionUpdate::AddDirectories(add_dirs) => {
                // Add directories to the allowed list
                for dir in add_dirs.directories {
                    if !state.allowed_directories.contains(&dir) {
                        state.allowed_directories.push(dir);
                    }
                }
            }
            PermissionUpdate::RemoveDirectories(remove_dirs) => {
                // Remove directories from the allowed list
                state
                    .allowed_directories
                    .retain(|d| !remove_dirs.directories.contains(d));
            }
        }

        Ok(())
    }

    /// Get current permission state (for debugging/inspection)
    pub async fn get_state(&self) -> (PermissionMode, Vec<String>) {
        let mode = *self.mode.lock().await;
        let state = self.state.lock().await;
        (mode, state.allowed_directories.clone())
    }
}

impl Default for PermissionEvaluator {
    fn default() -> Self {
        Self::new(PermissionMode::Default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_permission_evaluator_creation() {
        let evaluator = PermissionEvaluator::new(PermissionMode::Default);
        assert_eq!(evaluator.get_mode().await, PermissionMode::Default);
    }

    #[tokio::test]
    async fn test_permission_default_deny_no_handler() {
        let evaluator = PermissionEvaluator::new(PermissionMode::Default);
        let request = PermissionCheckRequest {
            tool: "web_search".to_string(),
            input: serde_json::json!({}),
            suggestion: "Use web_search?".to_string(),
        };

        let response = evaluator.check(request).await.unwrap();
        assert!(!response.allow); // Fail-safe DENY
        assert!(response.reason.is_some());
    }

    #[tokio::test]
    async fn test_permission_bypass_mode() {
        let evaluator = PermissionEvaluator::new(PermissionMode::BypassPermissions);
        let request = PermissionCheckRequest {
            tool: "web_search".to_string(),
            input: serde_json::json!({}),
            suggestion: "Use web_search?".to_string(),
        };

        let response = evaluator.check(request).await.unwrap();
        assert!(response.allow); // Auto-approve
    }

    #[tokio::test]
    async fn test_permission_accept_edits_auto_approve() {
        let evaluator = PermissionEvaluator::new(PermissionMode::AcceptEdits);
        let request = PermissionCheckRequest {
            tool: "web_search".to_string(),
            input: serde_json::json!({}),
            suggestion: "Use web_search?".to_string(),
        };

        let response = evaluator.check(request).await.unwrap();
        assert!(response.allow); // Auto-approve
    }

    #[tokio::test]
    async fn test_permission_handler_approve() {
        let evaluator = PermissionEvaluator::new(PermissionMode::Default);

        evaluator
            .register(|_req| {
                Box::pin(async {
                    Ok(PermissionResponse {
                        allow: true,
                        modified_input: None,
                        reason: Some("User approved".to_string()),
                    })
                })
            })
            .await;

        let request = PermissionCheckRequest {
            tool: "web_search".to_string(),
            input: serde_json::json!({}),
            suggestion: "Use web_search?".to_string(),
        };

        let response = evaluator.check(request).await.unwrap();
        assert!(response.allow);
    }

    #[tokio::test]
    async fn test_permission_handler_deny() {
        let evaluator = PermissionEvaluator::new(PermissionMode::Default);

        evaluator
            .register(|_req| {
                Box::pin(async {
                    Ok(PermissionResponse {
                        allow: false,
                        modified_input: None,
                        reason: Some("User denied".to_string()),
                    })
                })
            })
            .await;

        let request = PermissionCheckRequest {
            tool: "web_search".to_string(),
            input: serde_json::json!({}),
            suggestion: "Use web_search?".to_string(),
        };

        let response = evaluator.check(request).await.unwrap();
        assert!(!response.allow);
    }

    #[tokio::test]
    async fn test_permission_mode_change() {
        let evaluator = PermissionEvaluator::new(PermissionMode::Default);
        assert_eq!(evaluator.get_mode().await, PermissionMode::Default);

        evaluator.set_mode(PermissionMode::BypassPermissions).await;
        assert_eq!(
            evaluator.get_mode().await,
            PermissionMode::BypassPermissions
        );
    }
}
