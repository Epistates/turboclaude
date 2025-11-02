//! Registry for managing multiple MCP clients
//!
//! Allows storing and routing between multiple MCP clients from different SDKs.

use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::error::{McpError, McpResult};
use crate::trait_::{BoxedMcpClient, ToolResult};

/// Registry for managing multiple MCP clients
///
/// Enables storing and routing between multiple MCP clients, supporting
/// mixed-SDK deployments and dynamic client management.
#[derive(Clone)]
pub struct McpClientRegistry {
    clients: Arc<Mutex<HashMap<String, BoxedMcpClient>>>,
}

impl McpClientRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a client with a name
    pub fn register(&self, name: &str, client: BoxedMcpClient) -> McpResult<()> {
        self.clients
            .lock()
            .unwrap()
            .insert(name.to_string(), client);
        Ok(())
    }

    /// Unregister a client by name
    pub fn unregister(&self, name: &str) -> McpResult<Option<BoxedMcpClient>> {
        Ok(self.clients.lock().unwrap().remove(name))
    }

    /// Get a registered client by name
    pub fn get(&self, name: &str) -> McpResult<Option<BoxedMcpClient>> {
        Ok(self.clients.lock().unwrap().get(name).cloned())
    }

    /// List all registered client names
    pub fn list_names(&self) -> McpResult<Vec<String>> {
        Ok(self.clients.lock().unwrap().keys().cloned().collect())
    }

    /// Call a tool on a registered client
    pub async fn call_tool(
        &self,
        client_name: &str,
        tool_name: &str,
        arguments: Option<Value>,
    ) -> McpResult<ToolResult> {
        // Clone the client Arc to avoid holding the lock across await
        let client = {
            let clients = self.clients.lock().unwrap();
            clients
                .get(client_name)
                .ok_or_else(|| McpError::AdapterNotFound(client_name.to_string()))?
                .clone()
        };

        client.call_tool(tool_name, arguments).await
    }

    /// List tools available on a registered client
    pub async fn list_tools_for(&self, client_name: &str) -> McpResult<Vec<String>> {
        // Clone the client Arc to avoid holding the lock across await
        let client = {
            let clients = self.clients.lock().unwrap();
            clients
                .get(client_name)
                .ok_or_else(|| McpError::AdapterNotFound(client_name.to_string()))?
                .clone()
        };

        let tools = client.list_tools().await?;
        Ok(tools.into_iter().map(|t| t.name).collect())
    }

    /// Get count of registered clients
    pub fn count(&self) -> usize {
        self.clients.lock().unwrap().len()
    }

    /// Clear all registered clients
    pub fn clear(&self) -> McpResult<()> {
        self.clients.lock().unwrap().clear();
        Ok(())
    }
}

impl Default for McpClientRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::OfficialSdkStub;
    use std::sync::Arc;

    #[test]
    fn test_registry_creation() {
        let registry = McpClientRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let registry = McpClientRegistry::new();
        let client = Arc::new(OfficialSdkStub::new()) as BoxedMcpClient;
        registry.register("client1", client).unwrap();
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_registry_get() {
        let registry = McpClientRegistry::new();
        let client = Arc::new(OfficialSdkStub::new()) as BoxedMcpClient;
        registry.register("client1", client).unwrap();

        let retrieved = registry.get("client1").unwrap();
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_registry_get_nonexistent() {
        let registry = McpClientRegistry::new();
        let retrieved = registry.get("nonexistent").unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_registry_list_names() {
        let registry = McpClientRegistry::new();
        let client1 = Arc::new(OfficialSdkStub::new()) as BoxedMcpClient;
        let client2 = Arc::new(OfficialSdkStub::new()) as BoxedMcpClient;

        registry.register("client1", client1).unwrap();
        registry.register("client2", client2).unwrap();

        let names = registry.list_names().unwrap();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"client1".to_string()));
        assert!(names.contains(&"client2".to_string()));
    }

    #[test]
    fn test_registry_unregister() {
        let registry = McpClientRegistry::new();
        let client = Arc::new(OfficialSdkStub::new()) as BoxedMcpClient;
        registry.register("client1", client).unwrap();
        assert_eq!(registry.count(), 1);

        let unregistered = registry.unregister("client1").unwrap();
        assert!(unregistered.is_some());
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_registry_clear() {
        let registry = McpClientRegistry::new();
        let client1 = Arc::new(OfficialSdkStub::new()) as BoxedMcpClient;
        let client2 = Arc::new(OfficialSdkStub::new()) as BoxedMcpClient;

        registry.register("client1", client1).unwrap();
        registry.register("client2", client2).unwrap();
        assert_eq!(registry.count(), 2);

        registry.clear().unwrap();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_registry_default() {
        let registry = McpClientRegistry::default();
        assert_eq!(registry.count(), 0);
    }
}
