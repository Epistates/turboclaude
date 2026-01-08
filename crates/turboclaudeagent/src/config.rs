//! Agent SDK configuration

use crate::error::Result;
use crate::mcp::SdkMcpServer;
use crate::sandbox::SandboxSettings;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use turboclaude::types::beta::{BetaToolParam, CompactionControl, OutputConfig};
use turboclaude_protocol::PermissionMode;
use turboclaude_transport::http::RetryPolicy;

/// Tools option for agent configuration
///
/// Controls which tools are available to the agent during execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolsOption {
    /// Predefined tool preset
    Preset(ToolsPreset),
    /// Explicit list of tools
    List(Vec<BetaToolParam>),
}

impl Default for ToolsOption {
    fn default() -> Self {
        Self::Preset(ToolsPreset::AllDefaultTools)
    }
}

/// Predefined tool presets
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolsPreset {
    /// All available tools including beta tools
    AllTools,
    /// All default tools (standard set)
    AllDefaultTools,
    /// No tools enabled
    None,
}

impl ToolsOption {
    /// Create a tools option with all tools
    pub fn all_tools() -> Self {
        Self::Preset(ToolsPreset::AllTools)
    }

    /// Create a tools option with all default tools
    pub fn all_default_tools() -> Self {
        Self::Preset(ToolsPreset::AllDefaultTools)
    }

    /// Create a tools option with no tools
    pub fn none() -> Self {
        Self::Preset(ToolsPreset::None)
    }

    /// Create a tools option with specific tools
    pub fn with_tools(tools: Vec<BetaToolParam>) -> Self {
        Self::List(tools)
    }

    /// Check if this option includes any tools
    pub fn has_tools(&self) -> bool {
        match self {
            Self::Preset(ToolsPreset::None) => false,
            Self::List(tools) => !tools.is_empty(),
            _ => true,
        }
    }
}

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

    /// Sandbox settings for agent isolation
    ///
    /// Controls how the agent is isolated from the host system.
    /// When enabled, restricts filesystem and network access.
    pub sandbox: Option<SandboxSettings>,

    /// Tools configuration for the agent
    ///
    /// Controls which tools are available during execution.
    /// Defaults to all default tools.
    pub tools: ToolsOption,

    /// Output configuration for response generation
    ///
    /// Controls effort level and other output parameters.
    pub output_config: Option<OutputConfig>,

    /// Compaction control for conversation history
    ///
    /// Enables automatic summarization of conversation history
    /// when approaching context window limits.
    pub compaction: Option<CompactionControl>,
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
            sandbox: None,
            tools: ToolsOption::default(),
            output_config: None,
            compaction: None,
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

    /// Set sandbox settings for agent isolation
    ///
    /// When enabled, restricts filesystem and network access to protect
    /// the host system from unintended side effects.
    ///
    /// # Example
    ///
    /// ```rust
    /// use turboclaudeagent::{SessionConfig, SandboxSettings};
    ///
    /// let config = SessionConfig::new()
    ///     .with_sandbox(SandboxSettings::enabled()
    ///         .with_auto_allow_bash_if_sandboxed(true));
    /// ```
    pub fn with_sandbox(mut self, sandbox: SandboxSettings) -> Self {
        self.sandbox = Some(sandbox);
        self
    }

    /// Check if sandbox is enabled
    pub fn is_sandboxed(&self) -> bool {
        self.sandbox.as_ref().is_some_and(|s| s.is_enabled())
    }

    /// Set tools configuration
    ///
    /// Controls which tools are available during execution.
    ///
    /// # Example
    ///
    /// ```rust
    /// use turboclaudeagent::{SessionConfig, ToolsOption};
    /// use turboclaude::types::beta::BetaToolParam;
    ///
    /// // Use all default tools
    /// let config = SessionConfig::new()
    ///     .with_tools(ToolsOption::all_default_tools());
    ///
    /// // Use specific tools
    /// let config = SessionConfig::new()
    ///     .with_tools(ToolsOption::with_tools(vec![
    ///         BetaToolParam::bash(),
    ///         BetaToolParam::text_editor(),
    ///     ]));
    /// ```
    pub fn with_tools(mut self, tools: ToolsOption) -> Self {
        self.tools = tools;
        self
    }

    /// Set output configuration
    ///
    /// Controls response generation behavior including effort level.
    ///
    /// # Example
    ///
    /// ```rust
    /// use turboclaudeagent::SessionConfig;
    /// use turboclaude::types::beta::OutputConfig;
    ///
    /// let config = SessionConfig::new()
    ///     .with_output_config(OutputConfig::high_effort());
    /// ```
    pub fn with_output_config(mut self, config: OutputConfig) -> Self {
        self.output_config = Some(config);
        self
    }

    /// Set compaction control
    ///
    /// Enables automatic summarization of conversation history
    /// when approaching context window limits.
    ///
    /// # Example
    ///
    /// ```rust
    /// use turboclaudeagent::SessionConfig;
    /// use turboclaude::types::beta::CompactionControl;
    ///
    /// let config = SessionConfig::new()
    ///     .with_compaction(CompactionControl::enabled()
    ///         .with_threshold_tokens(150_000)
    ///         .with_target_tokens(80_000));
    /// ```
    pub fn with_compaction(mut self, compaction: CompactionControl) -> Self {
        self.compaction = Some(compaction);
        self
    }

    /// Check if compaction is enabled
    pub fn has_compaction(&self) -> bool {
        self.compaction.as_ref().is_some_and(|c| c.enabled)
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

    #[test]
    fn test_sandbox_config() {
        let config = SessionConfig::new()
            .with_sandbox(SandboxSettings::enabled());

        assert!(config.is_sandboxed());
        assert!(config.sandbox.is_some());
    }

    #[test]
    fn test_sandbox_disabled_by_default() {
        let config = SessionConfig::default();
        assert!(!config.is_sandboxed());
        assert!(config.sandbox.is_none());
    }

    #[test]
    fn test_sandbox_with_settings() {
        use crate::sandbox::SandboxNetworkConfig;

        let config = SessionConfig::new()
            .with_sandbox(
                SandboxSettings::enabled()
                    .with_auto_allow_bash_if_sandboxed(true)
                    .with_network(SandboxNetworkConfig::new().with_allow_browser(true))
            );

        assert!(config.is_sandboxed());
        let sandbox = config.sandbox.as_ref().unwrap();
        assert_eq!(sandbox.auto_allow_bash_if_sandboxed, Some(true));
        assert!(sandbox.network.as_ref().unwrap().is_browser_allowed());
    }

    #[test]
    fn test_tools_option_preset() {
        let preset = ToolsOption::all_tools();
        assert!(preset.has_tools());

        let none = ToolsOption::none();
        assert!(!none.has_tools());
    }

    #[test]
    fn test_tools_option_list() {
        use turboclaude::types::beta::BetaToolParam;

        let tools = ToolsOption::with_tools(vec![
            BetaToolParam::bash(),
            BetaToolParam::text_editor(),
        ]);
        assert!(tools.has_tools());

        let empty = ToolsOption::with_tools(vec![]);
        assert!(!empty.has_tools());
    }

    #[test]
    fn test_config_with_tools() {
        let config = SessionConfig::new()
            .with_tools(ToolsOption::all_tools());

        match config.tools {
            ToolsOption::Preset(p) => assert_eq!(p, ToolsPreset::AllTools),
            _ => panic!("Expected preset"),
        }
    }

    #[test]
    fn test_config_with_output() {
        use turboclaude::types::beta::{OutputConfig, OutputEffort};

        let config = SessionConfig::new()
            .with_output_config(OutputConfig::high_effort());

        let output = config.output_config.unwrap();
        assert_eq!(output.effort, Some(OutputEffort::High));
    }

    #[test]
    fn test_config_with_compaction() {
        use turboclaude::types::beta::CompactionControl;

        let config = SessionConfig::new()
            .with_compaction(
                CompactionControl::enabled()
                    .with_threshold_tokens(150_000)
                    .with_target_tokens(80_000)
            );

        assert!(config.has_compaction());
        let compaction = config.compaction.unwrap();
        assert!(compaction.enabled);
        assert_eq!(compaction.threshold_tokens, Some(150_000));
        assert_eq!(compaction.target_tokens, Some(80_000));
    }

    #[test]
    fn test_compaction_disabled_by_default() {
        let config = SessionConfig::default();
        assert!(!config.has_compaction());
    }
}
