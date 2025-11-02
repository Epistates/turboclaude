//! Content block types
//!
//! Represents the different types of content that can appear in messages.
//! Matches the Anthropic API's content block structure.

use serde::{Deserialize, Serialize};

/// A content block in a message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Plain text content.
    #[serde(rename = "text")]
    Text {
        /// The text content.
        text: String,
    },

    /// Image content.
    #[serde(rename = "image")]
    Image {
        /// The source of the image data.
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<ImageSource>,
    },

    /// A request from the model to use a tool.
    #[serde(rename = "tool_use")]
    ToolUse {
        /// The unique identifier for this tool use request.
        id: String,
        /// The name of the tool to be used.
        name: String,
        /// The input to the tool, as a JSON object.
        #[serde(default)]
        input: serde_json::Value,
    },

    /// The result of a tool execution.
    #[serde(rename = "tool_result")]
    ToolResult {
        /// The `id` of the `tool_use` block this result is for.
        tool_use_id: String,
        /// The content of the tool's output.
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        /// Whether the tool execution resulted in an error.
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },

    /// A block indicating that the model is performing an extended computation.
    #[serde(rename = "thinking")]
    Thinking {
        /// A description of the thinking process.
        thinking: String,
    },

    /// Content from a document.
    #[serde(rename = "document")]
    Document {
        /// The source of the document.
        source: DocumentSource,
        /// The title of the document.
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
    },
}

/// Image source specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImageSource {
    /// A base64-encoded image.
    #[serde(rename = "base64")]
    Base64 {
        /// The media type of the image (e.g., "image/jpeg").
        media_type: String,
        /// The base64-encoded image data.
        data: String,
    },

    /// An image referenced by a URL.
    #[serde(rename = "url")]
    Url {
        /// The URL of the image.
        url: String,
    },
}

/// Document source specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DocumentSource {
    /// A base64-encoded PDF document.
    #[serde(rename = "pdf")]
    Pdf {
        /// The base64-encoded PDF data.
        data: String,
    },

    /// A plain text document.
    #[serde(rename = "text")]
    Text {
        /// The text content of the document.
        text: String,
    },

    /// A document referenced by a URL.
    #[serde(rename = "url")]
    Url {
        /// The URL of the document.
        url: String,
    },
}

impl ContentBlock {
    /// Create a text content block
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    /// Create a tool use content block
    pub fn tool_use(
        id: impl Into<String>,
        name: impl Into<String>,
        input: serde_json::Value,
    ) -> Self {
        Self::ToolUse {
            id: id.into(),
            name: name.into(),
            input,
        }
    }

    /// Create a tool result content block
    pub fn tool_result(tool_use_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self::ToolResult {
            tool_use_id: tool_use_id.into(),
            content: Some(content.into()),
            is_error: None,
        }
    }

    /// Create an error tool result
    pub fn tool_error(tool_use_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self::ToolResult {
            tool_use_id: tool_use_id.into(),
            content: Some(error.into()),
            is_error: Some(true),
        }
    }

    /// Create a thinking content block
    pub fn thinking(thinking: impl Into<String>) -> Self {
        Self::Thinking {
            thinking: thinking.into(),
        }
    }

    /// Get the type name of this content block
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Text { .. } => "text",
            Self::Image { .. } => "image",
            Self::ToolUse { .. } => "tool_use",
            Self::ToolResult { .. } => "tool_result",
            Self::Thinking { .. } => "thinking",
            Self::Document { .. } => "document",
        }
    }

    /// Check if this is a text block
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text { .. })
    }

    /// Check if this is a tool use block
    pub fn is_tool_use(&self) -> bool {
        matches!(self, Self::ToolUse { .. })
    }

    /// Check if this is a tool result block
    pub fn is_tool_result(&self) -> bool {
        matches!(self, Self::ToolResult { .. })
    }

    /// Extract text if this is a text block
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text { text } => Some(text),
            _ => None,
        }
    }

    /// Extract tool use if this is a tool use block
    pub fn as_tool_use(&self) -> Option<(&str, &str, &serde_json::Value)> {
        match self {
            Self::ToolUse { id, name, input } => Some((id, name, input)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_content_serialization() {
        let content = ContentBlock::text("Hello, world!");
        let json = serde_json::to_string(&content).unwrap();
        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(content, deserialized);
    }

    #[test]
    fn test_tool_use_content() {
        let content =
            ContentBlock::tool_use("id_123", "bash", serde_json::json!({ "command": "ls" }));
        assert!(content.is_tool_use());
        assert_eq!(content.type_name(), "tool_use");
    }

    #[test]
    fn test_content_type_checks() {
        let text = ContentBlock::text("test");
        assert!(text.is_text());
        assert!(!text.is_tool_use());
    }
}
