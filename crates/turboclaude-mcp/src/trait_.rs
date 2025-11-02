//! Core McpClient trait - SDK-agnostic abstraction for MCP operations

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::error::McpResult;

/// Server information provided during initialization
#[derive(Debug, Clone)]
pub struct ServerInfo {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
}

/// Tool descriptor
#[derive(Debug, Clone)]
pub struct ToolInfo {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: Option<String>,
    /// Input schema (JSON Schema)
    pub input_schema: Option<Value>,
}

/// Resource descriptor
#[derive(Debug, Clone)]
pub struct ResourceInfo {
    /// Resource URI
    pub uri: String,
    /// Resource name
    pub name: String,
    /// Resource description
    pub description: Option<String>,
    /// Whether the resource supports reading contents
    pub read_only: bool,
}

/// Resource contents
#[derive(Debug, Clone)]
pub struct ResourceContents {
    /// Resource URI
    pub uri: String,
    /// Resource MIME type
    pub mime_type: Option<String>,
    /// Resource text contents
    pub text: String,
}

/// Prompt descriptor
#[derive(Debug, Clone)]
pub struct PromptInfo {
    /// Prompt name
    pub name: String,
    /// Prompt description
    pub description: Option<String>,
    /// Prompt arguments specification
    pub arguments: Option<Vec<PromptArgument>>,
}

/// Prompt argument
#[derive(Debug, Clone)]
pub struct PromptArgument {
    /// Argument name
    pub name: String,
    /// Argument description
    pub description: Option<String>,
    /// Whether argument is required
    pub required: bool,
}

/// Tool result
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// Tool output as JSON value
    pub content: Value,
    /// Whether this is an error result
    pub is_error: bool,
}

/// Prompt result (message content blocks)
#[derive(Debug, Clone)]
pub struct PromptResult {
    /// Prompt output messages
    pub messages: Vec<MessageContent>,
}

/// Message content
#[derive(Debug, Clone)]
pub struct MessageContent {
    /// Content role (user, assistant)
    pub role: String,
    /// Content text
    pub text: String,
}

/// Core MCP client trait - SDK-agnostic abstraction
///
/// This trait provides a unified interface for interacting with MCP servers,
/// regardless of the underlying SDK implementation (TurboMCP, official Rust SDK, etc.).
///
/// # Example
///
/// ```ignore
/// use turboclaude_mcp::McpClient;
///
/// async fn example(client: &(impl McpClient + ?Sized)) -> Result<()> {
///     // Initialize connection
///     let info = client.initialize().await?;
///     println!("Connected to: {}", info.name);
///
///     // List available tools
///     let tools = client.list_tools().await?;
///     for tool in tools {
///         println!("Tool: {}", tool.name);
///     }
///
///     // Call a tool
///     let result = client.call_tool("my_tool", Some(json!({"key": "value"}))).await?;
///     println!("Result: {}", result.content);
///
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait McpClient: Send + Sync {
    // === Lifecycle Management ===

    /// Initialize the MCP client connection
    ///
    /// This must be called before any other operations. It negotiates capabilities
    /// with the server and establishes the connection.
    ///
    /// # Errors
    ///
    /// Returns an error if the server is unreachable or rejects the initialization
    async fn initialize(&self) -> McpResult<ServerInfo>;

    /// Close the MCP client connection
    ///
    /// After calling this, the client should not be used for further operations.
    async fn close(&self) -> McpResult<()>;

    // === Tool Operations ===

    /// List all available tools
    ///
    /// # Errors
    ///
    /// Returns an error if the server doesn't support tool listing or if the
    /// request times out
    async fn list_tools(&self) -> McpResult<Vec<ToolInfo>>;

    /// Call a tool with the given arguments
    ///
    /// # Arguments
    ///
    /// * `name` - Tool name (must match one returned from `list_tools`)
    /// * `arguments` - Tool arguments as a JSON value (optional)
    ///
    /// # Errors
    ///
    /// Returns `ToolNotFound` if the tool doesn't exist, or `ToolExecutionError`
    /// if the tool execution fails
    async fn call_tool(&self, name: &str, arguments: Option<Value>) -> McpResult<ToolResult>;

    // === Resource Operations ===

    /// List all available resources
    ///
    /// # Errors
    ///
    /// Returns an error if the server doesn't support resource listing
    async fn list_resources(&self) -> McpResult<Vec<ResourceInfo>>;

    /// Read a resource by URI
    ///
    /// # Arguments
    ///
    /// * `uri` - Resource URI (must match one returned from `list_resources`)
    ///
    /// # Errors
    ///
    /// Returns `ResourceNotFound` if the resource doesn't exist, or
    /// `ResourceReadError` if reading fails
    async fn read_resource(&self, uri: &str) -> McpResult<ResourceContents>;

    // === Prompt Operations ===

    /// List all available prompts
    ///
    /// # Errors
    ///
    /// Returns an error if the server doesn't support prompt listing
    async fn list_prompts(&self) -> McpResult<Vec<PromptInfo>>;

    /// Get a prompt by name with optional arguments
    ///
    /// # Arguments
    ///
    /// * `name` - Prompt name (must match one returned from `list_prompts`)
    /// * `arguments` - Prompt arguments as a map (optional)
    ///
    /// # Errors
    ///
    /// Returns `PromptNotFound` if the prompt doesn't exist, or
    /// `PromptExecutionError` if prompt execution fails
    async fn get_prompt(
        &self,
        name: &str,
        arguments: Option<HashMap<String, String>>,
    ) -> McpResult<PromptResult>;

    // === Capability Queries ===

    /// Check if the server supports tools
    fn supports_tools(&self) -> bool;

    /// Check if the server supports resources
    fn supports_resources(&self) -> bool;

    /// Check if the server supports prompts
    fn supports_prompts(&self) -> bool;

    /// Check if the server supports resource subscriptions
    fn supports_resource_subscriptions(&self) -> bool;

    /// Get the underlying server info from last successful initialization
    ///
    /// Returns None if `initialize()` hasn't been called yet
    fn server_info(&self) -> Option<ServerInfo>;

    /// Check if the client is connected
    ///
    /// Note: This is a best-effort check. A true result doesn't guarantee
    /// that the next operation will succeed.
    fn is_connected(&self) -> bool;
}

/// Type alias for an Arc-wrapped MCP client
///
/// Uses Arc for cheap cloning and shared ownership.
pub type BoxedMcpClient = std::sync::Arc<dyn McpClient>;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_server_info_creation() {
        let info = ServerInfo {
            name: "test-server".to_string(),
            version: "1.0.0".to_string(),
        };
        assert_eq!(info.name, "test-server");
        assert_eq!(info.version, "1.0.0");
    }

    #[test]
    fn test_tool_info_creation() {
        let tool = ToolInfo {
            name: "my_tool".to_string(),
            description: Some("A test tool".to_string()),
            input_schema: Some(json!({"type": "object"})),
        };
        assert_eq!(tool.name, "my_tool");
        assert!(tool.description.is_some());
        assert!(tool.input_schema.is_some());
    }

    #[test]
    fn test_tool_result_creation() {
        let result = ToolResult {
            content: json!({"output": "test"}),
            is_error: false,
        };
        assert!(!result.is_error);
        assert_eq!(result.content.get("output").unwrap(), "test");
    }
}
