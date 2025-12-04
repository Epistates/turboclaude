//! Sandbox configuration types for agent isolation
//!
//! This module provides types for configuring sandbox environments that isolate
//! agent execution from the host system. Sandboxing helps prevent unintended
//! side effects from tool execution.
//!
//! # Overview
//!
//! The sandbox system supports:
//! - File system isolation (preventing access outside allowed directories)
//! - Network isolation (controlling outbound connections)
//! - Command exclusion (blocking specific shell commands)
//!
//! # Example
//!
//! ```rust
//! use turboclaudeagent::sandbox::{SandboxSettings, SandboxNetworkConfig};
//!
//! // Enable sandboxing with network access for browser operations
//! let settings = SandboxSettings::default()
//!     .with_enabled(true)
//!     .with_network(SandboxNetworkConfig::default().with_allow_browser(true));
//! ```

use serde::{Deserialize, Serialize};

/// Configuration for sandbox environment
///
/// Controls how the agent is isolated from the host system during execution.
/// When enabled, tool operations are restricted to prevent unintended side effects.
///
/// # Fields
///
/// - `enabled`: Whether sandboxing is active
/// - `auto_allow_bash_if_sandboxed`: Auto-approve Bash commands when sandboxed
/// - `excluded_commands`: Shell commands that are always blocked
/// - `allow_unsandboxed_commands`: Allow running commands outside the sandbox
/// - `network`: Network isolation configuration
/// - `ignore_violations`: Which violation types to ignore
/// - `enable_weaker_nested_sandbox`: Use a less restrictive sandbox for nested calls
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SandboxSettings {
    /// Whether sandbox mode is enabled
    ///
    /// When true, tool execution is restricted based on other settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Automatically allow Bash commands when running in sandbox mode
    ///
    /// When true and sandboxed, Bash tool calls are auto-approved without
    /// requiring explicit permission.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_allow_bash_if_sandboxed: Option<bool>,

    /// List of shell commands that are never allowed
    ///
    /// These commands are blocked regardless of sandbox state.
    /// Example: ["rm -rf", "dd", "mkfs"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excluded_commands: Option<Vec<String>>,

    /// Allow running commands that cannot be sandboxed
    ///
    /// Some commands may not support sandboxing. When true, these
    /// commands are allowed to run unsandboxed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_unsandboxed_commands: Option<bool>,

    /// Network isolation configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<SandboxNetworkConfig>,

    /// Violations to ignore during sandbox enforcement
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_violations: Option<SandboxIgnoreViolations>,

    /// Enable a weaker sandbox for nested tool calls
    ///
    /// When true, nested tool calls use a less restrictive sandbox,
    /// which may be needed for certain workflows.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_weaker_nested_sandbox: Option<bool>,
}

impl SandboxSettings {
    /// Create new sandbox settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create enabled sandbox settings
    pub fn enabled() -> Self {
        Self {
            enabled: Some(true),
            ..Default::default()
        }
    }

    /// Create disabled sandbox settings
    pub fn disabled() -> Self {
        Self {
            enabled: Some(false),
            ..Default::default()
        }
    }

    /// Set whether sandbox is enabled
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    /// Set whether to auto-allow Bash commands when sandboxed
    pub fn with_auto_allow_bash_if_sandboxed(mut self, auto_allow: bool) -> Self {
        self.auto_allow_bash_if_sandboxed = Some(auto_allow);
        self
    }

    /// Set excluded commands
    pub fn with_excluded_commands(mut self, commands: Vec<String>) -> Self {
        self.excluded_commands = Some(commands);
        self
    }

    /// Add an excluded command
    pub fn add_excluded_command(mut self, command: impl Into<String>) -> Self {
        self.excluded_commands
            .get_or_insert_with(Vec::new)
            .push(command.into());
        self
    }

    /// Set whether to allow unsandboxed commands
    pub fn with_allow_unsandboxed_commands(mut self, allow: bool) -> Self {
        self.allow_unsandboxed_commands = Some(allow);
        self
    }

    /// Set network configuration
    pub fn with_network(mut self, network: SandboxNetworkConfig) -> Self {
        self.network = Some(network);
        self
    }

    /// Set violations to ignore
    pub fn with_ignore_violations(mut self, ignore: SandboxIgnoreViolations) -> Self {
        self.ignore_violations = Some(ignore);
        self
    }

    /// Set whether to enable weaker nested sandbox
    pub fn with_enable_weaker_nested_sandbox(mut self, enable: bool) -> Self {
        self.enable_weaker_nested_sandbox = Some(enable);
        self
    }

    /// Check if sandboxing is effectively enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.unwrap_or(false)
    }

    /// Check if a command is excluded
    pub fn is_command_excluded(&self, command: &str) -> bool {
        self.excluded_commands
            .as_ref()
            .map(|cmds| cmds.iter().any(|c| command.contains(c)))
            .unwrap_or(false)
    }
}

/// Network configuration for sandbox
///
/// Controls network access within the sandboxed environment.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SandboxNetworkConfig {
    /// Allow browser-based network operations
    ///
    /// When true, browser tools can make network requests.
    /// This is useful for web scraping or automation tasks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_browser: Option<bool>,
}

impl SandboxNetworkConfig {
    /// Create new network configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to allow browser network access
    pub fn with_allow_browser(mut self, allow: bool) -> Self {
        self.allow_browser = Some(allow);
        self
    }

    /// Check if browser network access is allowed
    pub fn is_browser_allowed(&self) -> bool {
        self.allow_browser.unwrap_or(false)
    }
}

/// Violation types to ignore during sandbox enforcement
///
/// Allows selectively bypassing certain sandbox restrictions
/// when they are known to be safe for a specific use case.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SandboxIgnoreViolations {
    /// Ignore filesystem access violations
    ///
    /// When true, filesystem operations outside the sandbox
    /// won't trigger violations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filesystem: Option<bool>,

    /// Ignore network access violations
    ///
    /// When true, network operations outside the sandbox
    /// won't trigger violations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<bool>,
}

impl SandboxIgnoreViolations {
    /// Create new ignore violations configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to ignore filesystem violations
    pub fn with_filesystem(mut self, ignore: bool) -> Self {
        self.filesystem = Some(ignore);
        self
    }

    /// Set whether to ignore network violations
    pub fn with_network(mut self, ignore: bool) -> Self {
        self.network = Some(ignore);
        self
    }

    /// Check if filesystem violations are ignored
    pub fn ignores_filesystem(&self) -> bool {
        self.filesystem.unwrap_or(false)
    }

    /// Check if network violations are ignored
    pub fn ignores_network(&self) -> bool {
        self.network.unwrap_or(false)
    }

    /// Check if any violations are ignored
    pub fn ignores_any(&self) -> bool {
        self.ignores_filesystem() || self.ignores_network()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== SandboxSettings Tests =====

    #[test]
    fn test_sandbox_settings_default() {
        let settings = SandboxSettings::default();
        assert!(settings.enabled.is_none());
        assert!(!settings.is_enabled());
    }

    #[test]
    fn test_sandbox_settings_enabled() {
        let settings = SandboxSettings::enabled();
        assert_eq!(settings.enabled, Some(true));
        assert!(settings.is_enabled());
    }

    #[test]
    fn test_sandbox_settings_disabled() {
        let settings = SandboxSettings::disabled();
        assert_eq!(settings.enabled, Some(false));
        assert!(!settings.is_enabled());
    }

    #[test]
    fn test_sandbox_settings_builder() {
        let settings = SandboxSettings::new()
            .with_enabled(true)
            .with_auto_allow_bash_if_sandboxed(true)
            .with_allow_unsandboxed_commands(false)
            .with_enable_weaker_nested_sandbox(true);

        assert!(settings.is_enabled());
        assert_eq!(settings.auto_allow_bash_if_sandboxed, Some(true));
        assert_eq!(settings.allow_unsandboxed_commands, Some(false));
        assert_eq!(settings.enable_weaker_nested_sandbox, Some(true));
    }

    #[test]
    fn test_sandbox_settings_excluded_commands() {
        let settings = SandboxSettings::new()
            .with_excluded_commands(vec!["rm -rf".to_string(), "dd".to_string()])
            .add_excluded_command("mkfs");

        assert!(settings.is_command_excluded("rm -rf /"));
        assert!(settings.is_command_excluded("dd if=/dev/zero"));
        assert!(settings.is_command_excluded("mkfs.ext4"));
        assert!(!settings.is_command_excluded("ls"));
    }

    #[test]
    fn test_sandbox_settings_serialization() {
        let settings = SandboxSettings::new()
            .with_enabled(true)
            .with_auto_allow_bash_if_sandboxed(true);

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"autoAllowBashIfSandboxed\":true"));

        let deserialized: SandboxSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, deserialized);
    }

    #[test]
    fn test_sandbox_settings_skip_serializing_none() {
        let settings = SandboxSettings::new().with_enabled(true);

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"enabled\":true"));
        // Should not contain fields that are None
        assert!(!json.contains("autoAllowBashIfSandboxed"));
        assert!(!json.contains("excludedCommands"));
    }

    // ===== SandboxNetworkConfig Tests =====

    #[test]
    fn test_network_config_default() {
        let config = SandboxNetworkConfig::default();
        assert!(config.allow_browser.is_none());
        assert!(!config.is_browser_allowed());
    }

    #[test]
    fn test_network_config_builder() {
        let config = SandboxNetworkConfig::new().with_allow_browser(true);
        assert!(config.is_browser_allowed());
    }

    #[test]
    fn test_network_config_serialization() {
        let config = SandboxNetworkConfig::new().with_allow_browser(true);

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"allowBrowser\":true"));

        let deserialized: SandboxNetworkConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    // ===== SandboxIgnoreViolations Tests =====

    #[test]
    fn test_ignore_violations_default() {
        let ignore = SandboxIgnoreViolations::default();
        assert!(!ignore.ignores_filesystem());
        assert!(!ignore.ignores_network());
        assert!(!ignore.ignores_any());
    }

    #[test]
    fn test_ignore_violations_builder() {
        let ignore = SandboxIgnoreViolations::new()
            .with_filesystem(true)
            .with_network(true);

        assert!(ignore.ignores_filesystem());
        assert!(ignore.ignores_network());
        assert!(ignore.ignores_any());
    }

    #[test]
    fn test_ignore_violations_partial() {
        let ignore = SandboxIgnoreViolations::new().with_filesystem(true);

        assert!(ignore.ignores_filesystem());
        assert!(!ignore.ignores_network());
        assert!(ignore.ignores_any());
    }

    #[test]
    fn test_ignore_violations_serialization() {
        let ignore = SandboxIgnoreViolations::new()
            .with_filesystem(true)
            .with_network(false);

        let json = serde_json::to_string(&ignore).unwrap();
        assert!(json.contains("\"filesystem\":true"));
        assert!(json.contains("\"network\":false"));

        let deserialized: SandboxIgnoreViolations = serde_json::from_str(&json).unwrap();
        assert_eq!(ignore, deserialized);
    }

    // ===== Integration Tests =====

    #[test]
    fn test_full_sandbox_configuration() {
        let settings = SandboxSettings::new()
            .with_enabled(true)
            .with_auto_allow_bash_if_sandboxed(true)
            .with_excluded_commands(vec!["rm -rf".to_string()])
            .with_network(SandboxNetworkConfig::new().with_allow_browser(true))
            .with_ignore_violations(
                SandboxIgnoreViolations::new()
                    .with_filesystem(false)
                    .with_network(true),
            );

        // Verify all fields are set
        assert!(settings.is_enabled());
        assert_eq!(settings.auto_allow_bash_if_sandboxed, Some(true));
        assert!(settings.is_command_excluded("rm -rf /home"));
        assert!(settings.network.as_ref().unwrap().is_browser_allowed());
        assert!(!settings.ignore_violations.as_ref().unwrap().ignores_filesystem());
        assert!(settings.ignore_violations.as_ref().unwrap().ignores_network());
    }

    #[test]
    fn test_sandbox_configuration_round_trip() {
        let settings = SandboxSettings::new()
            .with_enabled(true)
            .with_auto_allow_bash_if_sandboxed(true)
            .with_excluded_commands(vec!["dd".to_string(), "mkfs".to_string()])
            .with_allow_unsandboxed_commands(false)
            .with_network(SandboxNetworkConfig::new().with_allow_browser(true))
            .with_ignore_violations(SandboxIgnoreViolations::new().with_filesystem(true))
            .with_enable_weaker_nested_sandbox(true);

        let json = serde_json::to_string(&settings).unwrap();
        let restored: SandboxSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings, restored);
    }

    #[test]
    fn test_python_sdk_compatibility() {
        // Test deserialization of JSON that matches Python SDK format
        let python_json = r#"{
            "enabled": true,
            "autoAllowBashIfSandboxed": true,
            "excludedCommands": ["rm -rf", "dd"],
            "allowUnsandboxedCommands": false,
            "network": {"allowBrowser": true},
            "ignoreViolations": {"filesystem": false, "network": true},
            "enableWeakerNestedSandbox": false
        }"#;

        let settings: SandboxSettings = serde_json::from_str(python_json).unwrap();

        assert!(settings.is_enabled());
        assert_eq!(settings.auto_allow_bash_if_sandboxed, Some(true));
        assert_eq!(
            settings.excluded_commands,
            Some(vec!["rm -rf".to_string(), "dd".to_string()])
        );
        assert_eq!(settings.allow_unsandboxed_commands, Some(false));
        assert!(settings.network.as_ref().unwrap().is_browser_allowed());
        assert!(!settings.ignore_violations.as_ref().unwrap().ignores_filesystem());
        assert!(settings.ignore_violations.as_ref().unwrap().ignores_network());
        assert_eq!(settings.enable_weaker_nested_sandbox, Some(false));
    }
}
