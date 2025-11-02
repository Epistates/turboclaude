//! Runtime control methods
//!
//! Provides methods for controlling the session at runtime including
//! interrupts, model changes, permission mode updates, and hook/permission registration.

use crate::error::{AgentError, Result as AgentResult};
use crate::session::core::AgentSession;
use std::sync::Arc;
use turboclaude_protocol::{ControlCommand, PermissionMode};

impl AgentSession {
    /// Register a hook callback for a specific event type
    ///
    /// Hooks are called during query execution to monitor or modify behavior.
    pub fn register_hook<F>(&self, event_type: String, handler: F)
    where
        F: Fn(
                turboclaude_protocol::HookRequest,
            ) -> std::pin::Pin<
                Box<
                    dyn std::future::Future<
                            Output = AgentResult<turboclaude_protocol::HookResponse>,
                        > + Send,
                >,
            > + Send
            + Sync
            + 'static,
    {
        // Store handler in HookRegistry (async operation wrapped in sync function)
        let hooks = Arc::clone(&self.hooks);
        let event_type_copy = event_type.clone();

        tokio::spawn(async move {
            hooks.register(event_type_copy, handler).await;
        });
    }

    /// Register a permission callback
    ///
    /// Called when Claude requests permission to use a tool.
    /// Must return a permission decision.
    pub fn register_permission_handler<F>(&self, handler: F)
    where
        F: Fn(
                turboclaude_protocol::PermissionCheckRequest,
            ) -> std::pin::Pin<
                Box<
                    dyn std::future::Future<
                            Output = AgentResult<turboclaude_protocol::PermissionResponse>,
                        > + Send,
                >,
            > + Send
            + Sync
            + 'static,
    {
        // Store handler in PermissionEvaluator
        let permissions = Arc::clone(&self.permissions);

        tokio::spawn(async move {
            permissions.register(handler).await;
        });
    }

    /// Interrupt the current query
    ///
    /// Sends a control request to stop the running query.
    pub async fn interrupt(&self) -> AgentResult<()> {
        // Create control request
        let control_request = turboclaude_protocol::protocol::ControlRequest {
            command: ControlCommand::Interrupt,
        };

        // Send via transport
        let message = turboclaude_protocol::ProtocolMessage::ControlRequest(control_request);
        let json = message.to_json().map_err(|e| {
            AgentError::Protocol(format!("Failed to serialize control request: {}", e))
        })?;
        let json_value = serde_json::from_str(&json)
            .map_err(|e| AgentError::Protocol(format!("Failed to parse JSON: {}", e)))?;

        self.transport
            .send_message(json_value)
            .await
            .map_err(|e| AgentError::Transport(format!("Failed to send interrupt: {}", e)))?;

        Ok(())
    }

    /// Change the model for future queries
    ///
    /// Updates both the local config and sends a control request to CLI.
    pub async fn set_model(&self, model: impl Into<String>) -> AgentResult<()> {
        let model_str = model.into();

        // Update state
        {
            let mut state = self.state.lock().await;
            state.current_model = model_str.clone();
        }

        // Create control request
        let control_request = turboclaude_protocol::protocol::ControlRequest {
            command: ControlCommand::SetModel(model_str),
        };

        // Send via transport
        let message = turboclaude_protocol::ProtocolMessage::ControlRequest(control_request);
        let json = message.to_json().map_err(|e| {
            AgentError::Protocol(format!("Failed to serialize control request: {}", e))
        })?;
        let json_value = serde_json::from_str(&json)
            .map_err(|e| AgentError::Protocol(format!("Failed to parse JSON: {}", e)))?;

        self.transport
            .send_message(json_value)
            .await
            .map_err(|e| AgentError::Transport(format!("Failed to send set_model: {}", e)))?;

        Ok(())
    }

    /// Change the permission mode for future queries
    ///
    /// Updates both the local config and permission evaluator.
    pub async fn set_permission_mode(&self, mode: PermissionMode) -> AgentResult<()> {
        // Update state and permissions
        {
            let mut state = self.state.lock().await;
            state.current_permission_mode = mode;
        }
        self.permissions.set_mode(mode).await;

        // Create control request with string representation
        let mode_str = format!("{:?}", mode).to_lowercase(); // Convert to string
        let control_request = turboclaude_protocol::protocol::ControlRequest {
            command: ControlCommand::SetPermissionMode(mode_str),
        };

        // Send via transport
        let message = turboclaude_protocol::ProtocolMessage::ControlRequest(control_request);
        let json = message.to_json().map_err(|e| {
            AgentError::Protocol(format!("Failed to serialize control request: {}", e))
        })?;
        let json_value = serde_json::from_str(&json)
            .map_err(|e| AgentError::Protocol(format!("Failed to parse JSON: {}", e)))?;

        self.transport.send_message(json_value).await.map_err(|e| {
            AgentError::Transport(format!("Failed to send set_permission_mode: {}", e))
        })?;

        Ok(())
    }

    /// Update permissions dynamically
    ///
    /// Applies a permission update to the session. Updates are validated before
    /// being applied and are atomic and thread-safe.
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
    /// # Examples
    ///
    /// ```no_run
    /// # use turboclaudeagent::ClaudeAgentClient;
    /// # use turboclaude_protocol::{PermissionUpdate, PermissionRuleValue, PermissionBehavior};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = ClaudeAgentClient::builder()
    ///     .api_key("your-api-key")
    ///     .build()?;
    /// let client = ClaudeAgentClient::new(config);
    /// let session = client.create_session().await?;
    ///
    /// // Add allow rule for bash tool
    /// let rules = vec![PermissionRuleValue::new("bash")];
    /// let update = PermissionUpdate::add_rules(rules, PermissionBehavior::Allow);
    /// session.update_permissions(update).await?;
    ///
    /// // Add allowed directory
    /// let update = PermissionUpdate::add_directories(vec!["/home/user/project".to_string()]);
    /// session.update_permissions(update).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_permissions(
        &self,
        update: turboclaude_protocol::PermissionUpdate,
    ) -> AgentResult<()> {
        self.permissions.update_permissions(update).await
    }
}

//
// ===== Skill Management Methods (requires 'skills' feature) =====
//

#[cfg(feature = "skills")]
impl AgentSession {
    /// Discover skills from configured directories
    ///
    /// Scans all skill directories for SKILL.md files and loads them into the registry.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Skills feature is not enabled
    /// - Discovery fails (filesystem issues, parse errors)
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
    /// let result = session.discover_skills().await?;
    /// println!("Discovered {} skills", result.loaded);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn discover_skills(&self) -> AgentResult<crate::skills::SkillDiscoveryResult> {
        let manager = self.skill_manager.read().await;
        match manager.as_ref() {
            Some(m) => m.discover().await,
            None => Err(AgentError::Config("Skills feature not enabled".into())),
        }
    }

    /// Load (activate) a skill by name
    ///
    /// Once loaded, the skill's context will be automatically injected into all queries
    /// and its allowed-tools constraints will be enforced.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Skills feature is not enabled
    /// - Skill not found in registry
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
    /// session.discover_skills().await?;
    /// session.load_skill("pdf").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_skill(&self, name: &str) -> AgentResult<()> {
        let manager = self.skill_manager.read().await;
        match manager.as_ref() {
            Some(m) => m.load(name).await,
            None => Err(AgentError::Config("Skills feature not enabled".into())),
        }
    }

    /// Unload (deactivate) a skill by name
    ///
    /// Removes the skill from the active set. Future queries will not include
    /// this skill's context or constraints.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Skills feature is not enabled
    /// - Skill is not currently active
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
    /// session.load_skill("pdf").await?;
    /// // ... do work ...
    /// session.unload_skill("pdf").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn unload_skill(&self, name: &str) -> AgentResult<()> {
        let manager = self.skill_manager.read().await;
        match manager.as_ref() {
            Some(m) => m.unload(name).await,
            None => Err(AgentError::Config("Skills feature not enabled".into())),
        }
    }

    /// List active skill names
    ///
    /// Returns the names of all currently loaded skills.
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
    /// session.load_skill("pdf").await?;
    /// let active = session.active_skills().await;
    /// println!("Active: {:?}", active);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn active_skills(&self) -> Vec<String> {
        let manager = self.skill_manager.read().await;
        match manager.as_ref() {
            Some(m) => m.list_active().await,
            None => Vec::new(),
        }
    }

    /// List all available skills in registry
    ///
    /// Returns the names of all skills that have been discovered and are available
    /// to load.
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
    /// session.discover_skills().await?;
    /// let available = session.available_skills().await;
    /// println!("Available: {:?}", available);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn available_skills(&self) -> Vec<String> {
        let manager = self.skill_manager.read().await;
        match manager.as_ref() {
            Some(m) => m.list_available().await,
            None => Vec::new(),
        }
    }

    /// Find skills matching a query
    ///
    /// Performs semantic search across all available skills.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Skills feature is not enabled
    /// - Search operation fails
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
    /// session.discover_skills().await?;
    /// let matches = session.find_skills("PDF processing").await?;
    /// println!("Found {} matching skills", matches.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_skills(&self, query: &str) -> AgentResult<Vec<turboclaude_skills::Skill>> {
        let manager = self.skill_manager.read().await;
        match manager.as_ref() {
            Some(m) => m.find(query).await,
            None => Err(AgentError::Config("Skills feature not enabled".into())),
        }
    }

    /// Get skill by name
    ///
    /// Retrieves the full skill object from the registry.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Skills feature is not enabled
    /// - Skill not found
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
    /// session.discover_skills().await?;
    /// let skill = session.get_skill("pdf").await?;
    /// println!("Skill: {}", skill.metadata.description);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_skill(&self, name: &str) -> AgentResult<turboclaude_skills::Skill> {
        let manager = self.skill_manager.read().await;
        match manager.as_ref() {
            Some(m) => m.get(name).await,
            None => Err(AgentError::Config("Skills feature not enabled".into())),
        }
    }

    /// Validate if a tool is allowed by active skills
    ///
    /// Checks the given tool name against the allowed-tools constraints of all
    /// active skills.
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
    /// session.load_skill("restricted-skill").await?;
    /// let result = session.validate_tool("bash").await;
    /// if !result.allowed {
    ///     println!("Tool blocked by: {:?}", result.blocked_by);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn validate_tool(&self, tool_name: &str) -> crate::skills::ToolValidationResult {
        let manager = self.skill_manager.read().await;
        match manager.as_ref() {
            Some(m) => m.validate_tool(tool_name).await,
            None => crate::skills::ToolValidationResult {
                allowed: true,
                tool_name: tool_name.to_string(),
                blocked_by: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_state_mutation() {
        use crate::session::state::SessionState;
        use turboclaude_protocol::PermissionMode;

        let state = Arc::new(Mutex::new(SessionState {
            is_connected: true,
            current_model: "model1".to_string(),
            current_permission_mode: PermissionMode::Default,
            active_queries: 0,
            conversation_history: Vec::new(),
        }));

        // Simulate mutation
        {
            let mut s = state.lock().await;
            s.is_connected = false;
            s.current_model = "model2".to_string();
        }

        // Verify
        let s = state.lock().await;
        assert!(!s.is_connected);
        assert_eq!(s.current_model, "model2");
    }
}
