//! # turboclaude-mcp
//!
//! SDK-agnostic MCP (Model Context Protocol) abstraction layer for turboclaude.
//!
//! This crate provides a unified interface for interacting with MCP servers,
//! regardless of the underlying SDK implementation.
//!
//! ## Overview
//!
//! The main component is the [`McpClient`] trait, which provides a common interface
//! for all MCP operations:
//!
//! - **Tool Operations**: List and call tools
//! - **Resource Operations**: List and read resources
//! - **Prompt Operations**: List and execute prompts
//! - **Capability Queries**: Check what features the server supports
//!
//! ## Architecture
//!
//! ```text

#![deny(unsafe_code)]
#![warn(missing_docs)]
//! Application Code (uses McpClient trait)
//!     ↓
//! SDK-specific Adapters
//! ├── TurboMCP Adapter (`Client<T>` → `McpClient`)
//! └── Official SDK Adapter (`Service<R>` → `McpClient`)
//!     ↓
//! Underlying MCP Server
//! ```
//!
//! ## Features
//!
//! - **SDK Agnostic**: Single interface works with any MCP SDK
//! - **Type Safe**: Compile-time guarantee of capabilities
//! - **Async/Await**: Built on tokio and async-trait
//! - **Error Handling**: Comprehensive error types
//! - **Zero Overhead**: Adapters use zero-cost abstractions where possible
//!
//! ## Example
//!
//! ```ignore
//! use turboclaude_mcp::McpClient;
//!
//! async fn interact_with_server(client: &(impl McpClient + ?Sized)) -> Result<()> {
//!     // Initialize connection
//!     let info = client.initialize().await?;
//!     println!("Connected to: {}", info.name);
//!
//!     // List tools
//!     let tools = client.list_tools().await?;
//!     println!("Available tools: {}", tools.len());
//!
//!     // Call a tool
//!     if let Some(tool) = tools.first() {
//!         let result = client.call_tool(&tool.name, None).await?;
//!         println!("Result: {}", result.content);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Adapters
//!
//! Adapters bridge the gap between SDK-specific APIs and the unified interface:
//!
//! - **TurboMCP Adapter** (feature: `turbomcp-adapter`): Wraps `turbomcp_client::Client<T>`
//! - **Official Rust SDK Adapter**: Wraps `rmcp::Service<R>`
//!
//! Enable features in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! turboclaude-mcp = { path = ".", features = ["turbomcp-adapter"] }
//! ```

pub mod adapters;
pub mod bridge;
pub mod error;
pub mod factory;
pub mod registry;
pub mod trait_;

pub use bridge::{McpBridge, McpBridgeBuilder};
pub use error::{McpError, McpResult};
pub use factory::{McpClientBuilder, SdkType};
pub use registry::McpClientRegistry;
pub use trait_::{
    BoxedMcpClient, McpClient, MessageContent, PromptArgument, PromptInfo, PromptResult,
    ResourceContents, ResourceInfo, ServerInfo, ToolInfo, ToolResult,
};

#[cfg(feature = "turbomcp-adapter")]
#[cfg_attr(docsrs, doc(cfg(feature = "turbomcp-adapter")))]
pub use adapters::TurbomcpAdapter;

// Always export OfficialSdkAdapter (stub when feature disabled, real when enabled)
pub use adapters::OfficialSdkAdapter;
