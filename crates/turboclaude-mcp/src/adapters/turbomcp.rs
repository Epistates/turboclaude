//! TurboMCP adapter - implements McpClient for `turbomcp_client::Client<T>`

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use turbomcp_client::Client as TurbomcpClient;
use turbomcp_transport::Transport;

use crate::error::{McpError, McpResult};
use crate::trait_::{
    McpClient, MessageContent, PromptArgument, PromptInfo, PromptResult, ResourceContents,
    ResourceInfo, ServerInfo, ToolInfo, ToolResult,
};

/// Wraps `turbomcp_client::Client<T>` and implements McpClient trait
///
/// This adapter provides the unified McpClient interface for TurboMCP clients,
/// allowing them to be used alongside other SDK implementations.
///
/// # Example
///
/// ```ignore
/// use turboclaude_mcp::adapters::TurbomcpAdapter;
/// use turbomcp_transport::stdio::StdioTransport;
///
/// let turbomcp_client = turbomcp_client::Client::new(StdioTransport::new());
/// let adapter = TurbomcpAdapter::new(turbomcp_client);
///
/// // Use adapter as McpClient
/// let info = adapter.initialize().await?;
/// let tools = adapter.list_tools().await?;
/// ```
#[derive(Clone)]
pub struct TurbomcpAdapter<T: Transport + 'static> {
    client: TurbomcpClient<T>,
    server_info: Arc<Mutex<Option<ServerInfo>>>,
}

impl<T: Transport + 'static> TurbomcpAdapter<T> {
    /// Create a new TurboMCP adapter
    pub fn new(client: TurbomcpClient<T>) -> Self {
        Self {
            client,
            server_info: Arc::new(Mutex::new(None)),
        }
    }

    /// Get the underlying TurboMCP client
    pub fn inner(&self) -> &TurbomcpClient<T> {
        &self.client
    }
}

#[async_trait]
impl<T: Transport + 'static> McpClient for TurbomcpAdapter<T> {
    async fn initialize(&self) -> McpResult<ServerInfo> {
        let init_result = self
            .client
            .initialize()
            .await
            .map_err(|e| McpError::init(format!("TurboMCP initialization failed: {}", e)))?;

        let server_info = ServerInfo {
            name: init_result.server_info.name.clone(),
            version: init_result.server_info.version.clone(),
        };

        *self.server_info.lock().unwrap() = Some(server_info.clone());
        Ok(server_info)
    }

    async fn close(&self) -> McpResult<()> {
        self.client
            .shutdown()
            .await
            .map_err(|e| McpError::protocol(format!("Failed to close TurboMCP client: {}", e)))
    }

    async fn list_tools(&self) -> McpResult<Vec<ToolInfo>> {
        if !self.is_connected() {
            return Err(McpError::init("TurboMCP client not initialized"));
        }

        let tools = self
            .client
            .list_tools()
            .await
            .map_err(|e| McpError::protocol(format!("Failed to list tools: {}", e)))?;

        Ok(tools
            .into_iter()
            .map(|t| ToolInfo {
                name: t.name,
                description: t.description,
                input_schema: serde_json::to_value(&t.input_schema).ok(),
            })
            .collect())
    }

    async fn call_tool(&self, name: &str, arguments: Option<Value>) -> McpResult<ToolResult> {
        if !self.is_connected() {
            return Err(McpError::init("TurboMCP client not initialized"));
        }

        let args = arguments
            .and_then(|v| {
                v.as_object().map(|o| {
                    o.iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect::<HashMap<String, Value>>()
                })
            })
            .unwrap_or_default();

        let result = self
            .client
            .call_tool(name, if args.is_empty() { None } else { Some(args) })
            .await
            .map_err(|e| {
                // Map TurboMCP errors to our error types
                let err_str = e.to_string();
                if err_str.contains("not found") || err_str.contains("does not exist") {
                    McpError::ToolNotFound(name.to_string())
                } else {
                    McpError::ToolExecutionError(format!("Tool '{}' failed: {}", name, e))
                }
            })?;

        // Extract is_error from result if it exists
        let is_error = result
            .get("is_error")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(ToolResult {
            content: result,
            is_error,
        })
    }

    async fn list_resources(&self) -> McpResult<Vec<ResourceInfo>> {
        if !self.is_connected() {
            return Err(McpError::init("TurboMCP client not initialized"));
        }

        let resources = self
            .client
            .list_resources()
            .await
            .map_err(|e| McpError::protocol(format!("Failed to list resources: {}", e)))?;

        Ok(resources
            .into_iter()
            .map(|r| ResourceInfo {
                uri: r.uri.to_string(),
                name: r.name,
                description: r.description,
                // TurboMCP doesn't expose mutability info, default to read_only
                read_only: true,
            })
            .collect())
    }

    async fn read_resource(&self, uri: &str) -> McpResult<ResourceContents> {
        if !self.is_connected() {
            return Err(McpError::init("TurboMCP client not initialized"));
        }

        let result = self.client.read_resource(uri).await.map_err(|e| {
            let err_str = e.to_string();
            if err_str.contains("not found") || err_str.contains("does not exist") {
                McpError::ResourceNotFound(uri.to_string())
            } else {
                McpError::ResourceReadError(format!("Failed to read '{}': {}", uri, e))
            }
        })?;

        // Extract text content from the response
        let mut text = String::new();
        let mut mime_type = None;

        for content in result.contents.iter() {
            // Match on the enum variant (tuple pattern)
            if let turbomcp_protocol::types::ResourceContent::Text(text_content) = content {
                text = text_content.text.clone();
                mime_type = text_content.mime_type.clone().map(|s| s.to_string());
                break;
            }
        }

        Ok(ResourceContents {
            uri: uri.to_string(),
            mime_type,
            text,
        })
    }

    async fn list_prompts(&self) -> McpResult<Vec<PromptInfo>> {
        if !self.is_connected() {
            return Err(McpError::init("TurboMCP client not initialized"));
        }

        let prompts = self
            .client
            .list_prompts()
            .await
            .map_err(|e| McpError::protocol(format!("Failed to list prompts: {}", e)))?;

        Ok(prompts
            .into_iter()
            .map(|p| PromptInfo {
                name: p.name,
                description: p.description.clone(),
                arguments: p.arguments.map(|args| {
                    args.into_iter()
                        .map(|arg| PromptArgument {
                            name: arg.name,
                            description: arg.description,
                            required: arg.required.unwrap_or(false),
                        })
                        .collect()
                }),
            })
            .collect())
    }

    async fn get_prompt(
        &self,
        name: &str,
        arguments: Option<HashMap<String, String>>,
    ) -> McpResult<PromptResult> {
        if !self.is_connected() {
            return Err(McpError::init("TurboMCP client not initialized"));
        }

        // Convert HashMap<String, String> to HashMap<String, Value>
        let prompt_args = arguments.map(|map| {
            map.into_iter()
                .map(|(k, v)| (k, Value::String(v)))
                .collect::<HashMap<String, Value>>()
        });

        let result = self
            .client
            .get_prompt(name, prompt_args)
            .await
            .map_err(|e| {
                let err_str = e.to_string();
                if err_str.contains("not found") || err_str.contains("does not exist") {
                    McpError::PromptNotFound(name.to_string())
                } else {
                    McpError::PromptExecutionError(format!(
                        "Failed to get prompt '{}': {}",
                        name, e
                    ))
                }
            })?;

        Ok(PromptResult {
            messages: result
                .messages
                .into_iter()
                .map(|m| {
                    // Role is an enum with User/Assistant variants
                    let role_str = match m.role {
                        turbomcp_protocol::types::Role::User => "user",
                        turbomcp_protocol::types::Role::Assistant => "assistant",
                    };

                    // Extract text from content based on the actual type
                    // For now, try to serialize and extract text
                    let text = serde_json::to_value(&m.content)
                        .ok()
                        .and_then(|v| {
                            v.get("text")
                                .and_then(|t| t.as_str())
                                .map(|s| s.to_string())
                        })
                        .unwrap_or_default();

                    MessageContent {
                        role: role_str.to_string(),
                        text,
                    }
                })
                .collect(),
        })
    }

    fn supports_tools(&self) -> bool {
        self.client.capabilities().tools
    }

    fn supports_resources(&self) -> bool {
        self.client.capabilities().resources
    }

    fn supports_prompts(&self) -> bool {
        self.client.capabilities().prompts
    }

    fn supports_resource_subscriptions(&self) -> bool {
        // TurboMCP doesn't expose subscription capabilities separately
        // If resources are supported, subscriptions may be available
        self.client.capabilities().resources
    }

    fn server_info(&self) -> Option<ServerInfo> {
        self.server_info.lock().unwrap().clone()
    }

    fn is_connected(&self) -> bool {
        // Check if client is initialized using the internal atomic flag
        // Since we can't directly access it, we check if server_info is available
        self.server_info().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_creation() {
        // We can't easily create a real client here without a transport
        // but we can test that the adapter struct exists and compiles
        let _ = std::any::type_name::<TurbomcpAdapter<turbomcp_transport::stdio::StdioTransport>>();
    }
}
