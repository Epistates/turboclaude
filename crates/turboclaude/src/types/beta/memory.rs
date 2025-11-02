//! Memory API types for persistent context across conversations
//!
//! The Memory API enables Claude to store and retrieve information across
//! conversations, maintaining context over time.
//!
//! ## Overview
//!
//! Memory allows Claude to:
//! - Store facts, preferences, and context from conversations
//! - Retrieve relevant information when needed
//! - Maintain continuity across multiple interactions
//!
//! ## Beta Status
//!
//! This is a beta feature. Requires beta header: `"anthropic-beta": "memory-2025-01-24"`

use serde::{Deserialize, Serialize};

/// Memory tool for storing and retrieving conversation context
///
/// Enables Claude to use a dedicated memory file for persisting information
/// across conversations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename = "memory_20250124")]
pub struct MemoryTool {
    /// Tool name (must be "memory")
    pub name: String,
}

impl MemoryTool {
    /// Create a new memory tool
    ///
    /// # Example
    ///
    /// ```rust
    /// use turboclaude::types::beta::MemoryTool;
    ///
    /// let memory = MemoryTool::new();
    /// ```
    pub fn new() -> Self {
        Self {
            name: "memory".to_string(),
        }
    }
}

impl Default for MemoryTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory entry stored by Claude
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Unique identifier for this memory
    pub id: String,

    /// The content stored in memory
    pub content: String,

    /// When this memory was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,

    /// When this memory was last accessed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessed_at: Option<String>,

    /// Metadata about this memory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Request to store information in memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStoreRequest {
    /// Content to store
    pub content: String,

    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Request to retrieve information from memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRetrieveRequest {
    /// Query to search for in memory
    pub query: String,

    /// Maximum number of results to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// Response from memory retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRetrieveResponse {
    /// Matching memory entries
    pub entries: Vec<MemoryEntry>,

    /// Whether there are more results available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_more: Option<bool>,
}

/// Request to delete a memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDeleteRequest {
    /// ID of the memory entry to delete
    pub id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tool_creation() {
        let memory = MemoryTool::new();
        assert_eq!(memory.name, "memory");
    }

    #[test]
    fn test_memory_tool_serialization() {
        let memory = MemoryTool::new();
        let json = serde_json::to_string(&memory).unwrap();
        assert!(json.contains("\"type\":\"memory_20250124\""));
        assert!(json.contains("\"name\":\"memory\""));
    }

    #[test]
    fn test_memory_entry() {
        let entry = MemoryEntry {
            id: "mem_123".to_string(),
            content: "User prefers TypeScript".to_string(),
            created_at: Some("2025-01-24T10:00:00Z".to_string()),
            accessed_at: None,
            metadata: None,
        };

        assert_eq!(entry.id, "mem_123");
        assert_eq!(entry.content, "User prefers TypeScript");
    }
}
