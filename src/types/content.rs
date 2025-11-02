//! Content block types

use serde::{Deserialize, Serialize};
use super::CacheControl;

/// A content block in a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    /// Text content
    #[serde(rename = "text")]
    Text {
        /// The text content
        text: String,
    },

    /// Image content
    #[serde(rename = "image")]
    Image {
        /// Image source
        source: ImageSource,
    },

    /// Tool use request
    #[serde(rename = "tool_use")]
    ToolUse {
        /// Unique identifier for this tool use
        id: String,
        /// Name of the tool
        name: String,
        /// Input parameters for the tool
        input: serde_json::Value,
    },

    /// Tool result
    #[serde(rename = "tool_result")]
    ToolResult {
        /// ID of the tool use this is responding to
        tool_use_id: String,
        /// Result content
        content: String,
        /// Whether the tool call was successful
        is_error: Option<bool>,
    },

    /// Thinking block (beta feature - extended thinking)
    #[serde(rename = "thinking")]
    Thinking {
        /// Signature identifying the thinking block
        signature: String,
        /// The model's reasoning/thinking process
        thinking: String,
    },
}

impl ContentBlock {
    /// Get text content if this is a text block.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            ContentBlock::Text { text } => Some(text.as_str()),
            _ => None,
        }
    }

    /// Get tool use if this is a tool use block.
    pub fn as_tool_use(&self) -> Option<(&str, &str, &serde_json::Value)> {
        match self {
            ContentBlock::ToolUse { id, name, input } => Some((id.as_str(), name.as_str(), input)),
            _ => None,
        }
    }

    /// Get thinking content if this is a thinking block (beta feature).
    pub fn as_thinking(&self) -> Option<(&str, &str)> {
        match self {
            ContentBlock::Thinking { signature, thinking } => Some((signature.as_str(), thinking.as_str())),
            _ => None,
        }
    }
}

/// Parameters for creating a content block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlockParam {
    /// Text content
    #[serde(rename = "text")]
    Text {
        /// The text content
        text: String,
    },

    /// Image content
    #[serde(rename = "image")]
    Image {
        /// Image source
        source: ImageSource,
    },

    /// Tool result
    #[serde(rename = "tool_result")]
    ToolResult {
        /// ID of the tool use this is responding to
        tool_use_id: String,
        /// Result content
        content: String,
        /// Whether the tool call was an error
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },

    /// Document content (PDF, plain text, etc.)
    #[serde(rename = "document")]
    Document {
        /// Document source
        source: DocumentSource,
        /// Optional cache control for the document
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
        /// Optional title for the document
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        /// Optional context about the document
        #[serde(skip_serializing_if = "Option::is_none")]
        context: Option<String>,
    },
}

/// Source for an image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    /// Type of the source (always "base64" for now)
    #[serde(rename = "type")]
    pub source_type: String,

    /// Media type of the image
    pub media_type: String,

    /// Base64-encoded image data
    pub data: String,
}

impl ImageSource {
    /// Create a new base64 image source.
    pub fn base64(media_type: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            source_type: "base64".to_string(),
            media_type: media_type.into(),
            data: data.into(),
        }
    }
}

/// Source for a document (PDF, plain text, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DocumentSource {
    /// Base64-encoded PDF document
    #[serde(rename = "base64")]
    Base64PDF {
        /// Media type (must be "application/pdf")
        media_type: String,
        /// Base64-encoded PDF data
        data: String,
    },

    /// PDF from a URL
    #[serde(rename = "url")]
    URL {
        /// URL to the PDF document
        url: String,
    },

    /// Plain text document
    #[serde(rename = "text")]
    PlainText {
        /// The text content
        text: String,
    },
}

impl DocumentSource {
    /// Create a base64-encoded PDF source.
    pub fn base64_pdf(data: impl Into<String>) -> Self {
        Self::Base64PDF {
            media_type: "application/pdf".to_string(),
            data: data.into(),
        }
    }

    /// Create a URL-based PDF source.
    pub fn url_pdf(url: impl Into<String>) -> Self {
        Self::URL {
            url: url.into(),
        }
    }

    /// Create a plain text source.
    pub fn plain_text(text: impl Into<String>) -> Self {
        Self::PlainText {
            text: text.into(),
        }
    }
}
