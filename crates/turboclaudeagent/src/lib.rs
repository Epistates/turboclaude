//! Interactive Agent SDK for TurboClaude
//!
//! This SDK enables interactive agent development within the Claude Code IDE,
//! with features like permission callbacks, hooks, dynamic control, and more.
//!
//! # Key Features
//!
//! - **Bidirectional Protocol**: Full two-way communication with Claude Code IDE
//! - **Permission Callbacks**: Fine-grained control over tool execution
//! - **Hook System**: React to key lifecycle events (PreToolUse, PostToolUse, etc.)
//! - **Interactive Sessions**: Stateful conversations with runtime control
//! - **Dynamic Control**: Interrupt, change model, or modify permissions mid-execution
//! - **Custom Agents**: Define specialized agent personas
//! - **In-Process Tools**: Simple function-based tools without subprocess overhead
//!
//! # Architecture
//!
//! The Agent SDK is built on three key layers:
//!
//! 1. **Protocol Layer** (`turboclaude-protocol`): Shared message types

#![deny(unsafe_code)]
//! 2. **Transport Layer** (`turboclaude-transport`): Subprocess communication
//! 3. **Agent Layer** (this crate): High-level client API
//!
//! # Usage Example
//!
//! ```ignore
//! use turboclaudeagent::ClaudeAgentClient;
//! use turboclaude::types::Models;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let client = ClaudeAgentClient::builder()
//!         .api_key("sk-...")
//!         .model(Models::CLAUDE_SONNET_4_5)
//!         .build()?;
//!
//!     let mut session = client.create_session().await?;
//!     let response = session.query("What is 2+2?").await?;
//!
//!     println!("Response: {}", response);
//!     Ok(())
//! }
//! ```
//!
//! # IDE-Specific
//!
//! This SDK is designed specifically for use within the Claude Code IDE.
//! It requires the Claude CLI to be available and communicates via
//! a bidirectional protocol over stdin/stdout.

#![warn(missing_docs)]

pub mod agent;
pub mod client;
pub mod config;
pub mod error;
pub mod hooks;
pub mod lifecycle;
pub mod mcp;
pub mod message_parser;
pub mod permissions;
pub mod plugin_resolver;
pub mod plugins;
pub mod routing;

// Session module is now organized into sub-modules
pub mod session;

#[cfg(feature = "skills")]
pub mod skills;

pub mod testing;

pub mod retry;

// Re-export commonly used types
pub use agent::AgentDefinition;
pub use client::ClaudeAgentClient;
pub use config::{ClaudeAgentClientConfig, SessionConfig};
pub use error::{AgentError, BackoffStrategy, ErrorRecovery, Result};
pub use hooks::HookRegistry;
pub use lifecycle::{SessionEvent, SessionGuard};
pub use message_parser::{MessageParseError, ParsedMessage, parse_message, parse_message_str};
pub use plugin_resolver::{DependencyResolver, PluginManifest, Version};
pub use plugins::{Plugin, PluginLoader, PluginMetadata, SdkPluginConfig};
pub use retry::{retry, retry_with_recovery};
pub use routing::MessageRouter;
pub use session::{AgentSession, QueryBuilder, SessionState};

#[cfg(feature = "skills")]
pub use skills::{ActiveSkill, SkillDiscoveryResult, SkillManager, ToolValidationResult};

pub use turboclaude_protocol::{
    HookRequest, HookResponse, PermissionCheckRequest, PermissionResponse,
};
