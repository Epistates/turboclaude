//! Integration tests for McpClient trait design and contract

use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use turboclaude_mcp::{
    McpClient, McpError, McpResult, MessageContent, PromptArgument, PromptInfo, PromptResult,
    ResourceContents, ResourceInfo, ServerInfo, ToolInfo, ToolResult,
};

/// Mock MCP client for testing trait contract
#[derive(Debug, Clone)]
struct MockMcpClient {
    server_info: Arc<Mutex<Option<ServerInfo>>>,
    is_connected: Arc<Mutex<bool>>,
    initialized: Arc<Mutex<bool>>,
}

impl MockMcpClient {
    fn new() -> Self {
        Self {
            server_info: Arc::new(Mutex::new(None)),
            is_connected: Arc::new(Mutex::new(false)),
            initialized: Arc::new(Mutex::new(false)),
        }
    }
}

#[async_trait]
impl McpClient for MockMcpClient {
    async fn initialize(&self) -> McpResult<ServerInfo> {
        let info = ServerInfo {
            name: "mock-server".to_string(),
            version: "1.0.0".to_string(),
        };
        *self.server_info.lock().await = Some(info.clone());
        *self.is_connected.lock().await = true;
        *self.initialized.lock().await = true;
        Ok(info)
    }

    async fn close(&self) -> McpResult<()> {
        *self.is_connected.lock().await = false;
        Ok(())
    }

    async fn list_tools(&self) -> McpResult<Vec<ToolInfo>> {
        if !*self.initialized.lock().await {
            return Err(McpError::init("Client not initialized"));
        }

        Ok(vec![
            ToolInfo {
                name: "tool1".to_string(),
                description: Some("First test tool".to_string()),
                input_schema: Some(
                    json!({"type": "object", "properties": {"arg": {"type": "string"}}}),
                ),
            },
            ToolInfo {
                name: "tool2".to_string(),
                description: None,
                input_schema: None,
            },
        ])
    }

    async fn call_tool(
        &self,
        name: &str,
        arguments: Option<serde_json::Value>,
    ) -> McpResult<ToolResult> {
        if !*self.initialized.lock().await {
            return Err(McpError::init("Client not initialized"));
        }

        match name {
            "tool1" => Ok(ToolResult {
                content: json!({"result": "tool1 called", "args": arguments}),
                is_error: false,
            }),
            "tool2" => Ok(ToolResult {
                content: json!({"result": "tool2 called"}),
                is_error: false,
            }),
            "error_tool" => Ok(ToolResult {
                content: json!({"error": "tool failed"}),
                is_error: true,
            }),
            _ => Err(McpError::ToolNotFound(format!("Tool '{}' not found", name))),
        }
    }

    async fn list_resources(&self) -> McpResult<Vec<ResourceInfo>> {
        if !*self.initialized.lock().await {
            return Err(McpError::init("Client not initialized"));
        }

        Ok(vec![
            ResourceInfo {
                uri: "resource://test/file1.txt".to_string(),
                name: "file1".to_string(),
                description: Some("Test resource 1".to_string()),
                read_only: true,
            },
            ResourceInfo {
                uri: "resource://test/file2.txt".to_string(),
                name: "file2".to_string(),
                description: None,
                read_only: false,
            },
        ])
    }

    async fn read_resource(&self, uri: &str) -> McpResult<ResourceContents> {
        if !*self.initialized.lock().await {
            return Err(McpError::init("Client not initialized"));
        }

        match uri {
            "resource://test/file1.txt" => Ok(ResourceContents {
                uri: uri.to_string(),
                mime_type: Some("text/plain".to_string()),
                text: "Content of file1".to_string(),
            }),
            "resource://test/file2.txt" => Ok(ResourceContents {
                uri: uri.to_string(),
                mime_type: Some("text/plain".to_string()),
                text: "Content of file2".to_string(),
            }),
            _ => Err(McpError::ResourceNotFound(format!(
                "Resource '{}' not found",
                uri
            ))),
        }
    }

    async fn list_prompts(&self) -> McpResult<Vec<PromptInfo>> {
        if !*self.initialized.lock().await {
            return Err(McpError::init("Client not initialized"));
        }

        Ok(vec![
            PromptInfo {
                name: "prompt1".to_string(),
                description: Some("First test prompt".to_string()),
                arguments: Some(vec![PromptArgument {
                    name: "question".to_string(),
                    description: Some("User question".to_string()),
                    required: true,
                }]),
            },
            PromptInfo {
                name: "prompt2".to_string(),
                description: None,
                arguments: None,
            },
        ])
    }

    async fn get_prompt(
        &self,
        name: &str,
        arguments: Option<HashMap<String, String>>,
    ) -> McpResult<PromptResult> {
        if !*self.initialized.lock().await {
            return Err(McpError::init("Client not initialized"));
        }

        match name {
            "prompt1" => Ok(PromptResult {
                messages: vec![
                    MessageContent {
                        role: "user".to_string(),
                        text: format!("Q: {:?}", arguments),
                    },
                    MessageContent {
                        role: "assistant".to_string(),
                        text: "A: Test answer".to_string(),
                    },
                ],
            }),
            "prompt2" => Ok(PromptResult {
                messages: vec![MessageContent {
                    role: "assistant".to_string(),
                    text: "Default response".to_string(),
                }],
            }),
            _ => Err(McpError::PromptNotFound(format!(
                "Prompt '{}' not found",
                name
            ))),
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
        futures::executor::block_on(async { self.server_info.lock().await.clone() })
    }

    fn is_connected(&self) -> bool {
        futures::executor::block_on(async { *self.is_connected.lock().await })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[tokio::test]
async fn test_mcpclient_trait_initialization() {
    let client = MockMcpClient::new();
    assert!(!client.is_connected());
    assert!(client.server_info().is_none());

    let info = client.initialize().await.expect("initialization failed");
    assert_eq!(info.name, "mock-server");
    assert_eq!(info.version, "1.0.0");
    assert!(client.is_connected());
    assert!(client.server_info().is_some());
}

#[tokio::test]
async fn test_mcpclient_trait_close() {
    let client = MockMcpClient::new();
    client.initialize().await.expect("initialization failed");
    assert!(client.is_connected());

    client.close().await.expect("close failed");
    assert!(!client.is_connected());
}

#[tokio::test]
async fn test_mcpclient_trait_list_tools() {
    let client = MockMcpClient::new();
    client.initialize().await.expect("initialization failed");

    let tools = client.list_tools().await.expect("list_tools failed");
    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0].name, "tool1");
    assert_eq!(tools[1].name, "tool2");
    assert!(tools[0].description.is_some());
    assert!(tools[1].description.is_none());
}

#[tokio::test]
async fn test_mcpclient_trait_list_tools_not_initialized() {
    let client = MockMcpClient::new();

    let result = client.list_tools().await;
    assert!(result.is_err());
    match result {
        Err(McpError::InitializationError(_)) => {} // Expected
        _ => panic!("Expected InitializationError"),
    }
}

#[tokio::test]
async fn test_mcpclient_trait_call_tool() {
    let client = MockMcpClient::new();
    client.initialize().await.expect("initialization failed");

    let result = client
        .call_tool("tool1", Some(json!({"arg": "value"})))
        .await
        .expect("call_tool failed");

    assert!(!result.is_error);
    assert_eq!(
        result.content.get("result").unwrap().as_str().unwrap(),
        "tool1 called"
    );
}

#[tokio::test]
async fn test_mcpclient_trait_call_tool_not_found() {
    let client = MockMcpClient::new();
    client.initialize().await.expect("initialization failed");

    let result = client.call_tool("nonexistent", None).await;
    assert!(result.is_err());
    match result {
        Err(McpError::ToolNotFound(_)) => {} // Expected
        _ => panic!("Expected ToolNotFound"),
    }
}

#[tokio::test]
async fn test_mcpclient_trait_call_tool_error_result() {
    let client = MockMcpClient::new();
    client.initialize().await.expect("initialization failed");

    let result = client
        .call_tool("error_tool", None)
        .await
        .expect("call_tool failed");

    assert!(result.is_error);
}

#[tokio::test]
async fn test_mcpclient_trait_list_resources() {
    let client = MockMcpClient::new();
    client.initialize().await.expect("initialization failed");

    let resources = client
        .list_resources()
        .await
        .expect("list_resources failed");

    assert_eq!(resources.len(), 2);
    assert_eq!(resources[0].uri, "resource://test/file1.txt");
    assert!(resources[0].description.is_some());
    assert!(resources[1].description.is_none());
}

#[tokio::test]
async fn test_mcpclient_trait_read_resource() {
    let client = MockMcpClient::new();
    client.initialize().await.expect("initialization failed");

    let content = client
        .read_resource("resource://test/file1.txt")
        .await
        .expect("read_resource failed");

    assert_eq!(content.uri, "resource://test/file1.txt");
    assert_eq!(content.text, "Content of file1");
    assert_eq!(content.mime_type, Some("text/plain".to_string()));
}

#[tokio::test]
async fn test_mcpclient_trait_read_resource_not_found() {
    let client = MockMcpClient::new();
    client.initialize().await.expect("initialization failed");

    let result = client.read_resource("resource://nonexistent").await;

    assert!(result.is_err());
    match result {
        Err(McpError::ResourceNotFound(_)) => {} // Expected
        _ => panic!("Expected ResourceNotFound"),
    }
}

#[tokio::test]
async fn test_mcpclient_trait_list_prompts() {
    let client = MockMcpClient::new();
    client.initialize().await.expect("initialization failed");

    let prompts = client.list_prompts().await.expect("list_prompts failed");

    assert_eq!(prompts.len(), 2);
    assert_eq!(prompts[0].name, "prompt1");
    assert!(prompts[0].arguments.is_some());
    assert!(prompts[1].arguments.is_none());
}

#[tokio::test]
async fn test_mcpclient_trait_get_prompt() {
    let client = MockMcpClient::new();
    client.initialize().await.expect("initialization failed");

    let result = client
        .get_prompt(
            "prompt1",
            Some(HashMap::from([(
                "question".to_string(),
                "What is MCP?".to_string(),
            )])),
        )
        .await
        .expect("get_prompt failed");

    assert_eq!(result.messages.len(), 2);
    assert_eq!(result.messages[0].role, "user");
    assert_eq!(result.messages[1].role, "assistant");
}

#[tokio::test]
async fn test_mcpclient_trait_get_prompt_not_found() {
    let client = MockMcpClient::new();
    client.initialize().await.expect("initialization failed");

    let result = client.get_prompt("nonexistent", None).await;

    assert!(result.is_err());
    match result {
        Err(McpError::PromptNotFound(_)) => {} // Expected
        _ => panic!("Expected PromptNotFound"),
    }
}

#[tokio::test]
async fn test_mcpclient_trait_supports_features() {
    let client = MockMcpClient::new();
    client.initialize().await.expect("initialization failed");

    assert!(client.supports_tools());
    assert!(client.supports_resources());
    assert!(client.supports_prompts());
    assert!(!client.supports_resource_subscriptions());
}

#[tokio::test]
async fn test_mcpclient_trait_server_info_after_init() {
    let client = MockMcpClient::new();

    assert!(client.server_info().is_none());

    client.initialize().await.expect("initialization failed");

    let info = client.server_info().expect("server_info should exist");
    assert_eq!(info.name, "mock-server");
    assert_eq!(info.version, "1.0.0");
}

#[tokio::test]
async fn test_mcpclient_trait_clone() {
    let client = MockMcpClient::new();
    client.initialize().await.expect("initialization failed");

    let cloned = client.clone();
    assert!(cloned.is_connected());
    assert!(cloned.server_info().is_some());

    // Verify both can call methods
    let _tools = client.list_tools().await.expect("list_tools failed");
    let _tools2 = cloned.list_tools().await.expect("list_tools failed");
}

#[tokio::test]
async fn test_mcpclient_trait_boxed() {
    use std::sync::Arc;
    let client = MockMcpClient::new();
    client.initialize().await.expect("initialization failed");

    let boxed: std::sync::Arc<dyn McpClient> = Arc::new(client);
    assert!(boxed.is_connected());

    let tools = boxed.list_tools().await.expect("list_tools failed");
    assert_eq!(tools.len(), 2);
}

#[tokio::test]
async fn test_mcpclient_trait_multiple_operations_sequence() {
    let client = MockMcpClient::new();

    // Sequence of operations
    client.initialize().await.expect("initialization failed");

    let tools = client.list_tools().await.expect("list_tools failed");
    assert_eq!(tools.len(), 2);

    let result = client
        .call_tool("tool1", Some(json!({"arg": "test"})))
        .await
        .expect("call_tool failed");
    assert!(!result.is_error);

    let resources = client
        .list_resources()
        .await
        .expect("list_resources failed");
    assert_eq!(resources.len(), 2);

    let content = client
        .read_resource("resource://test/file1.txt")
        .await
        .expect("read_resource failed");
    assert_eq!(content.text, "Content of file1");

    let prompts = client.list_prompts().await.expect("list_prompts failed");
    assert_eq!(prompts.len(), 2);

    client.close().await.expect("close failed");
    assert!(!client.is_connected());
}
