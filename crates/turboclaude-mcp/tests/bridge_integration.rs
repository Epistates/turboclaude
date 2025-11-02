//! Integration tests for MCP Bridge
//!
//! Tests the bridge's ability to aggregate multiple MCP clients and expose
//! their capabilities through a single unified interface.

use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use turboclaude_mcp::{
    McpBridge, McpClient, McpError, McpResult, MessageContent, PromptInfo, PromptResult,
    ResourceContents, ResourceInfo, ServerInfo, ToolInfo, ToolResult,
};

// ========================================
// Mock Clients for Testing
// ========================================

/// Mock search service client
#[derive(Debug, Clone)]
struct SearchServiceClient {
    server_info: Arc<Mutex<Option<ServerInfo>>>,
    is_connected: Arc<Mutex<bool>>,
}

impl SearchServiceClient {
    fn new() -> Self {
        Self {
            server_info: Arc::new(Mutex::new(None)),
            is_connected: Arc::new(Mutex::new(false)),
        }
    }
}

#[async_trait]
impl McpClient for SearchServiceClient {
    async fn initialize(&self) -> McpResult<ServerInfo> {
        let info = ServerInfo {
            name: "search-service".to_string(),
            version: "1.0.0".to_string(),
        };
        *self.server_info.lock().await = Some(info.clone());
        *self.is_connected.lock().await = true;
        Ok(info)
    }

    async fn close(&self) -> McpResult<()> {
        *self.is_connected.lock().await = false;
        Ok(())
    }

    async fn list_tools(&self) -> McpResult<Vec<ToolInfo>> {
        Ok(vec![
            ToolInfo {
                name: "web_search".to_string(),
                description: Some("Search the web".to_string()),
                input_schema: Some(
                    json!({"type": "object", "properties": {"query": {"type": "string"}}}),
                ),
            },
            ToolInfo {
                name: "image_search".to_string(),
                description: Some("Search for images".to_string()),
                input_schema: None,
            },
        ])
    }

    async fn call_tool(
        &self,
        name: &str,
        arguments: Option<serde_json::Value>,
    ) -> McpResult<ToolResult> {
        match name {
            "web_search" => Ok(ToolResult {
                content: json!({
                    "service": "search",
                    "tool": "web_search",
                    "query": arguments.and_then(|v| v.get("query").cloned()).unwrap_or(json!("default")),
                    "results": ["result1", "result2"]
                }),
                is_error: false,
            }),
            "image_search" => Ok(ToolResult {
                content: json!({"service": "search", "tool": "image_search", "images": []}),
                is_error: false,
            }),
            _ => Err(McpError::ToolNotFound(format!("Tool '{}' not found", name))),
        }
    }

    async fn list_resources(&self) -> McpResult<Vec<ResourceInfo>> {
        Ok(vec![ResourceInfo {
            uri: "search://index".to_string(),
            name: "search_index".to_string(),
            description: Some("Search index metadata".to_string()),
            read_only: true,
        }])
    }

    async fn read_resource(&self, uri: &str) -> McpResult<ResourceContents> {
        if uri == "search://index" {
            Ok(ResourceContents {
                uri: uri.to_string(),
                mime_type: Some("application/json".to_string()),
                text: json!({"indexed_pages": 1000000}).to_string(),
            })
        } else {
            Err(McpError::ResourceNotFound(format!(
                "Resource '{}' not found",
                uri
            )))
        }
    }

    async fn list_prompts(&self) -> McpResult<Vec<PromptInfo>> {
        Ok(vec![PromptInfo {
            name: "search_template".to_string(),
            description: Some("Template for search queries".to_string()),
            arguments: None,
        }])
    }

    async fn get_prompt(
        &self,
        name: &str,
        _arguments: Option<HashMap<String, String>>,
    ) -> McpResult<PromptResult> {
        if name == "search_template" {
            Ok(PromptResult {
                messages: vec![MessageContent {
                    role: "user".to_string(),
                    text: "Search for information about: [TOPIC]".to_string(),
                }],
            })
        } else {
            Err(McpError::PromptNotFound(format!(
                "Prompt '{}' not found",
                name
            )))
        }
    }

    fn supports_tools(&self) -> bool {
        true
    }

    fn supports_resources(&self) -> bool {
        true
    }

    fn supports_prompts(&self) -> bool {
        true
    }

    fn supports_resource_subscriptions(&self) -> bool {
        false
    }

    fn server_info(&self) -> Option<ServerInfo> {
        self.server_info
            .try_lock()
            .expect("Lock should not be contended")
            .clone()
    }

    fn is_connected(&self) -> bool {
        *self
            .is_connected
            .try_lock()
            .expect("Lock should not be contended")
    }
}

/// Mock database service client
#[derive(Debug, Clone)]
struct DatabaseServiceClient {
    server_info: Arc<Mutex<Option<ServerInfo>>>,
    is_connected: Arc<Mutex<bool>>,
}

impl DatabaseServiceClient {
    fn new() -> Self {
        Self {
            server_info: Arc::new(Mutex::new(None)),
            is_connected: Arc::new(Mutex::new(false)),
        }
    }
}

#[async_trait]
impl McpClient for DatabaseServiceClient {
    async fn initialize(&self) -> McpResult<ServerInfo> {
        let info = ServerInfo {
            name: "database-service".to_string(),
            version: "2.0.0".to_string(),
        };
        *self.server_info.lock().await = Some(info.clone());
        *self.is_connected.lock().await = true;
        Ok(info)
    }

    async fn close(&self) -> McpResult<()> {
        *self.is_connected.lock().await = false;
        Ok(())
    }

    async fn list_tools(&self) -> McpResult<Vec<ToolInfo>> {
        Ok(vec![
            ToolInfo {
                name: "query".to_string(),
                description: Some("Execute SQL query".to_string()),
                input_schema: Some(
                    json!({"type": "object", "properties": {"sql": {"type": "string"}}}),
                ),
            },
            ToolInfo {
                name: "insert".to_string(),
                description: Some("Insert data".to_string()),
                input_schema: None,
            },
        ])
    }

    async fn call_tool(
        &self,
        name: &str,
        arguments: Option<serde_json::Value>,
    ) -> McpResult<ToolResult> {
        match name {
            "query" => Ok(ToolResult {
                content: json!({
                    "service": "database",
                    "tool": "query",
                    "sql": arguments.and_then(|v| v.get("sql").cloned()).unwrap_or(json!("SELECT 1")),
                    "rows": [{"id": 1}]
                }),
                is_error: false,
            }),
            "insert" => Ok(ToolResult {
                content: json!({"service": "database", "tool": "insert", "inserted": true}),
                is_error: false,
            }),
            _ => Err(McpError::ToolNotFound(format!("Tool '{}' not found", name))),
        }
    }

    async fn list_resources(&self) -> McpResult<Vec<ResourceInfo>> {
        Ok(vec![ResourceInfo {
            uri: "db://schema".to_string(),
            name: "db_schema".to_string(),
            description: Some("Database schema".to_string()),
            read_only: true,
        }])
    }

    async fn read_resource(&self, uri: &str) -> McpResult<ResourceContents> {
        if uri == "db://schema" {
            Ok(ResourceContents {
                uri: uri.to_string(),
                mime_type: Some("application/sql".to_string()),
                text: "CREATE TABLE users (id INT, name VARCHAR(100));".to_string(),
            })
        } else {
            Err(McpError::ResourceNotFound(format!(
                "Resource '{}' not found",
                uri
            )))
        }
    }

    async fn list_prompts(&self) -> McpResult<Vec<PromptInfo>> {
        Ok(vec![PromptInfo {
            name: "query_builder".to_string(),
            description: Some("Help build SQL queries".to_string()),
            arguments: None,
        }])
    }

    async fn get_prompt(
        &self,
        name: &str,
        _arguments: Option<HashMap<String, String>>,
    ) -> McpResult<PromptResult> {
        if name == "query_builder" {
            Ok(PromptResult {
                messages: vec![MessageContent {
                    role: "assistant".to_string(),
                    text: "I'll help you build a SQL query for [TABLE]".to_string(),
                }],
            })
        } else {
            Err(McpError::PromptNotFound(format!(
                "Prompt '{}' not found",
                name
            )))
        }
    }

    fn supports_tools(&self) -> bool {
        true
    }

    fn supports_resources(&self) -> bool {
        true
    }

    fn supports_prompts(&self) -> bool {
        true
    }

    fn supports_resource_subscriptions(&self) -> bool {
        true
    }

    fn server_info(&self) -> Option<ServerInfo> {
        self.server_info
            .try_lock()
            .expect("Lock should not be contended")
            .clone()
    }

    fn is_connected(&self) -> bool {
        *self
            .is_connected
            .try_lock()
            .expect("Lock should not be contended")
    }
}

// ========================================
// Bridge Integration Tests
// ========================================

#[tokio::test]
async fn test_bridge_initialization() {
    let search = Arc::new(SearchServiceClient::new());
    let database = Arc::new(DatabaseServiceClient::new());

    let bridge = McpBridge::builder()
        .add_client("search", search.clone())
        .add_client("database", database.clone())
        .build();

    let info = bridge.initialize().await.unwrap();
    assert_eq!(info.name, "mcp-bridge");
    assert_eq!(info.version, "2 clients");

    assert!(bridge.is_connected());
}

#[tokio::test]
async fn test_bridge_aggregates_tools() {
    let search = Arc::new(SearchServiceClient::new());
    let database = Arc::new(DatabaseServiceClient::new());

    let bridge = McpBridge::builder()
        .add_client("search", search)
        .add_client("database", database)
        .build();

    bridge.initialize().await.unwrap();

    let tools = bridge.list_tools().await.unwrap();
    assert_eq!(tools.len(), 4); // 2 from search + 2 from database

    let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
    assert!(tool_names.contains(&"search::web_search".to_string()));
    assert!(tool_names.contains(&"search::image_search".to_string()));
    assert!(tool_names.contains(&"database::query".to_string()));
    assert!(tool_names.contains(&"database::insert".to_string()));
}

#[tokio::test]
async fn test_bridge_routes_tool_calls() {
    let search = Arc::new(SearchServiceClient::new());
    let database = Arc::new(DatabaseServiceClient::new());

    let bridge = McpBridge::builder()
        .add_client("search", search)
        .add_client("database", database)
        .build();

    bridge.initialize().await.unwrap();

    // Call search tool
    let search_result = bridge
        .call_tool("search::web_search", Some(json!({"query": "rust"})))
        .await
        .unwrap();
    assert_eq!(search_result.content["service"], "search");
    assert_eq!(search_result.content["query"], "rust");

    // Call database tool
    let db_result = bridge
        .call_tool(
            "database::query",
            Some(json!({"sql": "SELECT * FROM users"})),
        )
        .await
        .unwrap();
    assert_eq!(db_result.content["service"], "database");
    assert_eq!(db_result.content["sql"], "SELECT * FROM users");
}

#[tokio::test]
async fn test_bridge_aggregates_resources() {
    let search = Arc::new(SearchServiceClient::new());
    let database = Arc::new(DatabaseServiceClient::new());

    let bridge = McpBridge::builder()
        .add_client("search", search)
        .add_client("database", database)
        .build();

    bridge.initialize().await.unwrap();

    let resources = bridge.list_resources().await.unwrap();
    assert_eq!(resources.len(), 2); // 1 from search + 1 from database

    let resource_uris: Vec<String> = resources.iter().map(|r| r.uri.clone()).collect();
    assert!(resource_uris.contains(&"search::search://index".to_string()));
    assert!(resource_uris.contains(&"database::db://schema".to_string()));
}

#[tokio::test]
async fn test_bridge_routes_resource_reads() {
    let search = Arc::new(SearchServiceClient::new());
    let database = Arc::new(DatabaseServiceClient::new());

    let bridge = McpBridge::builder()
        .add_client("search", search)
        .add_client("database", database)
        .build();

    bridge.initialize().await.unwrap();

    // Read search resource
    let search_content = bridge
        .read_resource("search::search://index")
        .await
        .unwrap();
    assert!(search_content.text.contains("indexed_pages"));

    // Read database resource
    let db_content = bridge.read_resource("database::db://schema").await.unwrap();
    assert!(db_content.text.contains("CREATE TABLE"));
}

#[tokio::test]
async fn test_bridge_aggregates_prompts() {
    let search = Arc::new(SearchServiceClient::new());
    let database = Arc::new(DatabaseServiceClient::new());

    let bridge = McpBridge::builder()
        .add_client("search", search)
        .add_client("database", database)
        .build();

    bridge.initialize().await.unwrap();

    let prompts = bridge.list_prompts().await.unwrap();
    assert_eq!(prompts.len(), 2); // 1 from search + 1 from database

    let prompt_names: Vec<String> = prompts.iter().map(|p| p.name.clone()).collect();
    assert!(prompt_names.contains(&"search::search_template".to_string()));
    assert!(prompt_names.contains(&"database::query_builder".to_string()));
}

#[tokio::test]
async fn test_bridge_routes_prompt_requests() {
    let search = Arc::new(SearchServiceClient::new());
    let database = Arc::new(DatabaseServiceClient::new());

    let bridge = McpBridge::builder()
        .add_client("search", search)
        .add_client("database", database)
        .build();

    bridge.initialize().await.unwrap();

    // Get search prompt
    let search_prompt = bridge
        .get_prompt("search::search_template", None)
        .await
        .unwrap();
    assert_eq!(search_prompt.messages[0].role, "user");

    // Get database prompt
    let db_prompt = bridge
        .get_prompt("database::query_builder", None)
        .await
        .unwrap();
    assert_eq!(db_prompt.messages[0].role, "assistant");
}

#[tokio::test]
async fn test_bridge_capabilities() {
    let search = Arc::new(SearchServiceClient::new());
    let database = Arc::new(DatabaseServiceClient::new());

    let bridge = McpBridge::builder()
        .add_client("search", search)
        .add_client("database", database)
        .build();

    // Bridge supports capabilities if ANY client supports them
    assert!(bridge.supports_tools());
    assert!(bridge.supports_resources());
    assert!(bridge.supports_prompts());
    assert!(bridge.supports_resource_subscriptions()); // database supports this
}

#[tokio::test]
async fn test_bridge_error_handling_invalid_identifier() {
    let search = Arc::new(SearchServiceClient::new());

    let bridge = McpBridge::builder().add_client("search", search).build();

    bridge.initialize().await.unwrap();

    // Try to call tool without namespace
    let result = bridge.call_tool("invalid_tool", None).await;
    assert!(result.is_err());
    if let Err(McpError::InvalidInput(msg)) = result {
        assert!(msg.contains("must be namespaced"));
    } else {
        panic!("Expected InvalidInput error");
    }
}

#[tokio::test]
async fn test_bridge_error_handling_unknown_client() {
    let search = Arc::new(SearchServiceClient::new());

    let bridge = McpBridge::builder().add_client("search", search).build();

    bridge.initialize().await.unwrap();

    // Try to call tool for nonexistent client
    let result = bridge.call_tool("unknown::tool", None).await;
    assert!(result.is_err());
    if let Err(McpError::AdapterNotFound(msg)) = result {
        assert!(msg.contains("unknown"));
    } else {
        panic!("Expected AdapterNotFound error");
    }
}

#[tokio::test]
async fn test_bridge_error_handling_unknown_tool() {
    let search = Arc::new(SearchServiceClient::new());

    let bridge = McpBridge::builder().add_client("search", search).build();

    bridge.initialize().await.unwrap();

    // Try to call nonexistent tool
    let result = bridge.call_tool("search::unknown_tool", None).await;
    assert!(result.is_err());
    if let Err(McpError::ToolNotFound(_)) = result {
        // Expected
    } else {
        panic!("Expected ToolNotFound error");
    }
}

#[tokio::test]
async fn test_bridge_custom_separator() {
    let search = Arc::new(SearchServiceClient::new());

    let bridge = McpBridge::builder()
        .separator(".")
        .add_client("search", search)
        .build();

    bridge.initialize().await.unwrap();

    let tools = bridge.list_tools().await.unwrap();
    assert!(tools[0].name.contains("search."));
}

#[tokio::test]
async fn test_bridge_close() {
    let search = Arc::new(SearchServiceClient::new());
    let database = Arc::new(DatabaseServiceClient::new());

    let bridge = McpBridge::builder()
        .add_client("search", search.clone())
        .add_client("database", database.clone())
        .build();

    bridge.initialize().await.unwrap();
    assert!(bridge.is_connected());

    bridge.close().await.unwrap();
    assert!(!bridge.is_connected());
}

#[tokio::test]
async fn test_bridge_concurrent_operations() {
    let search = Arc::new(SearchServiceClient::new());
    let database = Arc::new(DatabaseServiceClient::new());

    let bridge = Arc::new(
        McpBridge::builder()
            .add_client("search", search)
            .add_client("database", database)
            .build(),
    );

    bridge.initialize().await.unwrap();

    // Spawn concurrent operations
    let bridge1 = bridge.clone();
    let handle1 = tokio::spawn(async move {
        bridge1
            .call_tool("search::web_search", Some(json!({"query": "test1"})))
            .await
    });

    let bridge2 = bridge.clone();
    let handle2 = tokio::spawn(async move {
        bridge2
            .call_tool("database::query", Some(json!({"sql": "SELECT 1"})))
            .await
    });

    // Wait for both to complete
    let result1 = handle1.await.unwrap().unwrap();
    let result2 = handle2.await.unwrap().unwrap();

    assert_eq!(result1.content["service"], "search");
    assert_eq!(result2.content["service"], "database");
}

#[tokio::test]
async fn test_bridge_mixed_workflow() {
    let search = Arc::new(SearchServiceClient::new());
    let database = Arc::new(DatabaseServiceClient::new());

    let bridge = McpBridge::builder()
        .add_client("search", search)
        .add_client("database", database)
        .build();

    bridge.initialize().await.unwrap();

    // Step 1: Search for data
    let search_result = bridge
        .call_tool("search::web_search", Some(json!({"query": "users"})))
        .await
        .unwrap();
    assert!(!search_result.is_error);

    // Step 2: Query database based on search results
    let db_result = bridge
        .call_tool(
            "database::query",
            Some(json!({"sql": "SELECT * FROM users"})),
        )
        .await
        .unwrap();
    assert!(!db_result.is_error);

    // Step 3: Read schema resource
    let schema = bridge.read_resource("database::db://schema").await.unwrap();
    assert!(schema.text.contains("users"));

    // All operations should succeed
    assert_eq!(search_result.content["service"], "search");
    assert_eq!(db_result.content["service"], "database");
}
