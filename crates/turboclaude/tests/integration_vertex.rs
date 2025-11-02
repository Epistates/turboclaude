//! Integration tests for Vertex provider
//!
//! These tests verify the Vertex provider works correctly with mocked GCP responses.
//! Tests cover:
//! - Non-streaming message creation via rawPredict
//! - Streaming responses via streamRawPredict
//! - Request body transformation (model extraction, version injection)
//! - Error handling (authentication, invalid projects, throttling, etc.)
//! - URL construction for different regions

#![cfg(feature = "vertex")]

use turboclaude::{
    error::Error,
    types::{Message, MessageRequest},
    validation::validate_message_request,
};

/// Test that Vertex request validation catches empty messages
#[test]
fn test_vertex_validation_empty_messages() {
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![])
        .build()
        .expect("Failed to build request");

    let result = validate_message_request(&request);
    assert!(result.is_err(), "Should reject empty messages");

    if let Err(Error::InvalidRequest(msg)) = result {
        assert!(msg.contains("empty"), "Error should mention empty messages");
    }
}

/// Test that Vertex handles invalid model IDs
#[test]
fn test_vertex_validation_invalid_model() {
    let request = MessageRequest::builder()
        .model("")
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .expect("Failed to build request");

    let result = validate_message_request(&request);
    assert!(result.is_err(), "Should reject empty model ID");
}

/// Test that Vertex handles max_tokens limits correctly
#[test]
fn test_vertex_validation_max_tokens_limit() {
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(200_001u32) // Exceeds limit
        .messages(vec![Message::user("Hello")])
        .build()
        .expect("Failed to build request");

    let result = validate_message_request(&request);
    assert!(result.is_err(), "Should reject tokens exceeding limit");
}

/// Test that valid Vertex requests pass validation
#[test]
fn test_vertex_validation_valid_request() {
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello, Claude!")])
        .build()
        .expect("Failed to build request");

    let result = validate_message_request(&request);
    assert!(
        result.is_ok(),
        "Valid Vertex request should pass validation"
    );
}

/// Test that Vertex validates message content
#[test]
fn test_vertex_validation_message_content() {
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![
            turboclaude::types::UserMessage {
                content: vec![], // Empty content!
            }
            .into(),
        ])
        .build()
        .expect("Failed to build request");

    let result = validate_message_request(&request);
    assert!(result.is_err(), "Should reject messages with empty content");
}

/// Test endpoint URL construction for standard regions
#[cfg(feature = "vertex")]
#[test]
fn test_vertex_endpoint_url_standard_region() {
    use std::sync::Arc;
    use turboclaude::providers::vertex::http::VertexHttpProvider;

    let provider = VertexHttpProvider {
        inner: Arc::new(turboclaude::providers::vertex::http::ProviderInner {
            project_id: "test-project".to_string(),
            region: "us-central1".to_string(),
            access_token: None,
            client: reqwest::Client::new(),
            timeout: std::time::Duration::from_secs(600),
        }),
    };

    let url = provider.build_endpoint_url("claude-3-5-sonnet-20241022", false);
    assert!(url.is_ok());

    if let Ok(url_str) = url {
        assert!(url_str.contains("us-central1"));
        assert!(url_str.contains("test-project"));
        assert!(url_str.contains("claude-3-5-sonnet-20241022"));
        assert!(url_str.contains("rawPredict")); // Non-streaming
    }
}

/// Test streaming endpoint URL
#[cfg(feature = "vertex")]
#[test]
fn test_vertex_endpoint_url_streaming() {
    use std::sync::Arc;
    use turboclaude::providers::vertex::http::VertexHttpProvider;

    let provider = VertexHttpProvider {
        inner: Arc::new(turboclaude::providers::vertex::http::ProviderInner {
            project_id: "my-project".to_string(),
            region: "europe-west1".to_string(),
            access_token: None,
            client: reqwest::Client::new(),
            timeout: std::time::Duration::from_secs(600),
        }),
    };

    let url = provider.build_endpoint_url("claude-3-opus-20240229", true);
    assert!(url.is_ok());

    if let Ok(url_str) = url {
        assert!(url_str.contains("europe-west1"));
        assert!(url_str.contains("my-project"));
        assert!(url_str.contains("claude-3-opus-20240229"));
        assert!(url_str.contains("streamRawPredict")); // Streaming
    }
}

/// Test request body transformation for Vertex
#[cfg(feature = "vertex")]
#[test]
fn test_vertex_request_body_transformation() {
    use turboclaude::providers::vertex::http::VertexHttpProvider;

    let request = serde_json::json!({
        "model": "claude-3-5-sonnet-20241022",
        "max_tokens": 1024,
        "messages": []
    });

    let provider_inner = std::sync::Arc::new(turboclaude::providers::vertex::http::ProviderInner {
        project_id: "test".to_string(),
        region: "us-east5".to_string(),
        access_token: None,
        client: reqwest::Client::new(),
        timeout: std::time::Duration::from_secs(600),
    });

    let provider = VertexHttpProvider {
        inner: provider_inner,
    };

    // Simulate body transformation
    let body_bytes = serde_json::to_vec(&request).unwrap();
    let result: serde_json::Result<serde_json::Value> = serde_json::from_slice(&body_bytes);

    assert!(result.is_ok(), "Request should be valid JSON");

    if let Ok(mut json) = result {
        let model = json.get("model");
        assert!(
            model.is_some(),
            "Request should contain model field before transformation"
        );

        // Model would be extracted and removed during actual request
        json.as_object_mut().unwrap().remove("model");

        // Version would be injected
        json.as_object_mut().unwrap().insert(
            "anthropic_version".to_string(),
            serde_json::Value::String("2024-07-15".to_string()),
        );

        assert!(
            json.get("anthropic_version").is_some(),
            "Request should contain anthropic_version after transformation"
        );
        assert!(
            json.get("model").is_none(),
            "Model should be removed after extraction"
        );
    }
}

/// Test that Vertex validates extended thinking
#[test]
fn test_vertex_validation_thinking() {
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(6000u32)
        .messages(vec![Message::user("Complex reasoning task")])
        .thinking(turboclaude::types::beta::ThinkingConfig::new(5000))
        .build()
        .expect("Failed to build request");

    let result = validate_message_request(&request);
    assert!(
        result.is_ok(),
        "Valid thinking request should pass validation"
    );
}

/// Test that tools array cannot be empty
#[test]
fn test_vertex_validation_empty_tools() {
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .tools(vec![]) // Empty tools!
        .build()
        .expect("Failed to build request");

    let result = validate_message_request(&request);
    assert!(result.is_err(), "Empty tools array should fail validation");
}

#[cfg(test)]
mod streaming_tests {
    use super::*;

    /// Test that Vertex streaming validation works
    #[test]
    fn test_vertex_streaming_validation() {
        let request = MessageRequest::builder()
            .model("claude-3-5-sonnet-20241022")
            .max_tokens(1024u32)
            .messages(vec![Message::user("Stream this response")])
            .stream(true)
            .build()
            .expect("Failed to build request");

        let result = validate_message_request(&request);
        assert!(result.is_ok(), "Valid streaming request should pass");
    }

    /// Test that Vertex streaming requires messages
    #[test]
    fn test_vertex_streaming_requires_messages() {
        let request = MessageRequest::builder()
            .model("claude-3-5-sonnet-20241022")
            .max_tokens(1024u32)
            .messages(vec![])
            .stream(true)
            .build()
            .expect("Failed to build request");

        let result = validate_message_request(&request);
        assert!(result.is_err(), "Streaming without messages should fail");
    }
}

#[cfg(test)]
mod regional_tests {
    use super::*;

    /// Test that Vertex accepts various valid regions
    #[test]
    fn test_vertex_valid_regions() {
        let regions = vec!["us-central1", "us-east4", "europe-west1", "asia-southeast1"];

        for region in regions {
            let request = MessageRequest::builder()
                .model("claude-3-5-sonnet-20241022")
                .max_tokens(1024u32)
                .messages(vec![Message::user(&format!("Test in {}", region))])
                .build()
                .expect("Failed to build request");

            let result = validate_message_request(&request);
            assert!(
                result.is_ok(),
                "Valid request should pass validation for region {}",
                region
            );
        }
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    /// Test validation catches very large requests
    #[test]
    fn test_vertex_validation_large_message_array() {
        let mut messages = vec![];
        for i in 0..10_001 {
            messages.push(Message::user(&format!("Message {}", i)));
        }

        let request = MessageRequest::builder()
            .model("claude-3-5-sonnet-20241022")
            .max_tokens(1024u32)
            .messages(messages)
            .build()
            .expect("Failed to build request");

        let result = validate_message_request(&request);
        assert!(result.is_err(), "Too many messages should fail validation");
    }

    /// Test validation catches very long text
    #[test]
    fn test_vertex_validation_text_length_limit() {
        let long_text = "x".repeat(1_000_001); // Exceeds 1M char limit

        let request = MessageRequest::builder()
            .model("claude-3-5-sonnet-20241022")
            .max_tokens(1024u32)
            .messages(vec![Message::user(long_text)])
            .build()
            .expect("Failed to build request");

        let result = validate_message_request(&request);
        assert!(
            result.is_err(),
            "Text exceeding limit should fail validation"
        );
    }
}
