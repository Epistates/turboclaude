//! Content block types

use super::CacheControl;
use serde::{Deserialize, Serialize};

/// A content block in a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    /// Text content
    #[serde(rename = "text")]
    Text {
        /// The text content
        text: String,
        /// Optional citations supporting the text block (beta feature)
        #[serde(skip_serializing_if = "Option::is_none")]
        citations: Option<Vec<crate::types::beta::TextCitation>>,
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
            ContentBlock::Text { text, .. } => Some(text.as_str()),
            _ => None,
        }
    }

    /// Get citations if this is a text block with citations (beta feature).
    pub fn citations(&self) -> Option<&[crate::types::beta::TextCitation]> {
        match self {
            ContentBlock::Text {
                citations: Some(citations),
                ..
            } => Some(citations.as_slice()),
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
            ContentBlock::Thinking {
                signature,
                thinking,
            } => Some((signature.as_str(), thinking.as_str())),
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
        Self::URL { url: url.into() }
    }

    /// Create a plain text source.
    pub fn plain_text(text: impl Into<String>) -> Self {
        Self::PlainText { text: text.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_source_conversion() {
        // Base64 PDF
        let base64 = DocumentSource::base64_pdf("JVBERi0xLjQK...");
        match &base64 {
            DocumentSource::Base64PDF { media_type, data } => {
                assert_eq!(media_type, "application/pdf");
                assert_eq!(data, "JVBERi0xLjQK...");
            }
            _ => panic!("Expected Base64PDF variant"),
        }

        let json = serde_json::to_value(&base64).unwrap();
        assert_eq!(json["type"], "base64");
        assert_eq!(json["media_type"], "application/pdf");

        // URL PDF
        let url = DocumentSource::url_pdf("https://example.com/doc.pdf");
        match &url {
            DocumentSource::URL { url: u } => {
                assert_eq!(u, "https://example.com/doc.pdf");
            }
            _ => panic!("Expected URL variant"),
        }

        let json = serde_json::to_value(&url).unwrap();
        assert_eq!(json["type"], "url");
        assert_eq!(json["url"], "https://example.com/doc.pdf");

        // Plain text
        let text = DocumentSource::plain_text("Document content");
        match &text {
            DocumentSource::PlainText { text: t } => {
                assert_eq!(t, "Document content");
            }
            _ => panic!("Expected PlainText variant"),
        }

        let json = serde_json::to_value(&text).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "Document content");
    }

    #[test]
    fn test_vision_content_block() {
        let image_block = ContentBlockParam::Image {
            source: ImageSource::base64("image/png", "iVBORw0KGgoAAAANS..."),
        };

        let json = serde_json::to_value(&image_block).unwrap();
        assert_eq!(json["type"], "image");
        assert_eq!(json["source"]["type"], "base64");
        assert_eq!(json["source"]["media_type"], "image/png");
        assert_eq!(json["source"]["data"], "iVBORw0KGgoAAAANS...");
    }

    #[test]
    fn test_content_block_as_text() {
        let text_block = ContentBlock::Text {
            text: "Hello world".to_string(),
            citations: None,
        };
        assert_eq!(text_block.as_text(), Some("Hello world"));

        let image_block = ContentBlock::Image {
            source: ImageSource::base64("image/png", "data"),
        };
        assert_eq!(image_block.as_text(), None);
    }

    #[test]
    fn test_content_block_as_tool_use() {
        use serde_json::json;

        let tool_block = ContentBlock::ToolUse {
            id: "tool_123".to_string(),
            name: "calculator".to_string(),
            input: json!({"expression": "2+2"}),
        };

        let result = tool_block.as_tool_use();
        assert!(result.is_some());

        let (id, name, input) = result.unwrap();
        assert_eq!(id, "tool_123");
        assert_eq!(name, "calculator");
        assert_eq!(input["expression"], "2+2");

        // Test non-tool-use block
        let text_block = ContentBlock::Text {
            text: "Not a tool".to_string(),
            citations: None,
        };
        assert!(text_block.as_tool_use().is_none());
    }

    #[test]
    fn test_image_source_base64() {
        let source = ImageSource::base64("image/jpeg", "base64data");
        assert_eq!(source.source_type, "base64");
        assert_eq!(source.media_type, "image/jpeg");
        assert_eq!(source.data, "base64data");

        let json = serde_json::to_value(&source).unwrap();
        assert_eq!(json["type"], "base64");
        assert_eq!(json["media_type"], "image/jpeg");
        assert_eq!(json["data"], "base64data");
    }

    #[test]
    fn test_content_block_with_citations() {
        use crate::types::beta::{CitationCharLocation, TextCitation};

        let citation = TextCitation::CharLocation(CitationCharLocation {
            cited_text: "cited text".to_string(),
            document_index: 0,
            document_title: Some("Document".to_string()),
            end_char_index: 50,
            file_id: None,
            start_char_index: 20,
        });

        let text_block = ContentBlock::Text {
            text: "Text with citation".to_string(),
            citations: Some(vec![citation]),
        };

        // Test as_text works with citations
        assert_eq!(text_block.as_text(), Some("Text with citation"));

        // Test citations() helper
        let citations = text_block.citations().unwrap();
        assert_eq!(citations.len(), 1);
        assert_eq!(citations[0].cited_text(), "cited text");
    }

    #[test]
    fn test_content_block_citations_serialization() {
        use crate::types::beta::{CitationPageLocation, TextCitation};

        let citation = TextCitation::PageLocation(CitationPageLocation {
            cited_text: "page citation".to_string(),
            document_index: 0,
            document_title: Some("PDF Doc".to_string()),
            end_page_number: 5,
            file_id: Some("file_123".to_string()),
            start_page_number: 3,
        });

        let text_block = ContentBlock::Text {
            text: "Text from PDF".to_string(),
            citations: Some(vec![citation]),
        };

        let json = serde_json::to_value(&text_block).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "Text from PDF");
        assert!(json["citations"].is_array());
        assert_eq!(json["citations"][0]["type"], "page_location");
        assert_eq!(json["citations"][0]["cited_text"], "page citation");
    }

    #[test]
    fn test_content_block_without_citations() {
        let text_block = ContentBlock::Text {
            text: "No citations".to_string(),
            citations: None,
        };

        assert!(text_block.citations().is_none());

        // Verify citations field is omitted in serialization
        let json = serde_json::to_value(&text_block).unwrap();
        assert!(json["citations"].is_null());
    }
}
