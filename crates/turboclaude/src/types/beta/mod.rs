//! Beta API types
//!
//! This module contains type definitions for beta/experimental Anthropic API features.
//! These APIs may change or be removed in future versions.

// Re-export beta types
pub use citations::*;
pub use context_management::*;
pub use files::*;
pub use memory::*;
pub use models::*;
pub use skills::*;
pub use thinking::{
    BetaAllThinkingTurnsParam, BetaClearThinking20251015EditParam,
    BetaClearThinking20251015EditResponse, BetaThinkingTurnsParam, Keep, ThinkingBlock,
    ThinkingConfig,
};
pub use tools::*;

/// Citation types for source attribution
pub mod citations;

/// Context management types for controlling conversation history
///
/// Supports:
/// - Context management edits for modifying conversation state
/// - Clearing old thinking blocks
/// - Managing token usage in long-running conversations
pub mod context_management;

/// Extended thinking types and context management
///
/// Supports:
/// - Thinking blocks for extended reasoning
/// - Thinking configuration
/// - Clear thinking operations to remove old thinking blocks
/// - Thinking turn parameters for fine-grained control
pub mod thinking;

/// Files API types
pub mod files;

/// Memory API types for persistent context
pub mod memory;

/// Models API types
pub mod models;

/// Skills API types
pub mod skills;

/// Beta tools types
pub mod tools;
