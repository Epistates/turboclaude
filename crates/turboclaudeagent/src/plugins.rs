//! Plugin system for extending Claude Code with custom functionality
//!
//! Plugins allow extending Claude Code with custom commands, agents, skills, and hooks.
//! Currently, the system supports loading local plugins from the filesystem.
//!
//! # Plugin Structure
//!
//! A plugin directory should have the following structure:
//! ```text
//! my-plugin/
//! ├── .claude-plugin/
//! │   └── plugin.json
//! └── commands/
//!     └── my-command.md
//! ```
//!
//! # Example
//!
//! ```rust
//! use turboclaudeagent::plugins::SdkPluginConfig;
//! use std::path::PathBuf;
//!
//! let plugin_config = SdkPluginConfig {
//!     plugin_type: "local".to_string(),
//!     path: "/path/to/my-plugin".to_string(),
//! };
//! ```

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// SDK plugin configuration
///
/// Specifies a plugin to be loaded. Currently only local plugins are supported.
///
/// # Fields
///
/// * `plugin_type` - The type of plugin (currently only "local" is supported)
/// * `path` - Path to the plugin directory (relative or absolute)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SdkPluginConfig {
    /// Plugin type (currently "local")
    #[serde(rename = "type")]
    pub plugin_type: String,

    /// Path to plugin directory
    pub path: String,
}

impl SdkPluginConfig {
    /// Create a new local plugin configuration
    ///
    /// # Arguments
    /// * `path` - Path to the plugin directory
    ///
    /// # Example
    /// ```rust
    /// use turboclaudeagent::plugins::SdkPluginConfig;
    ///
    /// let config = SdkPluginConfig::local("./plugins/my-plugin");
    /// ```
    pub fn local(path: impl Into<String>) -> Self {
        Self {
            plugin_type: "local".to_string(),
            path: path.into(),
        }
    }

    /// Validate that the plugin path exists and is a directory
    pub fn validate(&self) -> Result<(), String> {
        if self.plugin_type != "local" {
            return Err(format!(
                "Unsupported plugin type: {}. Only 'local' is supported.",
                self.plugin_type
            ));
        }

        let path = Path::new(&self.path);
        if !path.exists() {
            return Err(format!("Plugin path does not exist: {}", self.path));
        }

        if !path.is_dir() {
            return Err(format!("Plugin path is not a directory: {}", self.path));
        }

        Ok(())
    }
}

/// Metadata about a plugin
///
/// Loaded from `.claude-plugin/plugin.json` in the plugin directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin name
    pub name: String,

    /// Plugin description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Plugin version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Plugin author
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

/// A loaded plugin
///
/// Represents a plugin that has been discovered and loaded from the filesystem.
#[derive(Debug, Clone)]
pub struct Plugin {
    /// Plugin metadata
    pub metadata: PluginMetadata,

    /// Path to the plugin directory
    pub path: PathBuf,

    /// List of available commands
    pub commands: Vec<String>,
}

impl Plugin {
    /// Load a plugin from the specified path
    ///
    /// # Arguments
    /// * `path` - Path to the plugin directory
    ///
    /// # Errors
    /// Returns an error if:
    /// - The plugin directory doesn't exist
    /// - The .claude-plugin/plugin.json file is missing or invalid
    /// - JSON parsing fails
    ///
    /// # Example
    /// ```no_run
    /// use turboclaudeagent::plugins::Plugin;
    ///
    /// let plugin = Plugin::from_path("./plugins/my-plugin")
    ///     .expect("Failed to load plugin");
    /// ```
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();

        if !path.is_dir() {
            return Err(format!("Plugin path is not a directory: {:?}", path));
        }

        // Load plugin metadata
        let metadata_path = path.join(".claude-plugin").join("plugin.json");
        let metadata_content = fs::read_to_string(&metadata_path)
            .map_err(|e| format!("Failed to read plugin.json: {}", e))?;

        let metadata: PluginMetadata = serde_json::from_str(&metadata_content)
            .map_err(|e| format!("Invalid plugin.json: {}", e))?;

        // Discover commands
        let mut commands = Vec::new();
        let commands_dir = path.join("commands");

        if commands_dir.exists()
            && commands_dir.is_dir()
            && let Ok(entries) = fs::read_dir(&commands_dir)
        {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type()
                    && file_type.is_file()
                    && let Some(file_name) = entry.file_name().to_str()
                    && file_name.ends_with(".md")
                {
                    // Remove .md extension
                    let command_name = file_name.trim_end_matches(".md").to_string();
                    commands.push(command_name);
                }
            }
        }

        commands.sort();

        Ok(Plugin {
            metadata,
            path: path.to_path_buf(),
            commands,
        })
    }
}

/// Plugin loader for discovering and loading plugins
///
/// Handles loading plugins from filesystem paths and discovering available commands.
pub struct PluginLoader {
    config: SdkPluginConfig,
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new(config: SdkPluginConfig) -> Self {
        Self { config }
    }

    /// Get the plugin configuration
    pub fn config(&self) -> &SdkPluginConfig {
        &self.config
    }

    /// Load the plugin
    ///
    /// # Errors
    /// Returns an error if the plugin cannot be loaded or the configuration is invalid
    pub fn load(&self) -> Result<Plugin, String> {
        self.config.validate()?;
        Plugin::from_path(&self.config.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== SdkPluginConfig Tests =====

    #[test]
    fn test_sdk_plugin_config_local() {
        let config = SdkPluginConfig::local("./plugins/test");
        assert_eq!(config.plugin_type, "local");
        assert_eq!(config.path, "./plugins/test");
    }

    #[test]
    fn test_sdk_plugin_config_serialization() {
        let config = SdkPluginConfig::local("./my-plugin");
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"type\":\"local\""));
        assert!(json.contains("\"path\":\"./my-plugin\""));
    }

    #[test]
    fn test_sdk_plugin_config_deserialization() {
        let json = r#"{"type":"local","path":"/path/to/plugin"}"#;
        let config: SdkPluginConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.plugin_type, "local");
        assert_eq!(config.path, "/path/to/plugin");
    }

    #[test]
    fn test_sdk_plugin_config_equality() {
        let config1 = SdkPluginConfig::local("./plugin");
        let config2 = SdkPluginConfig::local("./plugin");
        assert_eq!(config1, config2);
    }

    #[test]
    fn test_sdk_plugin_config_validate_invalid_type() {
        let config = SdkPluginConfig {
            plugin_type: "remote".to_string(),
            path: "./plugin".to_string(),
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_sdk_plugin_config_validate_nonexistent_path() {
        let config = SdkPluginConfig::local("/nonexistent/path/12345");
        assert!(config.validate().is_err());
    }

    // ===== PluginMetadata Tests =====

    #[test]
    fn test_plugin_metadata_creation() {
        let metadata = PluginMetadata {
            name: "test-plugin".to_string(),
            description: Some("Test description".to_string()),
            version: Some("1.0.0".to_string()),
            author: Some("Test Author".to_string()),
        };
        assert_eq!(metadata.name, "test-plugin");
        assert_eq!(metadata.description, Some("Test description".to_string()));
    }

    #[test]
    fn test_plugin_metadata_serialization() {
        let metadata = PluginMetadata {
            name: "my-plugin".to_string(),
            description: Some("A test plugin".to_string()),
            version: Some("1.0.0".to_string()),
            author: None,
        };
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"name\":\"my-plugin\""));
        assert!(json.contains("\"description\":\"A test plugin\""));
        assert!(json.contains("\"version\":\"1.0.0\""));
        assert!(!json.contains("\"author\""));
    }

    #[test]
    fn test_plugin_metadata_deserialization() {
        let json =
            r#"{"name":"test","description":"Test plugin","version":"1.0","author":"Author"}"#;
        let metadata: PluginMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.name, "test");
        assert_eq!(metadata.description, Some("Test plugin".to_string()));
        assert_eq!(metadata.version, Some("1.0".to_string()));
        assert_eq!(metadata.author, Some("Author".to_string()));
    }

    #[test]
    fn test_plugin_metadata_minimal() {
        let json = r#"{"name":"minimal"}"#;
        let metadata: PluginMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.name, "minimal");
        assert!(metadata.description.is_none());
        assert!(metadata.version.is_none());
        assert!(metadata.author.is_none());
    }

    // ===== Plugin Tests =====

    #[test]
    fn test_plugin_creation() {
        let metadata = PluginMetadata {
            name: "test-plugin".to_string(),
            description: None,
            version: None,
            author: None,
        };
        let plugin = Plugin {
            metadata,
            path: PathBuf::from("./plugin"),
            commands: vec!["cmd1".to_string(), "cmd2".to_string()],
        };
        assert_eq!(plugin.metadata.name, "test-plugin");
        assert_eq!(plugin.commands.len(), 2);
    }

    #[test]
    fn test_plugin_commands_sorted() {
        let metadata = PluginMetadata {
            name: "test".to_string(),
            description: None,
            version: None,
            author: None,
        };
        let mut plugin = Plugin {
            metadata,
            path: PathBuf::from("./plugin"),
            commands: vec![
                "zebra".to_string(),
                "apple".to_string(),
                "monkey".to_string(),
            ],
        };
        plugin.commands.sort();
        assert_eq!(plugin.commands[0], "apple");
        assert_eq!(plugin.commands[2], "zebra");
    }

    // ===== PluginLoader Tests =====

    #[test]
    fn test_plugin_loader_creation() {
        let config = SdkPluginConfig::local("./plugin");
        let loader = PluginLoader::new(config);
        assert_eq!(loader.config.plugin_type, "local");
    }

    #[test]
    fn test_plugin_loader_load_nonexistent() {
        let config = SdkPluginConfig::local("/nonexistent/plugin");
        let loader = PluginLoader::new(config);
        assert!(loader.load().is_err());
    }

    // ===== Integration Tests =====

    #[test]
    fn test_plugin_config_and_loader_workflow() {
        // Create a plugin config
        let config = SdkPluginConfig::local("./my-plugin");

        // Validate the config (will fail for this test since path doesn't exist)
        match config.validate() {
            Ok(_) => {
                // If validation passes, we could load
                let loader = PluginLoader::new(config);
                let _ = loader.load();
            }
            Err(_) => {
                // Expected for this test
                assert_eq!(config.plugin_type, "local");
            }
        }
    }

    #[test]
    fn test_multiple_plugins_configs() {
        let config1 = SdkPluginConfig::local("./plugin1");
        let config2 = SdkPluginConfig::local("./plugin2");

        assert_ne!(config1.path, config2.path);
        assert_eq!(config1.plugin_type, config2.plugin_type);
    }

    #[test]
    fn test_plugin_metadata_round_trip() {
        let original = PluginMetadata {
            name: "round-trip".to_string(),
            description: Some("Test".to_string()),
            version: Some("2.0.0".to_string()),
            author: Some("Test Author".to_string()),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: PluginMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(original.name, deserialized.name);
        assert_eq!(original.description, deserialized.description);
        assert_eq!(original.version, deserialized.version);
        assert_eq!(original.author, deserialized.author);
    }
}
