//! Cross-SDK integration tests
//!
//! Tests the ability to work with multiple MCP clients from different SDKs,
//! managing them through the registry, and performing cross-SDK operations.

use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use turboclaude_mcp::{
    McpClient, McpClientRegistry, McpError, McpResult, MessageContent, PromptInfo, PromptResult,
    ResourceContents, ResourceInfo, ServerInfo, ToolInfo, ToolResult,
};

/// Mock client simulating TurboMCP SDK
#[derive(Debug, Clone)]
struct TurboMcpMockClient {
    name: String,
    server_info: Arc<Mutex<Option<ServerInfo>>>,
    is_connected: Arc<Mutex<bool>>,
}

impl TurboMcpMockClient {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            server_info: Arc::new(Mutex::new(None)),
            is_connected: Arc::new(Mutex::new(false)),
        }
    }
}

#[async_trait]
impl McpClient for TurboMcpMockClient {
    async fn initialize(&self) -> McpResult<ServerInfo> {
        let info = ServerInfo {
            name: format!("turbomcp-{}", self.name),
            version: "1.0.0-turbomcp".to_string(),
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
                name: "turbomcp_search".to_string(),
                description: Some("TurboMCP search tool".to_string()),
                input_schema: Some(
                    json!({"type": "object", "properties": {"query": {"type": "string"}}}),
                ),
            },
            ToolInfo {
                name: "turbomcp_analyze".to_string(),
                description: Some("TurboMCP analysis tool".to_string()),
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
            "turbomcp_search" => Ok(ToolResult {
                content: json!({
                    "sdk": "turbomcp",
                    "tool": "search",
                    "query": arguments.and_then(|v| v.get("query").cloned()).unwrap_or(json!("default")),
                    "results": ["result1", "result2", "result3"]
                }),
                is_error: false,
            }),
            "turbomcp_analyze" => Ok(ToolResult {
                content: json!({"sdk": "turbomcp", "tool": "analyze", "status": "complete"}),
                is_error: false,
            }),
            _ => Err(McpError::ToolNotFound(format!(
                "Tool '{}' not found in TurboMCP client",
                name
            ))),
        }
    }

    async fn list_resources(&self) -> McpResult<Vec<ResourceInfo>> {
        Ok(vec![ResourceInfo {
            uri: "turbomcp://resource/config".to_string(),
            name: "turbomcp_config".to_string(),
            description: Some("TurboMCP configuration resource".to_string()),
            read_only: true,
        }])
    }

    async fn read_resource(&self, uri: &str) -> McpResult<ResourceContents> {
        if uri == "turbomcp://resource/config" {
            Ok(ResourceContents {
                uri: uri.to_string(),
                mime_type: Some("application/json".to_string()),
                text: json!({"sdk": "turbomcp", "version": "1.0.0"}).to_string(),
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
            name: "turbomcp_template".to_string(),
            description: Some("TurboMCP prompt template".to_string()),
            arguments: None,
        }])
    }

    async fn get_prompt(
        &self,
        name: &str,
        _arguments: Option<HashMap<String, String>>,
    ) -> McpResult<PromptResult> {
        if name == "turbomcp_template" {
            Ok(PromptResult {
                messages: vec![MessageContent {
                    role: "user".to_string(),
                    text: "This is a TurboMCP prompt".to_string(),
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

/// Mock client simulating Official Rust SDK
#[derive(Debug, Clone)]
struct OfficialSdkMockClient {
    name: String,
    server_info: Arc<Mutex<Option<ServerInfo>>>,
    is_connected: Arc<Mutex<bool>>,
}

impl OfficialSdkMockClient {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            server_info: Arc::new(Mutex::new(None)),
            is_connected: Arc::new(Mutex::new(false)),
        }
    }
}

#[async_trait]
impl McpClient for OfficialSdkMockClient {
    async fn initialize(&self) -> McpResult<ServerInfo> {
        let info = ServerInfo {
            name: format!("official-{}", self.name),
            version: "1.0.0-official".to_string(),
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
                name: "official_fetch".to_string(),
                description: Some("Official SDK fetch tool".to_string()),
                input_schema: Some(
                    json!({"type": "object", "properties": {"url": {"type": "string"}}}),
                ),
            },
            ToolInfo {
                name: "official_process".to_string(),
                description: Some("Official SDK processing tool".to_string()),
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
            "official_fetch" => Ok(ToolResult {
                content: json!({
                    "sdk": "official",
                    "tool": "fetch",
                    "url": arguments.and_then(|v| v.get("url").cloned()).unwrap_or(json!("https://example.com")),
                    "data": "fetched content"
                }),
                is_error: false,
            }),
            "official_process" => Ok(ToolResult {
                content: json!({"sdk": "official", "tool": "process", "processed": true}),
                is_error: false,
            }),
            _ => Err(McpError::ToolNotFound(format!(
                "Tool '{}' not found in Official SDK client",
                name
            ))),
        }
    }

    async fn list_resources(&self) -> McpResult<Vec<ResourceInfo>> {
        Ok(vec![ResourceInfo {
            uri: "official://resource/data".to_string(),
            name: "official_data".to_string(),
            description: Some("Official SDK data resource".to_string()),
            read_only: false,
        }])
    }

    async fn read_resource(&self, uri: &str) -> McpResult<ResourceContents> {
        if uri == "official://resource/data" {
            Ok(ResourceContents {
                uri: uri.to_string(),
                mime_type: Some("text/plain".to_string()),
                text: "Official SDK resource data".to_string(),
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
            name: "official_pattern".to_string(),
            description: Some("Official SDK prompt pattern".to_string()),
            arguments: None,
        }])
    }

    async fn get_prompt(
        &self,
        name: &str,
        _arguments: Option<HashMap<String, String>>,
    ) -> McpResult<PromptResult> {
        if name == "official_pattern" {
            Ok(PromptResult {
                messages: vec![MessageContent {
                    role: "assistant".to_string(),
                    text: "This is an Official SDK prompt".to_string(),
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
// Cross-SDK Registry Tests
// ========================================

#[tokio::test]
async fn test_registry_multiple_sdks() {
    let registry = McpClientRegistry::new();

    // Register TurboMCP client
    let turbomcp = Arc::new(TurboMcpMockClient::new("server1"));
    registry.register("turbomcp1", turbomcp.clone()).unwrap();

    // Register Official SDK client
    let official = Arc::new(OfficialSdkMockClient::new("server2"));
    registry.register("official1", official.clone()).unwrap();

    assert_eq!(registry.count(), 2);

    // Verify both clients are retrievable
    let retrieved_turbo = registry.get("turbomcp1").unwrap();
    assert!(retrieved_turbo.is_some());

    let retrieved_official = registry.get("official1").unwrap();
    assert!(retrieved_official.is_some());
}

#[tokio::test]
async fn test_cross_sdk_initialization() {
    let registry = McpClientRegistry::new();

    // Register and initialize TurboMCP client
    let turbomcp = Arc::new(TurboMcpMockClient::new("test"));
    registry.register("turbomcp", turbomcp.clone()).unwrap();
    let turbo_info = turbomcp.initialize().await.unwrap();
    assert_eq!(turbo_info.name, "turbomcp-test");
    assert_eq!(turbo_info.version, "1.0.0-turbomcp");

    // Register and initialize Official SDK client
    let official = Arc::new(OfficialSdkMockClient::new("test"));
    registry.register("official", official.clone()).unwrap();
    let official_info = official.initialize().await.unwrap();
    assert_eq!(official_info.name, "official-test");
    assert_eq!(official_info.version, "1.0.0-official");

    // Both clients should be connected
    assert!(turbomcp.is_connected());
    assert!(official.is_connected());
}

#[tokio::test]
async fn test_cross_sdk_tool_discovery() {
    let registry = McpClientRegistry::new();

    // Setup TurboMCP client
    let turbomcp = Arc::new(TurboMcpMockClient::new("tools"));
    turbomcp.initialize().await.unwrap();
    registry.register("turbomcp", turbomcp.clone()).unwrap();

    // Setup Official SDK client
    let official = Arc::new(OfficialSdkMockClient::new("tools"));
    official.initialize().await.unwrap();
    registry.register("official", official.clone()).unwrap();

    // List tools from TurboMCP
    let turbo_tools = registry.list_tools_for("turbomcp").await.unwrap();
    assert_eq!(turbo_tools.len(), 2);
    assert!(turbo_tools.contains(&"turbomcp_search".to_string()));
    assert!(turbo_tools.contains(&"turbomcp_analyze".to_string()));

    // List tools from Official SDK
    let official_tools = registry.list_tools_for("official").await.unwrap();
    assert_eq!(official_tools.len(), 2);
    assert!(official_tools.contains(&"official_fetch".to_string()));
    assert!(official_tools.contains(&"official_process".to_string()));
}

#[tokio::test]
async fn test_cross_sdk_tool_calling() {
    let registry = McpClientRegistry::new();

    // Setup clients
    let turbomcp = Arc::new(TurboMcpMockClient::new("caller"));
    turbomcp.initialize().await.unwrap();
    registry.register("turbomcp", turbomcp).unwrap();

    let official = Arc::new(OfficialSdkMockClient::new("caller"));
    official.initialize().await.unwrap();
    registry.register("official", official).unwrap();

    // Call TurboMCP tool
    let turbo_result = registry
        .call_tool(
            "turbomcp",
            "turbomcp_search",
            Some(json!({"query": "rust"})),
        )
        .await
        .unwrap();
    assert!(!turbo_result.is_error);
    assert_eq!(turbo_result.content["sdk"], "turbomcp");
    assert_eq!(turbo_result.content["query"], "rust");

    // Call Official SDK tool
    let official_result = registry
        .call_tool(
            "official",
            "official_fetch",
            Some(json!({"url": "https://rust-lang.org"})),
        )
        .await
        .unwrap();
    assert!(!official_result.is_error);
    assert_eq!(official_result.content["sdk"], "official");
    assert_eq!(official_result.content["url"], "https://rust-lang.org");
}

#[tokio::test]
async fn test_cross_sdk_resource_access() {
    // Setup TurboMCP client
    let turbomcp = Arc::new(TurboMcpMockClient::new("resources"));
    turbomcp.initialize().await.unwrap();

    // Setup Official SDK client
    let official = Arc::new(OfficialSdkMockClient::new("resources"));
    official.initialize().await.unwrap();

    // List resources from TurboMCP
    let turbo_resources = turbomcp.list_resources().await.unwrap();
    assert_eq!(turbo_resources.len(), 1);
    assert_eq!(turbo_resources[0].uri, "turbomcp://resource/config");

    // Read TurboMCP resource
    let turbo_content = turbomcp
        .read_resource("turbomcp://resource/config")
        .await
        .unwrap();
    assert!(turbo_content.text.contains("turbomcp"));

    // List resources from Official SDK
    let official_resources = official.list_resources().await.unwrap();
    assert_eq!(official_resources.len(), 1);
    assert_eq!(official_resources[0].uri, "official://resource/data");

    // Read Official SDK resource
    let official_content = official
        .read_resource("official://resource/data")
        .await
        .unwrap();
    assert_eq!(official_content.text, "Official SDK resource data");
}

#[tokio::test]
async fn test_cross_sdk_prompt_handling() {
    // Setup TurboMCP client
    let turbomcp = Arc::new(TurboMcpMockClient::new("prompts"));
    turbomcp.initialize().await.unwrap();

    // Setup Official SDK client
    let official = Arc::new(OfficialSdkMockClient::new("prompts"));
    official.initialize().await.unwrap();

    // List prompts from TurboMCP
    let turbo_prompts = turbomcp.list_prompts().await.unwrap();
    assert_eq!(turbo_prompts.len(), 1);
    assert_eq!(turbo_prompts[0].name, "turbomcp_template");

    // Get TurboMCP prompt
    let turbo_prompt = turbomcp
        .get_prompt("turbomcp_template", None)
        .await
        .unwrap();
    assert_eq!(turbo_prompt.messages.len(), 1);
    assert_eq!(turbo_prompt.messages[0].role, "user");

    // List prompts from Official SDK
    let official_prompts = official.list_prompts().await.unwrap();
    assert_eq!(official_prompts.len(), 1);
    assert_eq!(official_prompts[0].name, "official_pattern");

    // Get Official SDK prompt
    let official_prompt = official.get_prompt("official_pattern", None).await.unwrap();
    assert_eq!(official_prompt.messages.len(), 1);
    assert_eq!(official_prompt.messages[0].role, "assistant");
}

#[tokio::test]
async fn test_cross_sdk_capability_differences() {
    let turbomcp = Arc::new(TurboMcpMockClient::new("caps"));
    let official = Arc::new(OfficialSdkMockClient::new("caps"));

    // TurboMCP doesn't support resource subscriptions
    assert!(!turbomcp.supports_resource_subscriptions());

    // Official SDK does support resource subscriptions
    assert!(official.supports_resource_subscriptions());

    // Both support basic capabilities
    assert!(turbomcp.supports_tools());
    assert!(turbomcp.supports_resources());
    assert!(turbomcp.supports_prompts());

    assert!(official.supports_tools());
    assert!(official.supports_resources());
    assert!(official.supports_prompts());
}

#[tokio::test]
async fn test_registry_error_handling_wrong_client() {
    let registry = McpClientRegistry::new();

    let turbomcp = Arc::new(TurboMcpMockClient::new("test"));
    turbomcp.initialize().await.unwrap();
    registry.register("turbomcp", turbomcp).unwrap();

    // Try to call nonexistent client
    let result = registry.call_tool("nonexistent", "some_tool", None).await;
    assert!(result.is_err());
    if let Err(McpError::AdapterNotFound(name)) = result {
        assert_eq!(name, "nonexistent");
    } else {
        panic!("Expected AdapterNotFound error");
    }
}

#[tokio::test]
async fn test_registry_error_handling_wrong_tool() {
    let registry = McpClientRegistry::new();

    let turbomcp = Arc::new(TurboMcpMockClient::new("test"));
    turbomcp.initialize().await.unwrap();
    registry.register("turbomcp", turbomcp).unwrap();

    // Try to call nonexistent tool
    let result = registry
        .call_tool("turbomcp", "nonexistent_tool", None)
        .await;
    assert!(result.is_err());
    if let Err(McpError::ToolNotFound(_)) = result {
        // Expected
    } else {
        panic!("Expected ToolNotFound error");
    }
}

#[tokio::test]
async fn test_concurrent_cross_sdk_operations() {
    let registry = Arc::new(McpClientRegistry::new());

    // Setup multiple clients
    let turbomcp1 = Arc::new(TurboMcpMockClient::new("concurrent1"));
    turbomcp1.initialize().await.unwrap();
    registry.register("turbomcp1", turbomcp1).unwrap();

    let turbomcp2 = Arc::new(TurboMcpMockClient::new("concurrent2"));
    turbomcp2.initialize().await.unwrap();
    registry.register("turbomcp2", turbomcp2).unwrap();

    let official = Arc::new(OfficialSdkMockClient::new("concurrent"));
    official.initialize().await.unwrap();
    registry.register("official", official).unwrap();

    // Spawn concurrent tool calls
    let reg1 = registry.clone();
    let handle1 = tokio::spawn(async move {
        reg1.call_tool(
            "turbomcp1",
            "turbomcp_search",
            Some(json!({"query": "test1"})),
        )
        .await
    });

    let reg2 = registry.clone();
    let handle2 =
        tokio::spawn(async move { reg2.call_tool("turbomcp2", "turbomcp_analyze", None).await });

    let reg3 = registry.clone();
    let handle3 = tokio::spawn(async move {
        reg3.call_tool(
            "official",
            "official_fetch",
            Some(json!({"url": "test.com"})),
        )
        .await
    });

    // Wait for all to complete
    let result1 = handle1.await.unwrap().unwrap();
    let result2 = handle2.await.unwrap().unwrap();
    let result3 = handle3.await.unwrap().unwrap();

    // Verify all succeeded
    assert!(!result1.is_error);
    assert!(!result2.is_error);
    assert!(!result3.is_error);

    // Verify distinct results
    assert_eq!(result1.content["query"], "test1");
    assert_eq!(result2.content["tool"], "analyze");
    assert_eq!(result3.content["url"], "test.com");
}

#[tokio::test]
async fn test_registry_lifecycle() {
    let registry = McpClientRegistry::new();

    // Register multiple clients
    let turbo1 = Arc::new(TurboMcpMockClient::new("lifecycle1"));
    let turbo2 = Arc::new(TurboMcpMockClient::new("lifecycle2"));
    let official = Arc::new(OfficialSdkMockClient::new("lifecycle"));

    turbo1.initialize().await.unwrap();
    turbo2.initialize().await.unwrap();
    official.initialize().await.unwrap();

    registry.register("turbo1", turbo1.clone()).unwrap();
    registry.register("turbo2", turbo2.clone()).unwrap();
    registry.register("official", official.clone()).unwrap();

    assert_eq!(registry.count(), 3);

    // Unregister one client
    let removed = registry.unregister("turbo2").unwrap();
    assert!(removed.is_some());
    assert_eq!(registry.count(), 2);

    // Clear all clients
    registry.clear().unwrap();
    assert_eq!(registry.count(), 0);

    // Clients should still be alive (Arc reference)
    assert!(turbo1.is_connected());
    assert!(turbo2.is_connected());
    assert!(official.is_connected());
}

#[tokio::test]
async fn test_mixed_sdk_tool_orchestration() {
    let registry = Arc::new(McpClientRegistry::new());

    // Setup clients
    let turbomcp = Arc::new(TurboMcpMockClient::new("orchestration"));
    turbomcp.initialize().await.unwrap();
    registry.register("turbomcp", turbomcp).unwrap();

    let official = Arc::new(OfficialSdkMockClient::new("orchestration"));
    official.initialize().await.unwrap();
    registry.register("official", official).unwrap();

    // Simulate a workflow using tools from both SDKs
    // Step 1: Search using TurboMCP
    let search_result = registry
        .call_tool(
            "turbomcp",
            "turbomcp_search",
            Some(json!({"query": "data"})),
        )
        .await
        .unwrap();
    assert_eq!(search_result.content["sdk"], "turbomcp");

    // Step 2: Fetch using Official SDK (using search results)
    let fetch_result = registry
        .call_tool(
            "official",
            "official_fetch",
            Some(json!({"url": "https://example.com/data"})),
        )
        .await
        .unwrap();
    assert_eq!(fetch_result.content["sdk"], "official");

    // Step 3: Analyze using TurboMCP (using fetched data)
    let analyze_result = registry
        .call_tool("turbomcp", "turbomcp_analyze", None)
        .await
        .unwrap();
    assert_eq!(analyze_result.content["tool"], "analyze");

    // All operations should succeed
    assert!(!search_result.is_error);
    assert!(!fetch_result.is_error);
    assert!(!analyze_result.is_error);
}
