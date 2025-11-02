//! Official Rust SDK (rmcp) adapter
//!
//! Wraps the official MCP Rust SDK client in the unified McpClient interface.
//!
//! Enable with the `official-sdk-adapter` feature:
//!
//! ```toml
//! [dependencies]
//! turboclaude-mcp = { version = "0.1", features = ["official-sdk-adapter"] }
//! ```

#[cfg(feature = "official-sdk-adapter")]
mod real_impl {
    use async_trait::async_trait;
    use serde_json::Value;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    use crate::error::{McpError, McpResult};
    use crate::trait_::{
        McpClient, MessageContent, PromptInfo, PromptResult, ResourceContents, ResourceInfo,
        ServerInfo, ToolInfo, ToolResult,
    };

    use rmcp::model::{CallToolRequestParam, GetPromptRequestParam, ReadResourceRequestParam};
    use rmcp::service::{Peer, RoleClient};

    /// Adapter for Official Rust SDK (rmcp)
    ///
    /// Wraps a `Peer<RoleClient>` from the rmcp crate to provide the unified
    /// `McpClient` interface.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use turboclaude_mcp::adapters::OfficialSdkAdapter;
    /// use rmcp::ServiceExt;
    /// use rmcp::transport::TokioChildProcess;
    /// use tokio::process::Command;
    ///
    /// // Create rmcp client
    /// let client = ().serve(TokioChildProcess::new(Command::new("mcp-server"))?).await?;
    ///
    /// // Wrap in adapter
    /// let adapter = OfficialSdkAdapter::new(client.peer);
    /// ```
    #[derive(Clone)]
    pub struct OfficialSdkAdapter {
        peer: Peer<RoleClient>,
        server_info: Arc<Mutex<Option<ServerInfo>>>,
    }

    impl OfficialSdkAdapter {
        /// Create a new Official SDK adapter
        ///
        /// # Arguments
        ///
        /// * `peer` - The `Peer<RoleClient>` from a running rmcp service
        pub fn new(peer: Peer<RoleClient>) -> Self {
            Self {
                peer,
                server_info: Arc::new(Mutex::new(None)),
            }
        }
    }

    #[async_trait]
    impl McpClient for OfficialSdkAdapter {
        async fn initialize(&self) -> McpResult<ServerInfo> {
            // Get peer info (already initialized by rmcp)
            let peer_info = self
                .peer
                .peer_info()
                .ok_or_else(|| McpError::ProtocolError("Server not initialized".to_string()))?;

            let info = ServerInfo {
                name: peer_info.server_info.name.clone(),
                version: peer_info.server_info.version.clone(),
            };

            *self.server_info.lock().await = Some(info.clone());
            Ok(info)
        }

        async fn close(&self) -> McpResult<()> {
            // rmcp handles cleanup when the Peer is dropped
            // We just clear our cached info
            *self.server_info.lock().await = None;
            Ok(())
        }

        async fn list_tools(&self) -> McpResult<Vec<ToolInfo>> {
            let tools = self
                .peer
                .list_all_tools()
                .await
                .map_err(|e| McpError::ProtocolError(e.to_string()))?;

            Ok(tools
                .into_iter()
                .map(|tool| ToolInfo {
                    name: tool.name.to_string(),
                    description: tool.description.map(|d| d.to_string()),
                    input_schema: Some(
                        serde_json::to_value(tool.input_schema.as_ref())
                            .unwrap_or(Value::Object(serde_json::Map::new())),
                    ),
                })
                .collect())
        }

        async fn call_tool(&self, name: &str, arguments: Option<Value>) -> McpResult<ToolResult> {
            // Convert Value to JsonObject (Map) if provided
            let args_map = arguments.and_then(|v| match v {
                Value::Object(map) => Some(map),
                _ => None,
            });

            let result = self
                .peer
                .call_tool(CallToolRequestParam {
                    name: name.to_string().into(),
                    arguments: args_map,
                })
                .await
                .map_err(|e| McpError::ToolExecutionError(e.to_string()))?;

            // Check if result contains an error
            let is_error = result.is_error.unwrap_or(false);

            // Extract content - handle Annotated<RawContent> through Deref
            let content = if result.content.len() == 1 {
                // Single content item - serialize it
                match &*result.content[0] {
                    rmcp::model::RawContent::Text(text_content) => {
                        Value::String(text_content.text.clone())
                    }
                    rmcp::model::RawContent::Image(image_content) => serde_json::json!({
                        "type": "image",
                        "data": image_content.data,
                        "mimeType": image_content.mime_type
                    }),
                    rmcp::model::RawContent::Resource(resource_content) => {
                        serde_json::json!({
                            "type": "resource",
                            "resource": resource_content.resource
                        })
                    }
                    rmcp::model::RawContent::Audio(audio_content) => serde_json::json!({
                        "type": "audio",
                        "data": audio_content.data,
                        "mimeType": audio_content.mime_type
                    }),
                    rmcp::model::RawContent::ResourceLink(resource) => {
                        serde_json::to_value(resource).unwrap_or(Value::Null)
                    }
                }
            } else {
                // Multiple content items - serialize as array
                serde_json::to_value(&result.content).unwrap_or(Value::Array(vec![]))
            };

            Ok(ToolResult { content, is_error })
        }

        async fn list_resources(&self) -> McpResult<Vec<ResourceInfo>> {
            let resources = self
                .peer
                .list_all_resources()
                .await
                .map_err(|e| McpError::ProtocolError(e.to_string()))?;

            Ok(resources
                .into_iter()
                .map(|resource| {
                    // Deref the Annotated<RawResource> to get RawResource
                    ResourceInfo {
                        uri: resource.uri.clone(),
                        name: resource.name.clone(),
                        description: resource.description.clone(),
                        read_only: true, // rmcp doesn't expose this, assume read-only
                    }
                })
                .collect())
        }

        async fn read_resource(&self, uri: &str) -> McpResult<ResourceContents> {
            let result = self
                .peer
                .read_resource(ReadResourceRequestParam {
                    uri: uri.to_string(),
                })
                .await
                .map_err(|e| McpError::ResourceReadError(e.to_string()))?;

            // Extract text content from the first content item
            if let Some(content) = result.contents.first() {
                match content {
                    rmcp::model::ResourceContents::TextResourceContents {
                        uri: content_uri,
                        mime_type,
                        text,
                        ..
                    } => Ok(ResourceContents {
                        uri: content_uri.clone(),
                        mime_type: mime_type.clone().or(Some("text/plain".to_string())),
                        text: text.clone(),
                    }),
                    rmcp::model::ResourceContents::BlobResourceContents {
                        uri: content_uri,
                        mime_type,
                        ..
                    } => Err(McpError::ResourceReadError(format!(
                        "Blob resource not supported: {} ({})",
                        content_uri,
                        mime_type.as_deref().unwrap_or("application/octet-stream")
                    ))),
                }
            } else {
                Err(McpError::ResourceReadError(
                    "No content in resource".to_string(),
                ))
            }
        }

        async fn list_prompts(&self) -> McpResult<Vec<PromptInfo>> {
            let prompts = self
                .peer
                .list_all_prompts()
                .await
                .map_err(|e| McpError::ProtocolError(e.to_string()))?;

            Ok(prompts
                .into_iter()
                .map(|prompt| PromptInfo {
                    name: prompt.name,
                    description: prompt.description,
                    arguments: None, // rmcp uses a different structure, simplified for now
                })
                .collect())
        }

        async fn get_prompt(
            &self,
            name: &str,
            arguments: Option<HashMap<String, String>>,
        ) -> McpResult<PromptResult> {
            // Convert HashMap<String, String> to JsonObject (Map<String, Value>)
            let args_map =
                arguments.map(|hm| hm.into_iter().map(|(k, v)| (k, Value::String(v))).collect());

            let result = self
                .peer
                .get_prompt(GetPromptRequestParam {
                    name: name.to_string(),
                    arguments: args_map,
                })
                .await
                .map_err(|e| McpError::PromptExecutionError(e.to_string()))?;

            // Convert rmcp PromptMessage to our MessageContent
            let messages = result
                .messages
                .into_iter()
                .map(|msg| {
                    // Convert role enum to string
                    let role = match msg.role {
                        rmcp::model::PromptMessageRole::User => "user".to_string(),
                        rmcp::model::PromptMessageRole::Assistant => "assistant".to_string(),
                    };

                    // Extract text from content (PromptMessageContent is a single variant enum)
                    let text = match msg.content {
                        rmcp::model::PromptMessageContent::Text { text } => text,
                        rmcp::model::PromptMessageContent::Image { .. } => {
                            "[Image content not converted to text]".to_string()
                        }
                        rmcp::model::PromptMessageContent::Resource { resource } => {
                            // Extract text from embedded resource if available
                            match &resource.resource {
                                rmcp::model::ResourceContents::TextResourceContents {
                                    text,
                                    ..
                                } => text.clone(),
                                rmcp::model::ResourceContents::BlobResourceContents { .. } => {
                                    "[Blob resource content not converted to text]".to_string()
                                }
                            }
                        }
                        rmcp::model::PromptMessageContent::ResourceLink { link } => {
                            format!("[Resource link: {}]", link.uri)
                        }
                    };

                    MessageContent { role, text }
                })
                .collect();

            Ok(PromptResult { messages })
        }

        fn supports_tools(&self) -> bool {
            // rmcp always supports tools
            true
        }

        fn supports_resources(&self) -> bool {
            // rmcp always supports resources
            true
        }

        fn supports_prompts(&self) -> bool {
            // rmcp always supports prompts
            true
        }

        fn supports_resource_subscriptions(&self) -> bool {
            // rmcp supports subscriptions
            true
        }

        fn server_info(&self) -> Option<ServerInfo> {
            self.server_info.try_lock().ok()?.clone()
        }

        fn is_connected(&self) -> bool {
            // If we have server info, we're connected
            self.server_info
                .try_lock()
                .map(|i| i.is_some())
                .unwrap_or(false)
        }
    }
}

#[cfg(feature = "official-sdk-adapter")]
pub use real_impl::OfficialSdkAdapter;

// Stub implementation - always available for tests
mod stub_impl {
    use async_trait::async_trait;
    use serde_json::Value;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    use crate::error::{McpError, McpResult};
    use crate::trait_::{
        McpClient, PromptInfo, PromptResult, ResourceContents, ResourceInfo, ServerInfo, ToolInfo,
        ToolResult,
    };

    /// Stub adapter for testing
    ///
    /// This is a non-functional stub available for testing when the real
    /// `official-sdk-adapter` feature is not needed.
    #[derive(Debug, Clone)]
    pub struct OfficialSdkStub {
        server_info: Arc<Mutex<Option<ServerInfo>>>,
        is_connected: Arc<Mutex<bool>>,
    }

    impl OfficialSdkStub {
        /// Create a new stub adapter
        pub fn new() -> Self {
            Self {
                server_info: Arc::new(Mutex::new(None)),
                is_connected: Arc::new(Mutex::new(false)),
            }
        }
    }

    impl Default for OfficialSdkStub {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl McpClient for OfficialSdkStub {
        async fn initialize(&self) -> McpResult<ServerInfo> {
            let info = ServerInfo {
                name: "official-sdk-stub".to_string(),
                version: "0.1.0-stub".to_string(),
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
            Ok(vec![])
        }

        async fn call_tool(&self, _name: &str, _arguments: Option<Value>) -> McpResult<ToolResult> {
            Err(McpError::FeatureNotSupported(
                "Official SDK adapter not enabled. Add feature: official-sdk-adapter".to_string(),
            ))
        }

        async fn list_resources(&self) -> McpResult<Vec<ResourceInfo>> {
            Ok(vec![])
        }

        async fn read_resource(&self, _uri: &str) -> McpResult<ResourceContents> {
            Err(McpError::FeatureNotSupported(
                "Official SDK adapter not enabled".to_string(),
            ))
        }

        async fn list_prompts(&self) -> McpResult<Vec<PromptInfo>> {
            Ok(vec![])
        }

        async fn get_prompt(
            &self,
            _name: &str,
            _arguments: Option<HashMap<String, String>>,
        ) -> McpResult<PromptResult> {
            Err(McpError::FeatureNotSupported(
                "Official SDK adapter not enabled".to_string(),
            ))
        }

        fn supports_tools(&self) -> bool {
            false
        }

        fn supports_resources(&self) -> bool {
            false
        }

        fn supports_prompts(&self) -> bool {
            false
        }

        fn supports_resource_subscriptions(&self) -> bool {
            false
        }

        fn server_info(&self) -> Option<ServerInfo> {
            self.server_info.try_lock().ok()?.clone()
        }

        fn is_connected(&self) -> bool {
            self.is_connected.try_lock().map(|i| *i).unwrap_or(false)
        }
    }
}

// Always export the stub for testing
pub use stub_impl::OfficialSdkStub;

// When feature is disabled, also export stub as OfficialSdkAdapter for backward compatibility
#[cfg(not(feature = "official-sdk-adapter"))]
pub use stub_impl::OfficialSdkStub as OfficialSdkAdapter;
