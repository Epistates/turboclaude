//! Usage statistics

use serde::{Deserialize, Serialize};

/// Token usage statistics for a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// Number of input tokens
    pub input_tokens: u32,

    /// Number of output tokens
    pub output_tokens: u32,

    /// Number of cache creation input tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u32>,

    /// Number of cache read input tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u32>,
}

impl Usage {
    /// Total number of tokens used.
    pub fn total_tokens(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_struct_creation() {
        let usage = Usage {
            input_tokens: 100,
            output_tokens: 50,
            cache_creation_input_tokens: Some(25),
            cache_read_input_tokens: Some(75),
        };

        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.cache_creation_input_tokens, Some(25));
        assert_eq!(usage.cache_read_input_tokens, Some(75));
        assert_eq!(usage.total_tokens(), 150);
    }

    #[test]
    fn test_usage_without_cache() {
        let usage = Usage {
            input_tokens: 200,
            output_tokens: 100,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        };

        assert_eq!(usage.total_tokens(), 300);
        assert!(usage.cache_creation_input_tokens.is_none());
        assert!(usage.cache_read_input_tokens.is_none());
    }

    #[test]
    fn test_usage_serialization() {
        let usage = Usage {
            input_tokens: 150,
            output_tokens: 75,
            cache_creation_input_tokens: Some(30),
            cache_read_input_tokens: None,
        };

        let json = serde_json::to_value(&usage).unwrap();
        assert_eq!(json["input_tokens"], 150);
        assert_eq!(json["output_tokens"], 75);
        assert_eq!(json["cache_creation_input_tokens"], 30);
        // None should be omitted
        assert!(json.get("cache_read_input_tokens").is_none());
    }

    #[test]
    fn test_usage_deserialization() {
        let json = r#"{
            "input_tokens": 250,
            "output_tokens": 125,
            "cache_creation_input_tokens": 50,
            "cache_read_input_tokens": 100
        }"#;

        let usage: Usage = serde_json::from_str(json).unwrap();
        assert_eq!(usage.input_tokens, 250);
        assert_eq!(usage.output_tokens, 125);
        assert_eq!(usage.cache_creation_input_tokens, Some(50));
        assert_eq!(usage.cache_read_input_tokens, Some(100));
    }
}
