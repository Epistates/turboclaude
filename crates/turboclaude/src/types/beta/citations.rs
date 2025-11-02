//! Citation types for source attribution in responses
//!
//! Citations provide source attribution for text in model responses. The API returns different
//! citation types depending on the source document type:
//!
//! - **Character location**: Plain text documents (char-based indexing)
//! - **Page location**: PDF documents (page-based indexing)
//! - **Content block location**: Structured content documents (block-based indexing)
//! - **Search result location**: Search results from tools
//! - **Web search result location**: Web search results
//!
//! # Example
//!
//! ```rust
//! use turboclaude::types::beta::TextCitation;
//!
//! // Citations are returned in ContentBlock::Text responses
//! // and provide traceability to source documents
//! ```

use serde::{Deserialize, Serialize};

/// A citation providing source attribution for text in a response.
///
/// Citations are tagged unions that vary by document type. Use the `type` field
/// to discriminate between variants during deserialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TextCitation {
    /// Citation with character-level location (plain text documents)
    #[serde(rename = "char_location")]
    CharLocation(CitationCharLocation),

    /// Citation with page-level location (PDF documents)
    #[serde(rename = "page_location")]
    PageLocation(CitationPageLocation),

    /// Citation with content block location (structured documents)
    #[serde(rename = "content_block_location")]
    ContentBlockLocation(CitationContentBlockLocation),

    /// Citation from search results
    #[serde(rename = "search_result_location")]
    SearchResultLocation(CitationSearchResultLocation),

    /// Citation from web search results
    #[serde(rename = "web_search_result_location")]
    WebSearchResultLocation(CitationWebSearchResultLocation),
}

impl TextCitation {
    /// Get the cited text regardless of citation type.
    pub fn cited_text(&self) -> &str {
        match self {
            TextCitation::CharLocation(c) => &c.cited_text,
            TextCitation::PageLocation(c) => &c.cited_text,
            TextCitation::ContentBlockLocation(c) => &c.cited_text,
            TextCitation::SearchResultLocation(c) => &c.cited_text,
            TextCitation::WebSearchResultLocation(c) => &c.cited_text,
        }
    }

    /// Get the document title if available.
    pub fn title(&self) -> Option<&str> {
        match self {
            TextCitation::CharLocation(c) => c.document_title.as_deref(),
            TextCitation::PageLocation(c) => c.document_title.as_deref(),
            TextCitation::ContentBlockLocation(c) => c.document_title.as_deref(),
            TextCitation::SearchResultLocation(c) => c.title.as_deref(),
            TextCitation::WebSearchResultLocation(c) => c.title.as_deref(),
        }
    }
}

/// Citation with character-level location for plain text documents.
///
/// Provides precise character offsets for cited text within the source document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationCharLocation {
    /// The text that was cited from the source document
    pub cited_text: String,

    /// Index of the document in the request (0-based)
    pub document_index: usize,

    /// Optional title of the source document
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_title: Option<String>,

    /// Character index where the citation ends (exclusive)
    pub end_char_index: usize,

    /// Optional file ID if the document was uploaded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,

    /// Character index where the citation starts (inclusive)
    pub start_char_index: usize,
}

/// Citation with page-level location for PDF documents.
///
/// Provides page number ranges for cited text within PDF documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationPageLocation {
    /// The text that was cited from the source document
    pub cited_text: String,

    /// Index of the document in the request (0-based)
    pub document_index: usize,

    /// Optional title of the source document
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_title: Option<String>,

    /// Page number where the citation ends (inclusive, 1-based)
    pub end_page_number: usize,

    /// Optional file ID if the document was uploaded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,

    /// Page number where the citation starts (inclusive, 1-based)
    pub start_page_number: usize,
}

/// Citation with content block location for structured documents.
///
/// Provides block-level indexing for cited text in structured content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationContentBlockLocation {
    /// The text that was cited from the source document
    pub cited_text: String,

    /// Index of the document in the request (0-based)
    pub document_index: usize,

    /// Optional title of the source document
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_title: Option<String>,

    /// Block index where the citation ends (inclusive, 0-based)
    pub end_block_index: usize,

    /// Optional file ID if the document was uploaded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,

    /// Block index where the citation starts (inclusive, 0-based)
    pub start_block_index: usize,
}

/// Citation from search results.
///
/// Provides attribution to search result content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationSearchResultLocation {
    /// The text that was cited from the search result
    pub cited_text: String,

    /// Block index where the citation ends (inclusive, 0-based)
    pub end_block_index: usize,

    /// Index of the search result (0-based)
    pub search_result_index: usize,

    /// Source URL or identifier for the search result
    pub source: String,

    /// Block index where the citation starts (inclusive, 0-based)
    pub start_block_index: usize,

    /// Optional title of the search result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Citation from web search results.
///
/// Provides attribution to web search results with encrypted indexing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationWebSearchResultLocation {
    /// The text that was cited from the web search result
    pub cited_text: String,

    /// Encrypted index for the web search result
    pub encrypted_index: String,

    /// Optional title of the web page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// URL of the web page
    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_citation_char_location_serialization() {
        let citation = CitationCharLocation {
            cited_text: "example text".to_string(),
            document_index: 0,
            document_title: Some("Test Document".to_string()),
            end_char_index: 100,
            file_id: Some("file_123".to_string()),
            start_char_index: 50,
        };

        let json = serde_json::to_value(&citation).unwrap();
        assert_eq!(json["cited_text"], "example text");
        assert_eq!(json["document_index"], 0);
        assert_eq!(json["document_title"], "Test Document");
        assert_eq!(json["start_char_index"], 50);
        assert_eq!(json["end_char_index"], 100);
        assert_eq!(json["file_id"], "file_123");
    }

    #[test]
    fn test_citation_page_location_serialization() {
        let citation = CitationPageLocation {
            cited_text: "page content".to_string(),
            document_index: 1,
            document_title: Some("PDF Document".to_string()),
            end_page_number: 5,
            file_id: None,
            start_page_number: 3,
        };

        let json = serde_json::to_value(&citation).unwrap();
        assert_eq!(json["cited_text"], "page content");
        assert_eq!(json["document_index"], 1);
        assert_eq!(json["start_page_number"], 3);
        assert_eq!(json["end_page_number"], 5);
        assert!(json["file_id"].is_null());
    }

    #[test]
    fn test_citation_content_block_location_serialization() {
        let citation = CitationContentBlockLocation {
            cited_text: "block content".to_string(),
            document_index: 0,
            document_title: None,
            end_block_index: 10,
            file_id: Some("file_456".to_string()),
            start_block_index: 5,
        };

        let json = serde_json::to_value(&citation).unwrap();
        assert_eq!(json["cited_text"], "block content");
        assert_eq!(json["start_block_index"], 5);
        assert_eq!(json["end_block_index"], 10);
        assert!(json["document_title"].is_null());
    }

    #[test]
    fn test_citation_search_result_location_serialization() {
        let citation = CitationSearchResultLocation {
            cited_text: "search result".to_string(),
            end_block_index: 3,
            search_result_index: 0,
            source: "https://example.com".to_string(),
            start_block_index: 1,
            title: Some("Example Page".to_string()),
        };

        let json = serde_json::to_value(&citation).unwrap();
        assert_eq!(json["cited_text"], "search result");
        assert_eq!(json["search_result_index"], 0);
        assert_eq!(json["source"], "https://example.com");
        assert_eq!(json["title"], "Example Page");
    }

    #[test]
    fn test_citation_web_search_result_location_serialization() {
        let citation = CitationWebSearchResultLocation {
            cited_text: "web search result".to_string(),
            encrypted_index: "enc_abc123".to_string(),
            title: Some("Web Page Title".to_string()),
            url: "https://example.org".to_string(),
        };

        let json = serde_json::to_value(&citation).unwrap();
        assert_eq!(json["cited_text"], "web search result");
        assert_eq!(json["encrypted_index"], "enc_abc123");
        assert_eq!(json["url"], "https://example.org");
        assert_eq!(json["title"], "Web Page Title");
    }

    #[test]
    fn test_text_citation_enum_deserialization() {
        // Test char_location
        let json = r#"{
            "type": "char_location",
            "cited_text": "test",
            "document_index": 0,
            "start_char_index": 10,
            "end_char_index": 20
        }"#;
        let citation: TextCitation = serde_json::from_str(json).unwrap();
        match citation {
            TextCitation::CharLocation(c) => {
                assert_eq!(c.cited_text, "test");
                assert_eq!(c.start_char_index, 10);
            }
            _ => panic!("Expected CharLocation variant"),
        }

        // Test page_location
        let json = r#"{
            "type": "page_location",
            "cited_text": "page test",
            "document_index": 0,
            "start_page_number": 1,
            "end_page_number": 2
        }"#;
        let citation: TextCitation = serde_json::from_str(json).unwrap();
        match citation {
            TextCitation::PageLocation(c) => {
                assert_eq!(c.cited_text, "page test");
                assert_eq!(c.start_page_number, 1);
            }
            _ => panic!("Expected PageLocation variant"),
        }

        // Test content_block_location
        let json = r#"{
            "type": "content_block_location",
            "cited_text": "block test",
            "document_index": 0,
            "start_block_index": 0,
            "end_block_index": 1
        }"#;
        let citation: TextCitation = serde_json::from_str(json).unwrap();
        match citation {
            TextCitation::ContentBlockLocation(c) => {
                assert_eq!(c.cited_text, "block test");
                assert_eq!(c.start_block_index, 0);
            }
            _ => panic!("Expected ContentBlockLocation variant"),
        }

        // Test search_result_location
        let json = r#"{
            "type": "search_result_location",
            "cited_text": "search test",
            "start_block_index": 0,
            "end_block_index": 1,
            "search_result_index": 0,
            "source": "https://example.com"
        }"#;
        let citation: TextCitation = serde_json::from_str(json).unwrap();
        match citation {
            TextCitation::SearchResultLocation(c) => {
                assert_eq!(c.cited_text, "search test");
                assert_eq!(c.source, "https://example.com");
            }
            _ => panic!("Expected SearchResultLocation variant"),
        }

        // Test web_search_result_location
        let json = r#"{
            "type": "web_search_result_location",
            "cited_text": "web test",
            "encrypted_index": "enc_123",
            "url": "https://example.org"
        }"#;
        let citation: TextCitation = serde_json::from_str(json).unwrap();
        match citation {
            TextCitation::WebSearchResultLocation(c) => {
                assert_eq!(c.cited_text, "web test");
                assert_eq!(c.url, "https://example.org");
            }
            _ => panic!("Expected WebSearchResultLocation variant"),
        }
    }

    #[test]
    fn test_text_citation_cited_text_helper() {
        let citation = TextCitation::CharLocation(CitationCharLocation {
            cited_text: "helper test".to_string(),
            document_index: 0,
            document_title: None,
            end_char_index: 20,
            file_id: None,
            start_char_index: 10,
        });

        assert_eq!(citation.cited_text(), "helper test");
    }

    #[test]
    fn test_text_citation_title_helper() {
        let citation = TextCitation::PageLocation(CitationPageLocation {
            cited_text: "test".to_string(),
            document_index: 0,
            document_title: Some("Test Title".to_string()),
            end_page_number: 2,
            file_id: None,
            start_page_number: 1,
        });

        assert_eq!(citation.title(), Some("Test Title"));

        let citation_no_title = TextCitation::CharLocation(CitationCharLocation {
            cited_text: "test".to_string(),
            document_index: 0,
            document_title: None,
            end_char_index: 10,
            file_id: None,
            start_char_index: 0,
        });

        assert_eq!(citation_no_title.title(), None);
    }
}
