//! MCP Server Configuration Types
//!
//! Types for configuring MCP (Model Context Protocol) servers in API requests.

use crate::types::CacheControl;
use serde::{Deserialize, Serialize};

/// Configuration for which tools are allowed from an MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerToolConfiguration {
    /// Optional list of allowed tools (whitelist)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<Vec<String>>,
    /// Whether this server is enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

impl McpServerToolConfiguration {
    /// Create a new tool configuration with allowed tools
    pub fn with_allowed_tools(tools: Vec<String>) -> Self {
        Self {
            allowed_tools: Some(tools),
            enabled: Some(true),
        }
    }

    /// Create a disabled configuration
    pub fn disabled() -> Self {
        Self {
            allowed_tools: None,
            enabled: Some(false),
        }
    }
}

/// MCP server definition via URL endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerUrlDefinition {
    /// Server name identifier
    pub name: String,
    /// Type identifier (always "url")
    #[serde(rename = "type")]
    pub definition_type: String,
    /// Server endpoint URL
    pub url: String,
    /// Optional authorization token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_token: Option<String>,
    /// Optional tool configuration for this server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_configuration: Option<McpServerToolConfiguration>,
}

impl McpServerUrlDefinition {
    /// Create a new MCP server URL definition
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            definition_type: "url".to_string(),
            url: url.into(),
            authorization_token: None,
            tool_configuration: None,
        }
    }

    /// Set authorization token
    pub fn with_auth_token(mut self, token: impl Into<String>) -> Self {
        self.authorization_token = Some(token.into());
        self
    }

    /// Set tool configuration
    pub fn with_tool_configuration(mut self, config: McpServerToolConfiguration) -> Self {
        self.tool_configuration = Some(config);
        self
    }
}

/// MCP tool use block for invoking MCP tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolUseBlock {
    /// Unique identifier for this tool use
    pub id: String,
    /// Name of the MCP tool
    pub name: String,
    /// MCP server name
    pub server_name: String,
    /// Input parameters for the tool
    pub input: serde_json::Value,
    /// Type identifier (always "mcp_tool_use")
    #[serde(rename = "type")]
    pub block_type: String,
    /// Optional cache control
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// MCP tool result block for returning tool results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResultBlock {
    /// ID of the tool use this is responding to
    pub tool_use_id: String,
    /// Result content (can be string or array of text blocks)
    pub content: McpToolResultContent,
    /// Whether this indicates an error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    /// Type identifier (always "mcp_tool_result")
    #[serde(rename = "type")]
    pub block_type: String,
}

/// Content for MCP tool result - can be a string or array of text blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpToolResultContent {
    /// Simple string content
    Text(String),
    /// Array of text blocks
    Blocks(Vec<McpToolResultTextBlock>),
}

/// Text block within MCP tool result content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResultTextBlock {
    /// Type identifier (always "text")
    #[serde(rename = "type")]
    pub block_type: String,
    /// Text content
    pub text: String,
}

impl McpToolResultTextBlock {
    /// Create a new text block
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            block_type: "text".to_string(),
            text: text.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_url_definition() {
        let server = McpServerUrlDefinition::new("my_server", "http://localhost:8080")
            .with_auth_token("secret_token");

        let json = serde_json::to_value(&server).unwrap();

        assert_eq!(json["name"], "my_server");
        assert_eq!(json["type"], "url");
        assert_eq!(json["url"], "http://localhost:8080");
        assert_eq!(json["authorization_token"], "secret_token");
    }

    #[test]
    fn test_mcp_server_tool_configuration() {
        let config = McpServerToolConfiguration::with_allowed_tools(vec![
            "tool1".to_string(),
            "tool2".to_string(),
        ]);

        let json = serde_json::to_value(&config).unwrap();

        assert!(json["allowed_tools"].is_array());
        assert_eq!(json["enabled"], true);
    }

    #[test]
    fn test_mcp_tool_result_content() {
        // Test string content
        let text_content = McpToolResultContent::Text("Result".to_string());
        let json = serde_json::to_value(&text_content).unwrap();
        assert_eq!(json, "Result");

        // Test blocks content
        let blocks_content = McpToolResultContent::Blocks(vec![
            McpToolResultTextBlock::new("Line 1"),
            McpToolResultTextBlock::new("Line 2"),
        ]);
        let json = serde_json::to_value(&blocks_content).unwrap();
        assert!(json.is_array());
        assert_eq!(json[0]["type"], "text");
        assert_eq!(json[0]["text"], "Line 1");
    }
}
