//! MCP Bridge - Aggregate multiple MCP clients into a single interface
//!
//! The bridge allows you to combine tools, resources, and prompts from multiple
//! MCP servers (potentially from different SDKs) and present them through a
//! single unified McpClient interface.
//!
//! ## Use Cases
//!
//! - **Multi-server aggregation**: Combine tools from multiple specialized MCP servers
//! - **Mixed SDK deployment**: Use both TurboMCP and Official SDK clients simultaneously
//! - **Capability composition**: Build complex workflows across multiple services
//!
//! ## Example
//!
//! ```ignore
//! use turboclaude_mcp::{McpBridge, McpClient};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create bridge with multiple clients
//!     let bridge = McpBridge::builder()
//!         .add_client("search", search_client)
//!         .add_client("database", db_client)
//!         .add_client("api", api_client)
//!         .build();
//!
//!     // Initialize all underlying clients
//!     bridge.initialize().await.unwrap();
//!
//!     // List tools from all clients (with prefixes)
//!     let all_tools = bridge.list_tools().await.unwrap();
//!     // Returns: ["search::web_search", "database::query", "api::fetch", ...]
//!
//!     // Call tool (bridge routes to correct client)
//!     let result = bridge.call_tool("search::web_search", Some(json!({"q": "rust"}))).await.unwrap();
//! }
//! ```

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{McpError, McpResult};
use crate::trait_::{
    BoxedMcpClient, McpClient, PromptInfo, PromptResult, ResourceContents, ResourceInfo,
    ServerInfo, ToolInfo, ToolResult,
};

/// MCP Bridge - Aggregates multiple MCP clients into a single interface
///
/// Tools, resources, and prompts from each client are exposed with a namespace
/// prefix (e.g., "client_name::tool_name") to avoid conflicts.
#[derive(Clone)]
pub struct McpBridge {
    clients: Arc<HashMap<String, BoxedMcpClient>>,
    separator: String,
}

impl McpBridge {
    /// Create a new bridge builder
    pub fn builder() -> McpBridgeBuilder {
        McpBridgeBuilder::new()
    }

    /// Create a bridge from a map of clients
    pub fn new(clients: HashMap<String, BoxedMcpClient>) -> Self {
        Self {
            clients: Arc::new(clients),
            separator: "::".to_string(),
        }
    }

    /// Parse a namespaced identifier (e.g., "client::tool") into (client, name)
    fn parse_identifier(&self, identifier: &str) -> McpResult<(String, String)> {
        let parts: Vec<&str> = identifier.splitn(2, &self.separator).collect();
        if parts.len() == 2 {
            Ok((parts[0].to_string(), parts[1].to_string()))
        } else {
            Err(McpError::InvalidInput(format!(
                "Identifier '{}' must be namespaced as 'client{}name'",
                identifier, self.separator
            )))
        }
    }

    /// Get a client by name
    fn get_client(&self, name: &str) -> McpResult<&BoxedMcpClient> {
        self.clients.get(name).ok_or_else(|| {
            McpError::AdapterNotFound(format!("No client named '{}' in bridge", name))
        })
    }

    /// Create a namespaced identifier
    fn namespace(&self, client_name: &str, item_name: &str) -> String {
        format!("{}{}{}", client_name, self.separator, item_name)
    }
}

#[async_trait]
impl McpClient for McpBridge {
    async fn initialize(&self) -> McpResult<ServerInfo> {
        // Initialize all underlying clients
        let mut errors = Vec::new();
        for (name, client) in self.clients.iter() {
            if let Err(e) = client.initialize().await {
                errors.push(format!("Client '{}': {}", name, e));
            }
        }

        if !errors.is_empty() {
            return Err(McpError::init(format!(
                "Failed to initialize {} client(s): {}",
                errors.len(),
                errors.join("; ")
            )));
        }

        Ok(ServerInfo {
            name: "mcp-bridge".to_string(),
            version: format!("{} clients", self.clients.len()),
        })
    }

    async fn close(&self) -> McpResult<()> {
        // Close all underlying clients
        let mut errors = Vec::new();
        for (name, client) in self.clients.iter() {
            if let Err(e) = client.close().await {
                errors.push(format!("Client '{}': {}", name, e));
            }
        }

        if !errors.is_empty() {
            return Err(McpError::ProtocolError(format!(
                "Failed to close {} client(s): {}",
                errors.len(),
                errors.join("; ")
            )));
        }

        Ok(())
    }

    async fn list_tools(&self) -> McpResult<Vec<ToolInfo>> {
        let mut all_tools = Vec::new();

        for (client_name, client) in self.clients.iter() {
            match client.list_tools().await {
                Ok(tools) => {
                    for tool in tools {
                        all_tools.push(ToolInfo {
                            name: self.namespace(client_name, &tool.name),
                            description: tool.description,
                            input_schema: tool.input_schema,
                        });
                    }
                }
                Err(e) => {
                    // Log error but continue with other clients
                    tracing::warn!("Failed to list tools from client '{}': {}", client_name, e);
                }
            }
        }

        Ok(all_tools)
    }

    async fn call_tool(&self, name: &str, arguments: Option<Value>) -> McpResult<ToolResult> {
        let (client_name, tool_name) = self.parse_identifier(name)?;
        let client = self.get_client(&client_name)?;
        client.call_tool(&tool_name, arguments).await
    }

    async fn list_resources(&self) -> McpResult<Vec<ResourceInfo>> {
        let mut all_resources = Vec::new();

        for (client_name, client) in self.clients.iter() {
            match client.list_resources().await {
                Ok(resources) => {
                    for resource in resources {
                        all_resources.push(ResourceInfo {
                            uri: self.namespace(client_name, &resource.uri),
                            name: resource.name,
                            description: resource.description,
                            read_only: resource.read_only,
                        });
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to list resources from client '{}': {}",
                        client_name,
                        e
                    );
                }
            }
        }

        Ok(all_resources)
    }

    async fn read_resource(&self, uri: &str) -> McpResult<ResourceContents> {
        let (client_name, resource_uri) = self.parse_identifier(uri)?;
        let client = self.get_client(&client_name)?;
        client.read_resource(&resource_uri).await
    }

    async fn list_prompts(&self) -> McpResult<Vec<PromptInfo>> {
        let mut all_prompts = Vec::new();

        for (client_name, client) in self.clients.iter() {
            match client.list_prompts().await {
                Ok(prompts) => {
                    for prompt in prompts {
                        all_prompts.push(PromptInfo {
                            name: self.namespace(client_name, &prompt.name),
                            description: prompt.description,
                            arguments: prompt.arguments,
                        });
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to list prompts from client '{}': {}",
                        client_name,
                        e
                    );
                }
            }
        }

        Ok(all_prompts)
    }

    async fn get_prompt(
        &self,
        name: &str,
        arguments: Option<HashMap<String, String>>,
    ) -> McpResult<PromptResult> {
        let (client_name, prompt_name) = self.parse_identifier(name)?;
        let client = self.get_client(&client_name)?;
        client.get_prompt(&prompt_name, arguments).await
    }

    fn supports_tools(&self) -> bool {
        self.clients.values().any(|c| c.supports_tools())
    }

    fn supports_resources(&self) -> bool {
        self.clients.values().any(|c| c.supports_resources())
    }

    fn supports_prompts(&self) -> bool {
        self.clients.values().any(|c| c.supports_prompts())
    }

    fn supports_resource_subscriptions(&self) -> bool {
        self.clients
            .values()
            .any(|c| c.supports_resource_subscriptions())
    }

    fn server_info(&self) -> Option<ServerInfo> {
        Some(ServerInfo {
            name: "mcp-bridge".to_string(),
            version: format!("{} clients", self.clients.len()),
        })
    }

    fn is_connected(&self) -> bool {
        // Bridge is connected if at least one client is connected
        self.clients.values().any(|c| c.is_connected())
    }
}

/// Builder for creating an MCP bridge
pub struct McpBridgeBuilder {
    clients: HashMap<String, BoxedMcpClient>,
    separator: String,
}

impl McpBridgeBuilder {
    /// Create a new bridge builder
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            separator: "::".to_string(),
        }
    }

    /// Add a client to the bridge with a namespace identifier
    ///
    /// # Arguments
    ///
    /// * `name` - Namespace prefix for this client (e.g., "search", "database")
    /// * `client` - The MCP client to add
    ///
    /// # Example
    ///
    /// ```ignore
    /// let bridge = McpBridge::builder()
    ///     .add_client("search", search_client)
    ///     .add_client("db", database_client)
    ///     .build();
    /// ```
    pub fn add_client(mut self, name: impl Into<String>, client: BoxedMcpClient) -> Self {
        self.clients.insert(name.into(), client);
        self
    }

    /// Set the separator used for namespacing (default: "::")
    ///
    /// # Example
    ///
    /// ```ignore
    /// let bridge = McpBridge::builder()
    ///     .separator(".")  // Use "client.tool" instead of "client::tool"
    ///     .add_client("search", client)
    ///     .build();
    /// ```
    pub fn separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }

    /// Build the bridge
    ///
    /// # Panics
    ///
    /// Panics if no clients have been added
    pub fn build(self) -> McpBridge {
        assert!(
            !self.clients.is_empty(),
            "Bridge must have at least one client"
        );

        McpBridge {
            clients: Arc::new(self.clients),
            separator: self.separator,
        }
    }

    /// Try to build the bridge
    ///
    /// Returns an error if no clients have been added
    pub fn try_build(self) -> Result<McpBridge, String> {
        if self.clients.is_empty() {
            return Err("Bridge must have at least one client".to_string());
        }

        Ok(McpBridge {
            clients: Arc::new(self.clients),
            separator: self.separator,
        })
    }
}

impl Default for McpBridgeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::OfficialSdkStub;

    #[test]
    fn test_bridge_builder() {
        let client1 = Arc::new(OfficialSdkStub::new());
        let client2 = Arc::new(OfficialSdkStub::new());

        let bridge = McpBridge::builder()
            .add_client("client1", client1)
            .add_client("client2", client2)
            .build();

        assert_eq!(bridge.clients.len(), 2);
        assert_eq!(bridge.separator, "::");
    }

    #[test]
    fn test_bridge_custom_separator() {
        let client = Arc::new(OfficialSdkStub::new());

        let bridge = McpBridge::builder()
            .separator(".")
            .add_client("test", client)
            .build();

        assert_eq!(bridge.separator, ".");
    }

    #[test]
    #[should_panic(expected = "Bridge must have at least one client")]
    fn test_bridge_empty_panics() {
        McpBridge::builder().build();
    }

    #[test]
    fn test_bridge_try_build_empty() {
        let result = McpBridge::builder().try_build();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_identifier() {
        let client = Arc::new(OfficialSdkStub::new());
        let bridge = McpBridge::builder().add_client("test", client).build();

        let (client_name, item_name) = bridge.parse_identifier("test::tool").unwrap();
        assert_eq!(client_name, "test");
        assert_eq!(item_name, "tool");
    }

    #[test]
    fn test_parse_identifier_invalid() {
        let client = Arc::new(OfficialSdkStub::new());
        let bridge = McpBridge::builder().add_client("test", client).build();

        let result = bridge.parse_identifier("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_namespace() {
        let client = Arc::new(OfficialSdkStub::new());
        let bridge = McpBridge::builder().add_client("test", client).build();

        let namespaced = bridge.namespace("client", "tool");
        assert_eq!(namespaced, "client::tool");
    }

    #[tokio::test]
    async fn test_bridge_supports_capabilities() {
        let client = Arc::new(OfficialSdkStub::new());
        let bridge = McpBridge::builder()
            .add_client("test", client.clone())
            .build();

        // OfficialSdkStub doesn't support any capabilities (it's a test stub)
        // To test real capabilities, users would need to create a real OfficialSdkAdapter with a peer
        assert!(!bridge.supports_tools());
        assert!(!bridge.supports_resources());
        assert!(!bridge.supports_prompts());
        assert!(!bridge.supports_resource_subscriptions());
    }

    #[test]
    fn test_bridge_server_info() {
        let client1 = Arc::new(OfficialSdkStub::new());
        let client2 = Arc::new(OfficialSdkStub::new());

        let bridge = McpBridge::builder()
            .add_client("c1", client1)
            .add_client("c2", client2)
            .build();

        let info = bridge.server_info().unwrap();
        assert_eq!(info.name, "mcp-bridge");
        assert_eq!(info.version, "2 clients");
    }
}
