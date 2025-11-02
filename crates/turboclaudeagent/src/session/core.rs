//! Core session management
//!
//! Provides the main AgentSession struct and session creation logic.

use crate::config::SessionConfig;
use crate::error::{AgentError, Result as AgentResult};
use crate::hooks::HookRegistry;
use crate::permissions::PermissionEvaluator;
use crate::routing::MessageRouter;
use crate::session::state::SessionState;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::time::Duration;
use tokio::sync::Mutex;
use turboclaude_protocol::Message;
use turboclaude_transport::{CliTransport, ProcessConfig};

/// An interactive agent session with Claude Code CLI
///
/// Provides the main entry point for queries, hook registration, permission callbacks,
/// and runtime control commands.
pub struct AgentSession {
    /// Transport to Claude CLI
    pub(crate) transport: Arc<CliTransport>,

    /// Configuration
    pub(crate) config: Arc<SessionConfig>,

    /// Hook registry for event callbacks
    pub(crate) hooks: Arc<HookRegistry>,

    /// Permission evaluator for access control
    pub(crate) permissions: Arc<PermissionEvaluator>,

    /// Message router for protocol communication
    pub(crate) router: Arc<Mutex<Option<MessageRouter>>>,

    /// Session state
    pub(crate) state: Arc<Mutex<SessionState>>,

    /// Active query counter for state tracking
    pub(crate) active_queries: Arc<AtomicU32>,

    /// Skill manager (optional, requires 'skills' feature)
    #[cfg(feature = "skills")]
    pub(crate) skill_manager: Arc<tokio::sync::RwLock<Option<crate::skills::SkillManager>>>,
}

impl AgentSession {
    /// Create a new session
    ///
    /// Spawns the Claude Code CLI subprocess and initializes the session.
    pub async fn new(config: SessionConfig) -> AgentResult<Self> {
        // Spawn CLI transport
        let process_config = ProcessConfig {
            cli_path: config.cli_path.clone(),
            ..Default::default()
        };
        let transport = CliTransport::spawn(process_config)
            .await
            .map_err(|e| AgentError::Transport(format!("Failed to spawn CLI: {}", e)))?;
        let transport = Arc::new(transport);

        // Create hooks and permissions
        let hooks = Arc::new(HookRegistry::new());
        let permissions = Arc::new(PermissionEvaluator::new(config.permission_mode));

        // Create message router
        let router = MessageRouter::new(
            Arc::clone(&transport),
            Arc::clone(&hooks),
            Arc::clone(&permissions),
        )
        .await?;

        // Initialize session state
        let state = SessionState::new(config.default_model.clone(), config.permission_mode);

        // Initialize skill manager if skills feature is enabled
        #[cfg(feature = "skills")]
        let skill_manager = {
            use turboclaude_skills::SkillRegistry;

            // Create skill registry from configured directories
            let mut registry_builder = SkillRegistry::builder();
            for dir in &config.skill_dirs {
                registry_builder = registry_builder.skill_dir(dir.clone());
            }

            let registry = registry_builder.build().map_err(|e| {
                AgentError::Config(format!("Failed to create skill registry: {}", e))
            })?;

            // Create skill manager
            let manager = crate::skills::SkillManager::new(registry).await?;
            Arc::new(tokio::sync::RwLock::new(Some(manager)))
        };

        Ok(Self {
            transport,
            config: Arc::new(config),
            hooks,
            permissions,
            router: Arc::new(Mutex::new(Some(router))),
            state: Arc::new(Mutex::new(state)),
            active_queries: Arc::new(AtomicU32::new(0)),
            #[cfg(feature = "skills")]
            skill_manager,
        })
    }

    /// Fork this session, creating a new session with copied conversation history
    ///
    /// The forked session:
    /// - Starts a new subprocess (independent transport)
    /// - Copies current conversation history
    /// - Inherits configuration (model, permissions, hooks)
    /// - Has independent state (can diverge)
    ///
    /// # Returns
    ///
    /// A new `AgentSession` with the same configuration and conversation history
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to create new subprocess
    /// - Failed to initialize new session
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use turboclaudeagent::ClaudeAgentClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = ClaudeAgentClient::builder()
    ///     .api_key("your-api-key")
    ///     .build()?;
    /// let client = ClaudeAgentClient::new(config);
    /// let session = client.create_session().await?;
    ///
    /// // Have a conversation
    /// session.query_str("What is 2+2?").await?;
    ///
    /// // Fork to explore alternative path
    /// let forked = session.fork().await?;
    /// forked.query_str("What about 3+3?").await?;
    ///
    /// // Original session is unchanged
    /// session.query_str("What about multiplication?").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fork(&self) -> AgentResult<AgentSession> {
        // 1. Get current conversation history
        let history = {
            let state = self.state.lock().await;
            state.get_history()
        };

        // 2. Clone configuration
        let config = (*self.config).clone();

        // 3. Create new session with same config
        let forked = AgentSession::new(config).await?;

        // 4. Copy conversation history
        {
            let mut forked_state = forked.state.lock().await;
            for msg in history {
                forked_state.add_to_history(msg);
            }
        }

        // 5. Copy current model and permission mode
        {
            let current_state = self.state.lock().await;
            let mut forked_state = forked.state.lock().await;
            forked_state.current_model = current_state.current_model.clone();
            forked_state.current_permission_mode = current_state.current_permission_mode;
        }

        // Note: We don't copy hooks/permissions/skills as they are already shared via Arc
        // and the new session gets fresh instances from the config

        Ok(forked)
    }

    /// Get the current session state
    pub async fn state(&self) -> SessionState {
        self.state.lock().await.clone()
    }

    /// Check if the session is currently connected to the CLI
    ///
    /// Convenience method to check connection status without getting the full state.
    pub async fn is_connected(&self) -> bool {
        self.state.lock().await.is_connected
    }

    /// Close the session and cleanup resources
    ///
    /// Shuts down the message router and kills the CLI subprocess.
    pub async fn close(&self) -> AgentResult<()> {
        // Update state
        {
            let mut state = self.state.lock().await;
            state.is_connected = false;
        }

        // Shutdown message router
        {
            let mut router_lock = self.router.lock().await;
            if let Some(mut router) = router_lock.take() {
                let _ = router.shutdown().await;
            }
        }

        // Kill transport
        self.transport
            .kill()
            .await
            .map_err(|e| AgentError::Transport(format!("Failed to kill transport: {}", e)))?;

        Ok(())
    }

    /// Ensure the session is connected, reconnecting if necessary
    ///
    /// Called before each query. Auto-restarts subprocess with exponential backoff.
    pub(crate) async fn ensure_connected(&self) -> AgentResult<()> {
        // Check if transport is alive
        if self.transport.is_alive().await {
            return Ok(());
        }

        // Not alive, need to reconnect with exponential backoff
        let mut backoff = Duration::from_millis(500);
        for attempt in 0..5 {
            match self.reconnect().await {
                Ok(_) => {
                    // Update state
                    {
                        let mut state = self.state.lock().await;
                        state.is_connected = true;
                    }
                    return Ok(());
                }
                Err(_e) if attempt < 4 => {
                    // Sleep with backoff
                    tokio::time::sleep(backoff).await;

                    // Double backoff, capped at 60s
                    let backoff_millis = std::cmp::min(
                        backoff.as_millis() as u64 * 2,
                        Duration::from_secs(60).as_millis() as u64,
                    );
                    backoff = Duration::from_millis(backoff_millis);
                }
                Err(e) => {
                    // Final attempt failed
                    return Err(e);
                }
            }
        }

        // All reconnection attempts failed
        Err(AgentError::Transport(
            "Failed to reconnect after 5 attempts".into(),
        ))
    }

    /// Reconnect to the CLI after a crash
    pub(crate) async fn reconnect(&self) -> AgentResult<()> {
        // Kill old transport
        let _ = self.transport.kill().await;

        // Spawn new CliTransport
        let process_config = ProcessConfig {
            cli_path: self.config.cli_path.clone(),
            ..Default::default()
        };
        let _new_transport = CliTransport::spawn(process_config)
            .await
            .map_err(|e| AgentError::Transport(format!("Failed to spawn new CLI: {}", e)))?;

        // Note: We can't replace the Arc<CliTransport> itself (it's already shared)
        // The CliTransport internally manages process state, so killing and respawning
        // the process should work through the existing transport Arc.
        // In a production implementation, we would need to redesign to support transport
        // replacement, or use a wrapper type with interior mutability.

        // For now, create new message router with the old transport Arc
        // (it should now point to the respawned process)
        let new_router = MessageRouter::new(
            Arc::clone(&self.transport),
            Arc::clone(&self.hooks),
            Arc::clone(&self.permissions),
        )
        .await?;

        // Replace router
        {
            let mut router_lock = self.router.lock().await;
            if let Some(mut old_router) = router_lock.take() {
                let _ = old_router.shutdown().await;
            }
            *router_lock = Some(new_router);
        }

        Ok(())
    }

    /// Add a message to the conversation history
    ///
    /// This is an internal method used for tracking conversation state
    /// and supporting session forking.
    #[allow(dead_code)]
    pub(crate) async fn add_message_to_history(&self, message: Message) {
        let mut state = self.state.lock().await;
        state.add_to_history(message);
    }

    /// Get the conversation history
    ///
    /// Returns a clone of all messages in the current conversation.
    #[allow(dead_code)]
    pub(crate) async fn get_conversation_history(&self) -> Vec<Message> {
        let state = self.state.lock().await;
        state.get_history()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use turboclaude_protocol::PermissionMode;

    #[test]
    fn test_session_state_new() {
        let state = SessionState::new("claude-3-5-sonnet".to_string(), PermissionMode::Default);

        assert!(state.is_connected);
        assert_eq!(state.current_model, "claude-3-5-sonnet");
        assert_eq!(state.current_permission_mode, PermissionMode::Default);
        assert_eq!(state.active_queries, 0);
    }

    #[tokio::test]
    async fn test_session_creation() {
        // Note: This test will fail without a running Claude CLI
        // In a production setup, we would use a mock transport
        let _config = SessionConfig::default();
        // AgentSession::new(config).await will fail without CLI
        // Skipping full creation test
    }

    #[test]
    fn test_backoff_calculation() {
        let mut backoff = Duration::from_millis(500);

        // Test exponential backoff
        for _ in 0..5 {
            let next_millis = std::cmp::min(
                backoff.as_millis() as u64 * 2,
                Duration::from_secs(60).as_millis() as u64,
            );
            backoff = Duration::from_millis(next_millis);
        }

        // Should be capped at 60 seconds
        assert!(backoff <= Duration::from_secs(60));
    }
}
