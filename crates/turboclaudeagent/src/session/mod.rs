//! Agent session for interactive conversations with Claude Code CLI
//!
//! Provides the public API for executing queries, registering hooks and permissions,
//! and controlling the agent session at runtime.
//!
//! # Module Organization
//!
//! The session module is organized into focused sub-modules:
//!
//! - [`state`] - Session state management and conversation history
//! - [`core`] - Core AgentSession struct and lifecycle methods (new, close, fork)
//! - [`query`] - Query execution and message streaming
//! - [`control`] - Runtime control (interrupts, model changes, permissions, hooks)
//!
//! # Examples
//!
//! Basic usage:
//! ```no_run
//! # use turboclaudeagent::ClaudeAgentClient;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = ClaudeAgentClient::builder()
//!     .api_key("your-api-key")
//!     .build()?;
//! let client = ClaudeAgentClient::new(config);
//! let session = client.create_session().await?;
//!
//! // Simple query
//! let response = session.query_str("What is 2+2?").await?;
//!
//! // Fork session to explore alternative path
//! let forked = session.fork().await?;
//! # Ok(())
//! # }
//! ```

pub mod control;
pub mod core;
pub mod query;
pub mod state;

// Re-export public types
pub use self::core::AgentSession;
pub use self::query::QueryBuilder;
pub use self::state::SessionState;

#[cfg(test)]
mod tests {
    use crate::config::SessionConfig;

    #[test]
    fn test_session_config_defaults() {
        let config = SessionConfig::default();
        assert_eq!(config.max_concurrent_queries, 1);
        assert!(!config.default_model.is_empty());
    }
}
