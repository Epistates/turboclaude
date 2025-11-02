//! Tool-related types

use serde::{Deserialize, Serialize};

/// A tool that can be used by the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Name of the tool
    pub name: String,

    /// Description of what the tool does
    pub description: String,

    /// JSON Schema for the tool's input parameters
    pub input_schema: serde_json::Value,
}

impl Tool {
    /// Create a new tool.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
        }
    }
}

/// Tool choice preference.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ToolChoice {
    /// Let the model choose
    Auto,

    /// Force the model to use any tool
    Any,

    /// Force the model to use a specific tool
    Tool {
        /// Name of the tool to use
        name: String,
    },
}

impl ToolChoice {
    /// Create a tool choice for a specific tool.
    pub fn specific(name: impl Into<String>) -> Self {
        Self::Tool { name: name.into() }
    }
}

impl Default for ToolChoice {
    fn default() -> Self {
        Self::Auto
    }
}
