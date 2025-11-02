//! Main client for the Agent SDK

use crate::config::{ClaudeAgentClientConfig, SessionConfig};
use crate::error::Result;
use crate::session::AgentSession;

/// Main client for interactive agent sessions
pub struct ClaudeAgentClient {
    _config: ClaudeAgentClientConfig,
}

impl ClaudeAgentClient {
    /// Create a builder
    pub fn builder() -> crate::config::ClaudeAgentClientBuilder {
        crate::config::ClaudeAgentClientBuilder::default()
    }

    /// Create from config
    pub fn new(config: ClaudeAgentClientConfig) -> Self {
        Self { _config: config }
    }

    /// Create a new session
    ///
    /// Creates a SessionConfig from the client config and spawns a new agent session.
    pub async fn create_session(&self) -> Result<AgentSession> {
        let mut session_config = SessionConfig::default();

        // Apply client config overrides
        if let Some(ref model) = self._config.model {
            session_config = session_config.with_default_model(model);
        }
        if let Some(ref cli_path) = self._config.cli_path {
            session_config = session_config.with_cli_path(cli_path.to_string_lossy().to_string());
        }

        AgentSession::new(session_config).await
    }
}
