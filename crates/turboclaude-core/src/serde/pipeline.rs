use serde::Serialize;
use serde::de::DeserializeOwned;

/// A type that can be serialized to and from JSON values.
///
/// This trait provides a common interface for all protocol types,
/// centralizing error handling and enabling custom serializers.
///
/// # Design Philosophy
///
/// SerializePipeline eliminates repetitive serialization boilerplate by
/// providing a unified interface for JSON operations. All types that implement
/// `Serialize + DeserializeOwned` automatically get these methods via blanket impl.
///
/// This follows the principle of "convention over configuration" - there's one
/// obvious way to serialize/deserialize, and it's always available.
///
/// # Blanket Implementation
///
/// This trait has a blanket implementation for all `T: Serialize + DeserializeOwned`,
/// meaning you don't need to manually implement it. Just derive the required traits:
///
/// ```
/// use serde::{Serialize, Deserialize};
/// use turboclaude_core::serde::SerializePipeline;
///
/// #[derive(Serialize, Deserialize)]
/// struct Message {
///     content: String,
///     count: u32,
/// }
///
/// // SerializePipeline is automatically available!
/// let msg = Message { content: "hello".to_string(), count: 42 };
/// let json_str = msg.to_json_string().unwrap();
/// let roundtrip = Message::from_json_string(&json_str).unwrap();
/// ```
///
/// # Use Cases
///
/// - **Protocol Messages**: Serialize Claude API requests/responses
/// - **Configuration**: Parse config files and write defaults
/// - **Debugging**: Pretty-print complex types for inspection
/// - **Storage**: Save/load application state as JSON
///
/// # Example
///
/// ```
/// use serde::{Serialize, Deserialize};
/// use turboclaude_core::serde::SerializePipeline;
///
/// #[derive(Serialize, Deserialize, Debug)]
/// struct Config {
///     api_key: String,
///     max_tokens: u32,
///     model: String,
/// }
///
/// let config = Config {
///     api_key: "sk-test".to_string(),
///     max_tokens: 1024,
///     model: "claude-3-opus".to_string(),
/// };
///
/// // Serialize to JSON value
/// let json_value = config.to_json_value().unwrap();
/// assert!(json_value.is_object());
///
/// // Serialize to string
/// let json_str = config.to_json_string().unwrap();
/// assert!(json_str.contains("claude-3-opus"));
///
/// // Deserialize back
/// let loaded = Config::from_json_string(&json_str).unwrap();
/// assert_eq!(loaded.model, "claude-3-opus");
///
/// // Pretty print for debugging
/// let pretty = config.to_json_string_pretty().unwrap();
/// println!("{}", pretty);
/// ```
pub trait SerializePipeline: Serialize + DeserializeOwned {
    /// Serialize to a JSON value.
    ///
    /// This is useful when you need to manipulate the JSON structure
    /// before converting to a string, or when working with dynamic JSON.
    ///
    /// # Errors
    ///
    /// Returns `serde_json::Error` if serialization fails (e.g., for
    /// types with non-string map keys, or custom serializers that error).
    ///
    /// # Example
    ///
    /// ```
    /// use serde::{Serialize, Deserialize};
    /// use turboclaude_core::serde::SerializePipeline;
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Point { x: i32, y: i32 }
    ///
    /// let p = Point { x: 10, y: 20 };
    /// let value = p.to_json_value().unwrap();
    ///
    /// assert_eq!(value["x"], 10);
    /// assert_eq!(value["y"], 20);
    /// ```
    fn to_json_value(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }

    /// Deserialize from a JSON value.
    ///
    /// This is the inverse of `to_json_value`, useful when working
    /// with dynamically-constructed JSON.
    ///
    /// # Errors
    ///
    /// Returns `serde_json::Error` if the value doesn't match the
    /// expected type structure.
    ///
    /// # Example
    ///
    /// ```
    /// use serde::{Serialize, Deserialize};
    /// use serde_json::json;
    /// use turboclaude_core::serde::SerializePipeline;
    ///
    /// #[derive(Serialize, Deserialize, PartialEq, Debug)]
    /// struct Point { x: i32, y: i32 }
    ///
    /// let value = json!({"x": 10, "y": 20});
    /// let point = Point::from_json_value(value).unwrap();
    ///
    /// assert_eq!(point, Point { x: 10, y: 20 });
    /// ```
    fn from_json_value(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }

    /// Serialize to a compact JSON string.
    ///
    /// This produces a single-line JSON string with minimal whitespace,
    /// ideal for storage or network transmission.
    ///
    /// # Errors
    ///
    /// Returns `serde_json::Error` if serialization fails.
    ///
    /// # Example
    ///
    /// ```
    /// use serde::{Serialize, Deserialize};
    /// use turboclaude_core::serde::SerializePipeline;
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Point { x: i32, y: i32 }
    ///
    /// let p = Point { x: 10, y: 20 };
    /// let json = p.to_json_string().unwrap();
    ///
    /// assert_eq!(json, r#"{"x":10,"y":20}"#);
    /// ```
    fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from a JSON string.
    ///
    /// This is the most common deserialization method, used when reading
    /// from files, network responses, or configuration.
    ///
    /// # Errors
    ///
    /// Returns `serde_json::Error` if:
    /// - The string is not valid JSON
    /// - The JSON structure doesn't match the expected type
    ///
    /// # Example
    ///
    /// ```
    /// use serde::{Serialize, Deserialize};
    /// use turboclaude_core::serde::SerializePipeline;
    ///
    /// #[derive(Serialize, Deserialize, PartialEq, Debug)]
    /// struct Point { x: i32, y: i32 }
    ///
    /// let json = r#"{"x":10,"y":20}"#;
    /// let point = Point::from_json_string(json).unwrap();
    ///
    /// assert_eq!(point, Point { x: 10, y: 20 });
    /// ```
    fn from_json_string(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Serialize to a pretty-printed JSON string.
    ///
    /// This produces a multi-line JSON string with indentation,
    /// ideal for debugging, logging, or human-readable config files.
    ///
    /// # Errors
    ///
    /// Returns `serde_json::Error` if serialization fails.
    ///
    /// # Example
    ///
    /// ```
    /// use serde::{Serialize, Deserialize};
    /// use turboclaude_core::serde::SerializePipeline;
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Config {
    ///     name: String,
    ///     settings: Settings,
    /// }
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Settings {
    ///     enabled: bool,
    ///     timeout: u32,
    /// }
    ///
    /// let config = Config {
    ///     name: "app".to_string(),
    ///     settings: Settings { enabled: true, timeout: 30 },
    /// };
    ///
    /// let pretty = config.to_json_string_pretty().unwrap();
    /// assert!(pretty.contains("  ")); // Has indentation
    /// assert!(pretty.contains("\n")); // Has newlines
    /// ```
    fn to_json_string_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

// Blanket implementation for all types that are Serialize + DeserializeOwned
// This means any type with #[derive(Serialize, Deserialize)] automatically
// gets all SerializePipeline methods without any additional work.
impl<T> SerializePipeline for T where T: Serialize + DeserializeOwned {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestMessage {
        content: String,
        count: u32,
        nested: Option<TestNested>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestNested {
        value: i32,
        tags: Vec<String>,
    }

    #[test]
    fn test_serialize_pipeline_value_roundtrip() {
        let msg = TestMessage {
            content: "hello world".to_string(),
            count: 42,
            nested: Some(TestNested {
                value: 99,
                tags: vec!["a".to_string(), "b".to_string()],
            }),
        };

        // Serialize to JSON value
        let json_value = msg.to_json_value().unwrap();
        assert!(json_value.is_object());
        assert_eq!(json_value["content"], "hello world");
        assert_eq!(json_value["count"], 42);
        assert_eq!(json_value["nested"]["value"], 99);

        // Deserialize back
        let roundtrip = TestMessage::from_json_value(json_value).unwrap();
        assert_eq!(msg, roundtrip);
    }

    #[test]
    fn test_serialize_pipeline_string_roundtrip() {
        let msg = TestMessage {
            content: "test message".to_string(),
            count: 123,
            nested: None,
        };

        // Serialize to compact string
        let json_str = msg.to_json_string().unwrap();
        assert!(!json_str.contains('\n')); // No newlines in compact format
        assert!(json_str.contains("test message"));

        // Deserialize back
        let roundtrip = TestMessage::from_json_string(&json_str).unwrap();
        assert_eq!(msg, roundtrip);
    }

    #[test]
    fn test_pretty_print() {
        let msg = TestMessage {
            content: "pretty test".to_string(),
            count: 1,
            nested: Some(TestNested {
                value: 42,
                tags: vec!["tag1".to_string(), "tag2".to_string()],
            }),
        };

        let pretty = msg.to_json_string_pretty().unwrap();

        // Verify pretty formatting
        assert!(pretty.contains('\n'), "Should have newlines");
        assert!(pretty.contains("  "), "Should have indentation");
        assert!(pretty.contains("\"content\": \"pretty test\""));
        assert!(pretty.contains("\"count\": 1"));

        // Should still deserialize correctly
        let roundtrip = TestMessage::from_json_string(&pretty).unwrap();
        assert_eq!(msg, roundtrip);
    }

    #[test]
    fn test_empty_optional_fields() {
        let msg = TestMessage {
            content: "minimal".to_string(),
            count: 0,
            nested: None,
        };

        let json_value = msg.to_json_value().unwrap();
        assert_eq!(json_value["nested"], serde_json::Value::Null);

        let roundtrip = TestMessage::from_json_value(json_value).unwrap();
        assert_eq!(msg, roundtrip);
        assert!(roundtrip.nested.is_none());
    }

    #[test]
    fn test_nested_structures() {
        let msg = TestMessage {
            content: "nested test".to_string(),
            count: 999,
            nested: Some(TestNested {
                value: -42,
                tags: vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()],
            }),
        };

        let json_str = msg.to_json_string().unwrap();
        let roundtrip = TestMessage::from_json_string(&json_str).unwrap();

        assert_eq!(msg, roundtrip);
        let nested = roundtrip.nested.unwrap();
        assert_eq!(nested.value, -42);
        assert_eq!(nested.tags.len(), 3);
        assert_eq!(nested.tags[0], "tag1");
    }

    #[test]
    fn test_invalid_json_string() {
        let invalid_json = r#"{"content": "test", invalid}"#;
        let result = TestMessage::from_json_string(invalid_json);

        assert!(result.is_err());
        let err = result.unwrap_err();
        // Just verify it failed - error message format may vary
        let msg = err.to_string();
        assert!(!msg.is_empty(), "Error message should not be empty");
    }

    #[test]
    fn test_type_mismatch() {
        let json = r#"{"content": "test", "count": "not_a_number", "nested": null}"#;
        let result = TestMessage::from_json_string(json);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("invalid type") || err.to_string().contains("expected"));
    }

    #[test]
    fn test_missing_required_field() {
        let json = r#"{"content": "test"}"#; // Missing 'count' field
        let result = TestMessage::from_json_string(json);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("missing field") || err.to_string().contains("count"));
    }

    #[test]
    fn test_extra_fields_ignored() {
        let json = r#"{
            "content": "test",
            "count": 42,
            "nested": null,
            "extra_field": "ignored",
            "another_extra": 123
        }"#;

        // serde ignores unknown fields by default
        let msg = TestMessage::from_json_string(json).unwrap();
        assert_eq!(msg.content, "test");
        assert_eq!(msg.count, 42);
        assert!(msg.nested.is_none());
    }

    #[test]
    fn test_unicode_handling() {
        let msg = TestMessage {
            content: "Hello 世界 🌍".to_string(),
            count: 42,
            nested: Some(TestNested {
                value: 0,
                tags: vec!["tag1".to_string(), "тэг2".to_string(), "タグ3".to_string()],
            }),
        };

        let json_str = msg.to_json_string().unwrap();
        let roundtrip = TestMessage::from_json_string(&json_str).unwrap();

        assert_eq!(msg, roundtrip);
        assert_eq!(roundtrip.content, "Hello 世界 🌍");
    }

    #[test]
    fn test_empty_collections() {
        let msg = TestMessage {
            content: "empty".to_string(),
            count: 0,
            nested: Some(TestNested {
                value: 1,
                tags: vec![], // Empty vector
            }),
        };

        let json_value = msg.to_json_value().unwrap();
        assert!(json_value["nested"]["tags"].is_array());
        assert_eq!(json_value["nested"]["tags"].as_array().unwrap().len(), 0);

        let roundtrip = TestMessage::from_json_value(json_value).unwrap();
        assert_eq!(msg, roundtrip);
    }

    #[test]
    fn test_large_numbers() {
        let msg = TestMessage {
            content: "numbers".to_string(),
            count: u32::MAX,
            nested: Some(TestNested {
                value: i32::MIN,
                tags: vec![],
            }),
        };

        let json_str = msg.to_json_string().unwrap();
        let roundtrip = TestMessage::from_json_string(&json_str).unwrap();

        assert_eq!(msg, roundtrip);
        assert_eq!(roundtrip.count, u32::MAX);
        assert_eq!(roundtrip.nested.unwrap().value, i32::MIN);
    }
}
