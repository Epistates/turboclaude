//! Integration tests for citation support in the turboclaude SDK.
//!
//! These tests verify that citations are properly parsed from API responses
//! and that all citation types work correctly.

use turboclaude::types::beta::{CitationCharLocation, CitationPageLocation, TextCitation};
use turboclaude::types::{ContentBlock, Message, Role};

#[test]
fn test_citation_char_location_deserialization() {
    let json = r#"{
        "type": "char_location",
        "cited_text": "This is cited text",
        "document_index": 0,
        "document_title": "Test Document",
        "start_char_index": 10,
        "end_char_index": 28,
        "file_id": "file_abc123"
    }"#;

    let citation: TextCitation = serde_json::from_str(json).unwrap();

    match &citation {
        TextCitation::CharLocation(loc) => {
            assert_eq!(loc.cited_text, "This is cited text");
            assert_eq!(loc.document_index, 0);
            assert_eq!(loc.document_title, Some("Test Document".to_string()));
            assert_eq!(loc.start_char_index, 10);
            assert_eq!(loc.end_char_index, 28);
            assert_eq!(loc.file_id, Some("file_abc123".to_string()));
        }
        _ => panic!("Expected CharLocation variant"),
    }

    // Test helper methods
    assert_eq!(citation.cited_text(), "This is cited text");
    assert_eq!(citation.title(), Some("Test Document"));
}

#[test]
fn test_citation_page_location_deserialization() {
    let json = r#"{
        "type": "page_location",
        "cited_text": "Content from PDF",
        "document_index": 1,
        "document_title": "Research Paper",
        "start_page_number": 3,
        "end_page_number": 5
    }"#;

    let citation: TextCitation = serde_json::from_str(json).unwrap();

    match citation {
        TextCitation::PageLocation(loc) => {
            assert_eq!(loc.cited_text, "Content from PDF");
            assert_eq!(loc.document_index, 1);
            assert_eq!(loc.document_title, Some("Research Paper".to_string()));
            assert_eq!(loc.start_page_number, 3);
            assert_eq!(loc.end_page_number, 5);
            assert_eq!(loc.file_id, None);
        }
        _ => panic!("Expected PageLocation variant"),
    }
}

#[test]
fn test_citation_content_block_location_deserialization() {
    let json = r#"{
        "type": "content_block_location",
        "cited_text": "Structured content",
        "document_index": 0,
        "start_block_index": 2,
        "end_block_index": 5
    }"#;

    let citation: TextCitation = serde_json::from_str(json).unwrap();

    match citation {
        TextCitation::ContentBlockLocation(loc) => {
            assert_eq!(loc.cited_text, "Structured content");
            assert_eq!(loc.document_index, 0);
            assert_eq!(loc.start_block_index, 2);
            assert_eq!(loc.end_block_index, 5);
        }
        _ => panic!("Expected ContentBlockLocation variant"),
    }
}

#[test]
fn test_citation_search_result_location_deserialization() {
    let json = r#"{
        "type": "search_result_location",
        "cited_text": "Search result text",
        "search_result_index": 0,
        "source": "https://example.com",
        "title": "Example Page",
        "start_block_index": 0,
        "end_block_index": 2
    }"#;

    let citation: TextCitation = serde_json::from_str(json).unwrap();

    match citation {
        TextCitation::SearchResultLocation(loc) => {
            assert_eq!(loc.cited_text, "Search result text");
            assert_eq!(loc.search_result_index, 0);
            assert_eq!(loc.source, "https://example.com");
            assert_eq!(loc.title, Some("Example Page".to_string()));
            assert_eq!(loc.start_block_index, 0);
            assert_eq!(loc.end_block_index, 2);
        }
        _ => panic!("Expected SearchResultLocation variant"),
    }
}

#[test]
fn test_citation_web_search_result_location_deserialization() {
    let json = r#"{
        "type": "web_search_result_location",
        "cited_text": "Web search text",
        "encrypted_index": "enc_xyz789",
        "url": "https://web-example.org",
        "title": "Web Page"
    }"#;

    let citation: TextCitation = serde_json::from_str(json).unwrap();

    match citation {
        TextCitation::WebSearchResultLocation(loc) => {
            assert_eq!(loc.cited_text, "Web search text");
            assert_eq!(loc.encrypted_index, "enc_xyz789");
            assert_eq!(loc.url, "https://web-example.org");
            assert_eq!(loc.title, Some("Web Page".to_string()));
        }
        _ => panic!("Expected WebSearchResultLocation variant"),
    }
}

#[test]
fn test_message_with_citations() {
    let json = r#"{
        "id": "msg_123",
        "type": "message",
        "role": "assistant",
        "model": "claude-3-5-sonnet-20241022",
        "stop_reason": "end_turn",
        "content": [
            {
                "type": "text",
                "text": "According to the document, the answer is clear.",
                "citations": [
                    {
                        "type": "char_location",
                        "cited_text": "the answer is clear",
                        "document_index": 0,
                        "document_title": "Source Document",
                        "start_char_index": 50,
                        "end_char_index": 69
                    }
                ]
            }
        ],
        "usage": {
            "input_tokens": 100,
            "output_tokens": 50
        }
    }"#;

    let message: Message = serde_json::from_str(json).unwrap();

    assert_eq!(message.id, "msg_123");
    assert_eq!(message.role, Role::Assistant);
    assert_eq!(message.content.len(), 1);

    match &message.content[0] {
        ContentBlock::Text { text, citations } => {
            assert_eq!(text, "According to the document, the answer is clear.");
            assert!(citations.is_some());

            let citations = citations.as_ref().unwrap();
            assert_eq!(citations.len(), 1);

            match &citations[0] {
                TextCitation::CharLocation(loc) => {
                    assert_eq!(loc.cited_text, "the answer is clear");
                    assert_eq!(loc.document_title, Some("Source Document".to_string()));
                }
                _ => panic!("Expected CharLocation"),
            }
        }
        _ => panic!("Expected Text content block"),
    }
}

#[test]
fn test_message_with_multiple_citations() {
    let json = r#"{
        "id": "msg_456",
        "type": "message",
        "role": "assistant",
        "model": "claude-3-5-sonnet-20241022",
        "stop_reason": "end_turn",
        "content": [
            {
                "type": "text",
                "text": "Based on multiple sources, we can conclude X.",
                "citations": [
                    {
                        "type": "page_location",
                        "cited_text": "source one",
                        "document_index": 0,
                        "start_page_number": 1,
                        "end_page_number": 1
                    },
                    {
                        "type": "page_location",
                        "cited_text": "source two",
                        "document_index": 1,
                        "start_page_number": 5,
                        "end_page_number": 6
                    }
                ]
            }
        ],
        "usage": {
            "input_tokens": 200,
            "output_tokens": 75
        }
    }"#;

    let message: Message = serde_json::from_str(json).unwrap();

    match &message.content[0] {
        ContentBlock::Text { citations, .. } => {
            let citations = citations.as_ref().unwrap();
            assert_eq!(citations.len(), 2);

            // Verify both citations
            assert_eq!(citations[0].cited_text(), "source one");
            assert_eq!(citations[1].cited_text(), "source two");
        }
        _ => panic!("Expected Text content block"),
    }
}

#[test]
fn test_message_without_citations() {
    let json = r#"{
        "id": "msg_789",
        "type": "message",
        "role": "assistant",
        "model": "claude-3-5-sonnet-20241022",
        "stop_reason": "end_turn",
        "content": [
            {
                "type": "text",
                "text": "This is a response without citations."
            }
        ],
        "usage": {
            "input_tokens": 50,
            "output_tokens": 25
        }
    }"#;

    let message: Message = serde_json::from_str(json).unwrap();

    match &message.content[0] {
        ContentBlock::Text { text, citations } => {
            assert_eq!(text, "This is a response without citations.");
            assert!(citations.is_none());
        }
        _ => panic!("Expected Text content block"),
    }
}

#[test]
fn test_content_block_citations_helper() {
    let citation = TextCitation::CharLocation(CitationCharLocation {
        cited_text: "helper test".to_string(),
        document_index: 0,
        document_title: Some("Test".to_string()),
        start_char_index: 0,
        end_char_index: 11,
        file_id: None,
    });

    let block = ContentBlock::Text {
        text: "Text with citation".to_string(),
        citations: Some(vec![citation]),
    };

    // Test the citations() helper method
    let citations = block.citations().unwrap();
    assert_eq!(citations.len(), 1);
    assert_eq!(citations[0].cited_text(), "helper test");
    assert_eq!(citations[0].title(), Some("Test"));

    // Test block without citations
    let block_no_citations = ContentBlock::Text {
        text: "No citations".to_string(),
        citations: None,
    };
    assert!(block_no_citations.citations().is_none());
}

#[test]
fn test_citation_serialization_roundtrip() {
    let original = CitationPageLocation {
        cited_text: "roundtrip test".to_string(),
        document_index: 2,
        document_title: Some("Roundtrip Doc".to_string()),
        start_page_number: 10,
        end_page_number: 15,
        file_id: Some("file_roundtrip".to_string()),
    };

    let citation = TextCitation::PageLocation(original.clone());

    // Serialize to JSON
    let json = serde_json::to_value(&citation).unwrap();
    assert_eq!(json["type"], "page_location");
    assert_eq!(json["cited_text"], "roundtrip test");
    assert_eq!(json["start_page_number"], 10);

    // Deserialize back
    let deserialized: TextCitation = serde_json::from_value(json).unwrap();

    match deserialized {
        TextCitation::PageLocation(loc) => {
            assert_eq!(loc.cited_text, original.cited_text);
            assert_eq!(loc.document_index, original.document_index);
            assert_eq!(loc.start_page_number, original.start_page_number);
            assert_eq!(loc.end_page_number, original.end_page_number);
        }
        _ => panic!("Expected PageLocation after roundtrip"),
    }
}

#[test]
fn test_citation_without_optional_fields() {
    // Test citations with minimal fields (no optional document_title, file_id, etc.)
    let json = r#"{
        "type": "char_location",
        "cited_text": "minimal citation",
        "document_index": 0,
        "start_char_index": 0,
        "end_char_index": 16
    }"#;

    let citation: TextCitation = serde_json::from_str(json).unwrap();

    match &citation {
        TextCitation::CharLocation(loc) => {
            assert_eq!(loc.cited_text, "minimal citation");
            assert_eq!(loc.document_title, None);
            assert_eq!(loc.file_id, None);
        }
        _ => panic!("Expected CharLocation"),
    }

    assert_eq!(citation.title(), None);
}

#[test]
fn test_all_citation_types_in_one_message() {
    // Test a message with all different citation types
    let json = r#"{
        "id": "msg_all_types",
        "type": "message",
        "role": "assistant",
        "model": "claude-3-5-sonnet-20241022",
        "stop_reason": "end_turn",
        "content": [
            {
                "type": "text",
                "text": "This response uses all citation types.",
                "citations": [
                    {
                        "type": "char_location",
                        "cited_text": "char",
                        "document_index": 0,
                        "start_char_index": 0,
                        "end_char_index": 4
                    },
                    {
                        "type": "page_location",
                        "cited_text": "page",
                        "document_index": 1,
                        "start_page_number": 1,
                        "end_page_number": 1
                    },
                    {
                        "type": "content_block_location",
                        "cited_text": "block",
                        "document_index": 2,
                        "start_block_index": 0,
                        "end_block_index": 1
                    },
                    {
                        "type": "search_result_location",
                        "cited_text": "search",
                        "search_result_index": 0,
                        "source": "https://example.com",
                        "start_block_index": 0,
                        "end_block_index": 1
                    },
                    {
                        "type": "web_search_result_location",
                        "cited_text": "web",
                        "encrypted_index": "enc_123",
                        "url": "https://web.example.com"
                    }
                ]
            }
        ],
        "usage": {
            "input_tokens": 300,
            "output_tokens": 100
        }
    }"#;

    let message: Message = serde_json::from_str(json).unwrap();

    match &message.content[0] {
        ContentBlock::Text { citations, .. } => {
            let citations = citations.as_ref().unwrap();
            assert_eq!(citations.len(), 5);

            // Verify each type
            assert!(matches!(citations[0], TextCitation::CharLocation(_)));
            assert!(matches!(citations[1], TextCitation::PageLocation(_)));
            assert!(matches!(
                citations[2],
                TextCitation::ContentBlockLocation(_)
            ));
            assert!(matches!(
                citations[3],
                TextCitation::SearchResultLocation(_)
            ));
            assert!(matches!(
                citations[4],
                TextCitation::WebSearchResultLocation(_)
            ));

            // Verify cited text for each
            assert_eq!(citations[0].cited_text(), "char");
            assert_eq!(citations[1].cited_text(), "page");
            assert_eq!(citations[2].cited_text(), "block");
            assert_eq!(citations[3].cited_text(), "search");
            assert_eq!(citations[4].cited_text(), "web");
        }
        _ => panic!("Expected Text content block"),
    }
}
