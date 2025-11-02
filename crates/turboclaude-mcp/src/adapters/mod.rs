//! SDK-specific adapters that implement McpClient trait
//!
//! This module provides adapters for different MCP SDK implementations,
//! allowing them to be used through the unified McpClient interface.

#[cfg(feature = "turbomcp-adapter")]
pub mod turbomcp;

#[cfg(feature = "turbomcp-adapter")]
pub use turbomcp::TurbomcpAdapter;

pub mod official_sdk;
pub use official_sdk::{OfficialSdkAdapter, OfficialSdkStub};
