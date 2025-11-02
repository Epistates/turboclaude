use serde::{Deserialize, Serialize};
use turboclaude_core::serde::SerializePipeline;

/// Example message type for Claude API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Message {
    role: Role,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<Metadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Role {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Metadata {
    timestamp: u64,
    tokens: u32,
    #[serde(default)]
    tags: Vec<String>,
}

/// Example configuration type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Config {
    api_key: String,
    model: String,
    max_tokens: u32,
    temperature: f32,
    #[serde(default)]
    stop_sequences: Vec<String>,
}

/// Example conversation type with nested structures.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Conversation {
    id: String,
    messages: Vec<Message>,
    config: Config,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
}

fn main() {
    println!("=== TurboClaude Core: Serialization Pipeline Examples ===\n");

    // Example 1: Simple message serialization
    println!("Example 1: Simple message serialization");
    let msg = Message {
        role: Role::User,
        content: "Hello, Claude!".to_string(),
        metadata: None,
    };

    let json_str = msg.to_json_string().unwrap();
    println!("  Compact JSON: {}", json_str);

    let pretty = msg.to_json_string_pretty().unwrap();
    println!("  Pretty JSON:\n{}", indent(&pretty, 4));
    println!();

    // Example 2: Message with metadata
    println!("Example 2: Message with metadata");
    let msg_with_meta = Message {
        role: Role::Assistant,
        content: "I'm here to help!".to_string(),
        metadata: Some(Metadata {
            timestamp: 1234567890,
            tokens: 42,
            tags: vec!["greeting".to_string(), "helpful".to_string()],
        }),
    };

    let json_value = msg_with_meta.to_json_value().unwrap();
    println!("  JSON Value:");
    println!("    role: {}", json_value["role"]);
    println!("    content: {}", json_value["content"]);
    println!("    metadata.tokens: {}", json_value["metadata"]["tokens"]);
    println!();

    // Example 3: Configuration serialization
    println!("Example 3: Configuration serialization");
    let config = Config {
        api_key: "sk-ant-api03-xxxx".to_string(),
        model: "claude-3-opus-20240229".to_string(),
        max_tokens: 1024,
        temperature: 0.7,
        stop_sequences: vec!["\n\nHuman:".to_string(), "\n\nAssistant:".to_string()],
    };

    let config_json = config.to_json_string_pretty().unwrap();
    println!("  Config JSON:\n{}", indent(&config_json, 4));
    println!();

    // Example 4: Complex nested structure
    println!("Example 4: Complex nested conversation");
    let conversation = Conversation {
        id: "conv_123456".to_string(),
        messages: vec![
            Message {
                role: Role::System,
                content: "You are a helpful assistant.".to_string(),
                metadata: None,
            },
            Message {
                role: Role::User,
                content: "What is Rust?".to_string(),
                metadata: Some(Metadata {
                    timestamp: 1234567890,
                    tokens: 10,
                    tags: vec!["question".to_string()],
                }),
            },
            Message {
                role: Role::Assistant,
                content: "Rust is a systems programming language...".to_string(),
                metadata: Some(Metadata {
                    timestamp: 1234567895,
                    tokens: 50,
                    tags: vec!["answer".to_string(), "technical".to_string()],
                }),
            },
        ],
        config: config.clone(),
        summary: Some("Discussion about Rust programming language".to_string()),
    };

    let conv_json = conversation.to_json_string_pretty().unwrap();
    println!(
        "  Conversation JSON (truncated):\n{}",
        truncate(&conv_json, 500)
    );
    println!();

    // Example 5: Roundtrip serialization
    println!("Example 5: Roundtrip serialization");
    let original = Message {
        role: Role::User,
        content: "Test message".to_string(),
        metadata: Some(Metadata {
            timestamp: 999,
            tokens: 5,
            tags: vec!["test".to_string()],
        }),
    };

    println!("  Original: {:?}", original);

    // To JSON string
    let json_str = original.to_json_string().unwrap();
    println!("  Serialized: {}", json_str);

    // From JSON string
    let deserialized = Message::from_json_string(&json_str).unwrap();
    println!("  Deserialized: {:?}", deserialized);

    // Verify equality
    assert_eq!(original, deserialized);
    println!("  ✓ Roundtrip successful!");
    println!();

    // Example 6: JSON Value manipulation
    println!("Example 6: JSON Value manipulation");
    let msg = Message {
        role: Role::User,
        content: "Original content".to_string(),
        metadata: None,
    };

    // Convert to JSON Value
    let mut json_value = msg.to_json_value().unwrap();
    println!("  Original value: {}", json_value);

    // Modify the JSON value directly
    if let Some(obj) = json_value.as_object_mut() {
        obj.insert("content".to_string(), serde_json::json!("Modified content"));
        obj.insert("injected_field".to_string(), serde_json::json!(true));
    }
    println!("  Modified value: {}", json_value);

    // Convert back to struct (extra field ignored)
    let modified_msg = Message::from_json_value(json_value).unwrap();
    println!("  Back to struct: {:?}", modified_msg);
    assert_eq!(modified_msg.content, "Modified content");
    println!();

    // Example 7: Error handling
    println!("Example 7: Error handling");

    // Invalid JSON string
    let invalid_json = r#"{"role": "user", "content": "test", invalid}"#;
    match Message::from_json_string(invalid_json) {
        Ok(_) => println!("  Unexpected success"),
        Err(e) => println!("  ✓ Parse error caught: {}", e),
    }

    // Type mismatch
    let type_mismatch = r#"{"role": "user", "content": 123, "metadata": null}"#;
    match Message::from_json_string(type_mismatch) {
        Ok(_) => println!("  Unexpected success"),
        Err(e) => println!("  ✓ Type error caught: {}", e),
    }

    // Missing required field
    let missing_field = r#"{"role": "user"}"#;
    match Message::from_json_string(missing_field) {
        Ok(_) => println!("  Unexpected success"),
        Err(e) => println!("  ✓ Missing field error caught: {}", e),
    }
    println!();

    // Example 8: Performance comparison
    println!("Example 8: Compact vs Pretty");
    let large_conversation = Conversation {
        id: "perf_test".to_string(),
        messages: vec![
            Message {
                role: Role::User,
                content: "A".repeat(100),
                metadata: Some(Metadata {
                    timestamp: 1,
                    tokens: 100,
                    tags: vec!["a".to_string(); 10],
                }),
            };
            10
        ],
        config: Config {
            api_key: "test".to_string(),
            model: "claude-3".to_string(),
            max_tokens: 1024,
            temperature: 0.5,
            stop_sequences: vec![],
        },
        summary: None,
    };

    let compact = large_conversation.to_json_string().unwrap();
    let pretty = large_conversation.to_json_string_pretty().unwrap();

    println!("  Compact size: {} bytes", compact.len());
    println!("  Pretty size:  {} bytes", pretty.len());
    println!(
        "  Overhead:     {:.1}%",
        (pretty.len() as f64 / compact.len() as f64 - 1.0) * 100.0
    );
    println!();

    // Summary
    println!("=== Summary ===");
    println!("SerializePipeline provides a unified interface for JSON operations.");
    println!("All types with #[derive(Serialize, Deserialize)] get it automatically.");
    println!("Methods available:");
    println!("  - to_json_value() / from_json_value()");
    println!("  - to_json_string() / from_json_string()");
    println!("  - to_json_string_pretty()");
    println!("Zero boilerplate, comprehensive error handling included.");
}

// Helper function to indent text
fn indent(text: &str, spaces: usize) -> String {
    let prefix = " ".repeat(spaces);
    text.lines()
        .map(|line| format!("{}{}", prefix, line))
        .collect::<Vec<_>>()
        .join("\n")
}

// Helper function to truncate text
fn truncate(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!(
            "{}...\n    [truncated {} chars]",
            &text[..max_len],
            text.len() - max_len
        )
    }
}
