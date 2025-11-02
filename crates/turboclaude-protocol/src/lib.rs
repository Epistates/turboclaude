//! Shared protocol types and definitions for TurboClaude REST and Agent SDKs
//!
//! This crate provides the core type definitions and protocol structures used by both
//! the REST client (turboclaude) and the Agent client (turboclaudeagent). By centralizing
//! these types, we achieve DRY principles and ensure consistency across the ecosystem.
//!
//! # Type Organization
//!
//! - **Content types**: [`content`] - Text, images, tool use/results
//! - **Message types**: [`message`] - Messages, content blocks
//! - **Common types**: [`types`] - Models, usage, cache info
//! - **Agent protocol**: [`agent`] - Control requests, hooks, permissions
//! - **Error types**: [`error`] - Protocol and message errors
//!
//! # Design Principles
//!
//! - **Zero I/O**: All types are pure data structures
//! - **Serialization**: serde-based for both JSON and future formats
//! - **Idiomatic Rust**: Owned types, `Result<T>` for errors, `Option<T>` for optional values
//! - **No circular dependencies**: turboclaude-protocol depends only on serde/chrono

#![deny(unsafe_code)]
#![warn(missing_docs)]
//!
//! # Usage
//!
//! ```ignore
//! use turboclaude_protocol::message::Message;
//! use turboclaude_protocol::content::ContentBlock;
//!
//! let msg = Message {
//!     id: "msg_123".to_string(),
//!     content: vec![ContentBlock::Text { text: "Hello".to_string() }],
//!     // ...
//! };
//! ```

pub mod agent;
pub mod content;
pub mod error;
pub mod hooks;
pub mod message;
pub mod permissions;
pub mod protocol;
pub mod types;

// Re-export commonly used types at crate level
pub use agent::{AgentDefinition, ControlRequest, HookEvent, ToolPermissionRequest};
pub use content::ContentBlock;
pub use error::{ProtocolError, Result};
pub use hooks::{ContinueReason, HookContext, HookMatcher, PermissionDecision, StopReason};
pub use message::{
    AssistantMessage, Message, MessageRequest, ResultMessage, StreamEvent, SystemMessage,
    UserMessage,
};
pub use permissions::{
    AddDirectoriesUpdate, AddRulesUpdate, PermissionBehavior, PermissionRuleValue,
    PermissionUpdate, PermissionUpdateDestination, RemoveDirectoriesUpdate, RemoveRulesUpdate,
    ReplaceRulesUpdate, SetModeUpdate,
};
pub use protocol::{
    ControlCommand, ControlResponse, HookRequest, HookResponse, ModifiedInputs,
    PermissionCheckRequest, PermissionResponse, ProtocolErrorMessage, ProtocolMessage,
    QueryRequest, QueryResponse, RequestId,
};
pub use types::{Model, PermissionMode, ToolDefinition, Usage};
