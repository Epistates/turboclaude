//! Model Context Protocol (MCP) integration for TurboClaude Agent.
//!
//! This module provides SDK MCP server support, allowing tools to run
//! in-process without subprocess overhead.

pub mod sdk;

// Re-export commonly used types
pub use sdk::{SdkMcpServer, SdkMcpServerBuilder, SdkTool, SdkToolError};
