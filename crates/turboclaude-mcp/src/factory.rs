//! Factory and builder for creating McpClient instances
//!
//! Provides convenient methods for creating different types of MCP clients.

use crate::adapters::turbomcp::TurbomcpAdapter;
use crate::trait_::BoxedMcpClient;
use std::sync::Arc;
use turbomcp_client::Client as TurbomcpClient;
use turbomcp_transport::stdio::StdioTransport;

/// SDK type selector for factory creation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdkType {
    /// TurboMCP SDK
    TurboMcp,
    /// Official Anthropic Rust SDK
    Official,
}

/// Builder for creating McpClient instances
pub struct McpClientBuilder {
    sdk_type: Option<SdkType>,
}

impl McpClientBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self { sdk_type: None }
    }

    /// Select the SDK type
    pub fn with_sdk(mut self, sdk_type: SdkType) -> Self {
        self.sdk_type = Some(sdk_type);
        self
    }

    /// Build the client
    pub fn build(self) -> Result<BoxedMcpClient, String> {
        match self.sdk_type {
            Some(SdkType::Official) => {
                Err(
                    "Official SDK adapter must be created directly with an initialized Peer<RoleClient>. \
                     Use: OfficialSdkAdapter::new(peer)".to_string()
                )
            }
            Some(SdkType::TurboMcp) => {
                let transport = StdioTransport::new();
                let client = TurbomcpClient::new(transport);
                let adapter = TurbomcpAdapter::new(client);
                Ok(Arc::new(adapter))
            }
            None => Err("SDK type not specified".to_string()),
        }
    }
}

impl Default for McpClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creation() {
        let builder = McpClientBuilder::new();
        assert!(builder.sdk_type.is_none());
    }

    #[test]
    fn test_builder_with_official_sdk() {
        let builder = McpClientBuilder::new().with_sdk(SdkType::Official);
        assert_eq!(builder.sdk_type, Some(SdkType::Official));
    }

    #[test]
    fn test_builder_build_official() {
        let client = McpClientBuilder::new().with_sdk(SdkType::Official).build();
        // Official SDK adapter requires a peer argument, factory should return error
        assert!(client.is_err());
    }

    #[test]
    fn test_builder_build_no_sdk() {
        let client = McpClientBuilder::new().build();
        assert!(client.is_err());
    }

    #[test]
    fn test_default_builder() {
        let builder = McpClientBuilder::default();
        assert!(builder.sdk_type.is_none());
    }
}
