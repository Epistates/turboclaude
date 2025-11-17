//! Core types for the Anthropic API
//!
//! This module contains all the type definitions used throughout the SDK,
//! mirroring the Python SDK's type system but adapted for Rust's type safety.

// Re-export commonly used types from submodules
pub use batch::*;
pub use cache::*;
pub use content::*;
pub use message::*;
pub use tool::*;
pub use usage::*;

// Re-export model types from protocol crate
pub use turboclaude_protocol::types::models;
pub use turboclaude_protocol::types::Model;

// Submodules
pub mod batch;
pub mod cache;
pub mod content;
pub mod message;
pub mod tool;
pub mod usage;

/// Beta/experimental API types
pub mod beta;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let user_msg = Message::user("Hello, Claude!");
        assert_eq!(user_msg.role, Role::User);
        assert_eq!(user_msg.content.len(), 1);

        let assistant_msg = Message::assistant("Hello! How can I help you?");
        assert_eq!(assistant_msg.role, Role::Assistant);
        assert_eq!(assistant_msg.content.len(), 1);
    }

    #[test]
    fn test_content_block() {
        let text_block = ContentBlock::Text {
            text: "Hello".to_string(),
            citations: None,
        };
        assert_eq!(text_block.as_text(), Some("Hello"));

        let tool_block = ContentBlock::ToolUse {
            id: "tool_123".to_string(),
            name: "calculator".to_string(),
            input: serde_json::json!({"a": 1, "b": 2}),
        };
        assert!(tool_block.as_tool_use().is_some());
    }

    #[test]
    fn test_tool_choice() {
        let auto = ToolChoice::default();
        assert!(matches!(auto, ToolChoice::Auto));

        let specific = ToolChoice::specific("calculator");
        match specific {
            ToolChoice::Tool { name, .. } => assert_eq!(name, "calculator"),
            _ => panic!("Expected Tool variant"),
        }
    }

    #[test]
    fn test_usage_total() {
        let usage = Usage {
            input_tokens: 100,
            output_tokens: 50,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        };
        assert_eq!(usage.total_tokens(), 150);
    }

    #[test]
    fn test_cache_control_serialization() {
        // Ephemeral with default TTL
        let ephemeral = CacheControl::ephemeral();
        let json = serde_json::to_value(ephemeral).unwrap();
        assert_eq!(json["type"], "ephemeral");
        assert!(json.get("ttl").is_none());

        // Ephemeral with 5m TTL
        let with_ttl = CacheControl::ephemeral_with_ttl(CacheTTL::FiveMinutes);
        let json = serde_json::to_value(with_ttl).unwrap();
        assert_eq!(json["type"], "ephemeral");
        assert_eq!(json["ttl"], "5m");

        // Ephemeral with 1h TTL
        let with_ttl = CacheControl::ephemeral_with_ttl(CacheTTL::OneHour);
        let json = serde_json::to_value(with_ttl).unwrap();
        assert_eq!(json["type"], "ephemeral");
        assert_eq!(json["ttl"], "1h");
    }

    #[test]
    fn test_system_prompt_block_serialization() {
        // Text block without caching
        let text = SystemPromptBlock::text("System prompt");
        let json = serde_json::to_value(text).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "System prompt");
        assert!(json.get("cache_control").is_none());

        // Cached text block
        let cached = SystemPromptBlock::text_cached("Cached system prompt");
        let json = serde_json::to_value(cached).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "Cached system prompt");
        assert_eq!(json["cache_control"]["type"], "ephemeral");

        // Cached with TTL
        let cached_ttl = SystemPromptBlock::text_cached_with_ttl("Cached 1h", CacheTTL::OneHour);
        let json = serde_json::to_value(cached_ttl).unwrap();
        assert_eq!(json["cache_control"]["ttl"], "1h");
    }

    #[test]
    fn test_system_prompt_serialization() {
        // String variant
        let string_prompt = SystemPrompt::from("Simple string");
        let json = serde_json::to_value(string_prompt).unwrap();
        assert_eq!(json, "Simple string");

        // Blocks variant
        let blocks = vec![
            SystemPromptBlock::text("Part 1"),
            SystemPromptBlock::text_cached("Part 2 (cached)"),
        ];
        let blocks_prompt = SystemPrompt::from(blocks);
        let json = serde_json::to_value(blocks_prompt).unwrap();
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 2);
        assert_eq!(json[0]["text"], "Part 1");
        assert_eq!(json[1]["text"], "Part 2 (cached)");
        assert_eq!(json[1]["cache_control"]["type"], "ephemeral");
    }

    #[test]
    fn test_message_request_with_cached_system() {
        let request = MessageRequest::builder()
            .model(models::CLAUDE_3_5_SONNET_20241022)
            .max_tokens(1024u32)
            .messages(vec![Message::user("Hello")])
            .system(vec![
                SystemPromptBlock::text("You are a helpful assistant."),
                SystemPromptBlock::text_cached("This part is cached."),
            ])
            .build()
            .unwrap();

        let json = serde_json::to_value(&request).unwrap();
        assert!(json["system"].is_array());
        assert_eq!(json["system"][0]["type"], "text");
        assert_eq!(json["system"][1]["cache_control"]["type"], "ephemeral");
    }

    #[test]
    fn test_cache_ttl_default() {
        let default_ttl = CacheTTL::default();
        assert_eq!(default_ttl, CacheTTL::FiveMinutes);
    }

    #[test]
    fn test_document_source_serialization() {
        // Base64 PDF
        let base64_pdf = DocumentSource::base64_pdf("JVBERi0xLjQK...");
        let json = serde_json::to_value(base64_pdf).unwrap();
        assert_eq!(json["type"], "base64");
        assert_eq!(json["media_type"], "application/pdf");
        assert_eq!(json["data"], "JVBERi0xLjQK...");

        // URL PDF
        let url_pdf = DocumentSource::url_pdf("https://example.com/doc.pdf");
        let json = serde_json::to_value(url_pdf).unwrap();
        assert_eq!(json["type"], "url");
        assert_eq!(json["url"], "https://example.com/doc.pdf");

        // Plain text
        let plain_text = DocumentSource::plain_text("Document content here");
        let json = serde_json::to_value(plain_text).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "Document content here");
    }

    #[test]
    fn test_document_content_block() {
        // Document without caching
        let doc_block = ContentBlockParam::Document {
            source: DocumentSource::base64_pdf("JVBERi0xLjQK..."),
            cache_control: None,
            title: Some("Technical Specifications".to_string()),
            context: Some("Product documentation".to_string()),
        };

        let json = serde_json::to_value(doc_block).unwrap();
        assert_eq!(json["type"], "document");
        assert_eq!(json["source"]["type"], "base64");
        assert_eq!(json["title"], "Technical Specifications");
        assert_eq!(json["context"], "Product documentation");
        assert!(json.get("cache_control").is_none());

        // Cached document
        let cached_doc = ContentBlockParam::Document {
            source: DocumentSource::url_pdf("https://example.com/spec.pdf"),
            cache_control: Some(CacheControl::ephemeral()),
            title: Some("Cached Spec".to_string()),
            context: None,
        };

        let json = serde_json::to_value(cached_doc).unwrap();
        assert_eq!(json["cache_control"]["type"], "ephemeral");
        assert!(json.get("context").is_none());
    }

    #[test]
    fn test_message_request_with_document() {
        let request = MessageRequest::builder()
            .model(models::CLAUDE_3_5_SONNET_20241022)
            .max_tokens(1024u32)
            .messages(vec![MessageParam {
                role: Role::User,
                content: vec![
                    ContentBlockParam::Text {
                        text: "Summarize this document".to_string(),
                    },
                    ContentBlockParam::Document {
                        source: DocumentSource::base64_pdf("JVBERi0xLjQK..."),
                        cache_control: Some(CacheControl::ephemeral_with_ttl(CacheTTL::OneHour)),
                        title: Some("Research Paper".to_string()),
                        context: Some("Academic research on AI".to_string()),
                    },
                ],
            }])
            .build()
            .unwrap();

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["messages"][0]["content"][0]["type"], "text");
        assert_eq!(json["messages"][0]["content"][1]["type"], "document");
        assert_eq!(
            json["messages"][0]["content"][1]["source"]["type"],
            "base64"
        );
        assert_eq!(
            json["messages"][0]["content"][1]["cache_control"]["ttl"],
            "1h"
        );
    }

    #[test]
    fn test_message_request_with_thinking() {
        use crate::types::beta::ThinkingConfig;

        let request = MessageRequest::builder()
            .model("claude-sonnet-4-5-20250929")
            .max_tokens(3200u32)
            .messages(vec![MessageParam {
                role: Role::User,
                content: vec![ContentBlockParam::Text {
                    text: "Create a haiku".to_string(),
                }],
            }])
            .thinking(ThinkingConfig::new(1600))
            .build()
            .unwrap();

        assert_eq!(request.model, "claude-sonnet-4-5-20250929");
        assert_eq!(request.max_tokens, 3200);
        assert!(request.thinking.is_some());

        if let Some(thinking) = &request.thinking {
            assert_eq!(thinking.budget_tokens, 1600);
            assert_eq!(thinking.config_type, "enabled");
        }

        // Test serialization
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"thinking\""));
        assert!(json.contains("\"budget_tokens\":1600"));
        assert!(json.contains("\"type\":\"enabled\""));
    }

    #[test]
    fn test_content_block_thinking() {
        let block = ContentBlock::Thinking {
            signature: "sig123".to_string(),
            thinking: "Let me think about this...".to_string(),
        };

        assert!(matches!(block, ContentBlock::Thinking { .. }));

        let (sig, thought) = block.as_thinking().unwrap();
        assert_eq!(sig, "sig123");
        assert_eq!(thought, "Let me think about this...");

        // Test serialization
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"type\":\"thinking\""));
        assert!(json.contains("\"signature\":\"sig123\""));
        assert!(json.contains("\"thinking\":\"Let me think about this...\""));
    }
}
