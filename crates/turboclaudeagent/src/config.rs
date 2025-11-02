//! Agent SDK configuration

use crate::error::Result;
use crate::mcp::SdkMcpServer;
use std::time::Duration;
use turboclaude_protocol::PermissionMode;
use turboclaude_transport::http::RetryPolicy;

/// Configuration for ClaudeAgentClient
#[derive(Debug, Clone)]
pub struct ClaudeAgentClientConfig {
    /// API key for Claude
    pub api_key: String,

    /// Model to use
    pub model: Option<String>,

    /// CLI path
    pub cli_path: Option<std::path::PathBuf>,
}

/// Configuration for an agent session
///
/// Controls how the agent SDK connects to Claude Code CLI and handles
/// queries, permissions, hooks, and error recovery.
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Path to the Claude Code CLI executable
    pub cli_path: String,

    /// Default model for queries if not specified
    pub default_model: String,

    /// Default system prompt for queries
    pub system_prompt: Option<String>,

    /// Maximum tokens in responses
    pub max_tokens: u32,

    /// Default permission mode for tool use
    pub permission_mode: PermissionMode,

    /// Retry policy for subprocess failures
    pub restart_policy: RetryPolicy,

    /// Timeout for individual requests
    pub request_timeout: Duration,

    /// Maximum number of concurrent queries
    pub max_concurrent_queries: usize,

    /// Directories to search for skills (requires 'skills' feature)
    #[cfg(feature = "skills")]
    pub skill_dirs: Vec<std::path::PathBuf>,

    /// SDK MCP servers for in-process tool execution
    pub sdk_servers: Vec<SdkMcpServer>,
}

impl ClaudeAgentClientConfig {
    /// Create a builder
    pub fn builder() -> ClaudeAgentClientBuilder {
        ClaudeAgentClientBuilder::default()
    }
}

/// Builder for ClaudeAgentClientConfig
#[derive(Debug, Default)]
pub struct ClaudeAgentClientBuilder {
    api_key: Option<String>,
    model: Option<String>,
    cli_path: Option<std::path::PathBuf>,
}

impl ClaudeAgentClientBuilder {
    /// Set the API key
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Set the model
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the CLI path
    pub fn cli_path(mut self, path: std::path::PathBuf) -> Self {
        self.cli_path = Some(path);
        self
    }

    /// Build the configuration
    pub fn build(self) -> Result<ClaudeAgentClientConfig> {
        let api_key = self
            .api_key
            .ok_or_else(|| crate::AgentError::Config("API key required".to_string()))?;

        Ok(ClaudeAgentClientConfig {
            api_key,
            model: self.model,
            cli_path: self.cli_path,
        })
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            cli_path: "claude".to_string(),
            default_model: "claude-3-5-sonnet-20241022".to_string(),
            system_prompt: None,
            max_tokens: 4096,
            permission_mode: PermissionMode::Default,
            restart_policy: RetryPolicy::default(),
            request_timeout: Duration::from_secs(300),
            max_concurrent_queries: 1, // Serial by default (safe)
            #[cfg(feature = "skills")]
            skill_dirs: vec![std::path::PathBuf::from("./skills")],
            sdk_servers: Vec::new(),
        }
    }
}

impl SessionConfig {
    /// Create a new session config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the CLI path
    pub fn with_cli_path(mut self, path: impl Into<String>) -> Self {
        self.cli_path = path.into();
        self
    }

    /// Set the default model
    pub fn with_default_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = model.into();
        self
    }

    /// Set the default system prompt
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Set the maximum tokens per response
    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = tokens;
        self
    }

    /// Set the permission mode
    pub fn with_permission_mode(mut self, mode: PermissionMode) -> Self {
        self.permission_mode = mode;
        self
    }

    /// Set the restart policy for subprocess failures
    pub fn with_restart_policy(mut self, policy: RetryPolicy) -> Self {
        self.restart_policy = policy;
        self
    }

    /// Set the request timeout
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Set the maximum concurrent queries
    ///
    /// Default is 1 (serial execution). Increase for concurrent parallel queries.
    pub fn with_concurrent_queries(mut self, count: usize) -> Self {
        self.max_concurrent_queries = std::cmp::max(count, 1);
        self
    }

    /// Set skill directories (requires 'skills' feature)
    ///
    /// Directories to search for SKILL.md files during discovery.
    #[cfg(feature = "skills")]
    pub fn with_skill_dirs(mut self, dirs: Vec<std::path::PathBuf>) -> Self {
        self.skill_dirs = dirs;
        self
    }

    /// Add a skill directory (requires 'skills' feature)
    ///
    /// Adds a single directory to the skill search path.
    #[cfg(feature = "skills")]
    pub fn add_skill_dir(mut self, dir: std::path::PathBuf) -> Self {
        self.skill_dirs.push(dir);
        self
    }

    /// Set SDK MCP servers for in-process tool execution
    ///
    /// These servers run within the same process, eliminating subprocess overhead.
    pub fn with_sdk_servers(mut self, servers: Vec<SdkMcpServer>) -> Self {
        self.sdk_servers = servers;
        self
    }

    /// Add a single SDK MCP server
    pub fn add_sdk_server(mut self, server: SdkMcpServer) -> Self {
        self.sdk_servers.push(server);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SessionConfig::default();
        assert_eq!(config.cli_path, "claude");
        assert_eq!(config.default_model, "claude-3-5-sonnet-20241022");
        assert_eq!(config.max_tokens, 4096);
        assert_eq!(config.permission_mode, PermissionMode::Default);
        assert_eq!(config.max_concurrent_queries, 1);
    }

    #[test]
    fn test_config_builder() {
        let config = SessionConfig::new()
            .with_cli_path("/usr/local/bin/claude")
            .with_default_model("claude-3-5-haiku-20241022")
            .with_max_tokens(2048)
            .with_permission_mode(PermissionMode::BypassPermissions)
            .with_concurrent_queries(4);

        assert_eq!(config.cli_path, "/usr/local/bin/claude");
        assert_eq!(config.default_model, "claude-3-5-haiku-20241022");
        assert_eq!(config.max_tokens, 2048);
        assert_eq!(config.permission_mode, PermissionMode::BypassPermissions);
        assert_eq!(config.max_concurrent_queries, 4);
    }

    #[test]
    fn test_system_prompt() {
        let config = SessionConfig::new().with_system_prompt("You are a helpful assistant");

        assert_eq!(
            config.system_prompt,
            Some("You are a helpful assistant".to_string())
        );
    }

    #[test]
    fn test_concurrent_queries_minimum_one() {
        let config = SessionConfig::new().with_concurrent_queries(0);
        assert_eq!(config.max_concurrent_queries, 1); // Minimum is 1
    }
}
