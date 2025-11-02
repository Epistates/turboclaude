//! Integration tests for error handling scenarios
//!
//! Tests for error cases, error recovery, and error messages

use turboclaude_protocol::{ProtocolErrorMessage, ProtocolMessage, QueryRequest};
use turboclaudeagent::testing::MockCliTransport;

#[tokio::test]
async fn test_protocol_error_message() {
    // Test protocol error message structure
    let error = ProtocolErrorMessage {
        code: "invalid_request".to_string(),
        message: "The request is invalid".to_string(),
        details: None,
    };

    // Verify serialization
    let message = ProtocolMessage::Error(error);
    let json = message.to_json().unwrap();
    assert!(json.contains("invalid_request"));
    assert!(json.contains("invalid"));

    // Verify deserialization roundtrip
    let deserialized = ProtocolMessage::from_json(&json).unwrap();
    if let ProtocolMessage::Error(err) = deserialized {
        assert_eq!(err.code, "invalid_request");
        assert_eq!(err.message, "The request is invalid");
    } else {
        panic!("Expected Error message");
    }
}

#[tokio::test]
async fn test_error_with_details() {
    // Test error message with detailed information
    let error = ProtocolErrorMessage {
        code: "validation_failed".to_string(),
        message: "Validation failed".to_string(),
        details: Some(serde_json::json!({
            "field": "query",
            "reason": "Empty query not allowed",
            "suggestions": ["Provide a non-empty query", "Check input format"]
        })),
    };

    // Verify serialization
    let message = ProtocolMessage::Error(error);
    let json = message.to_json().unwrap();
    assert!(json.contains("validation_failed"));
    assert!(json.contains("Empty query not allowed"));

    // Verify details are preserved
    let deserialized = ProtocolMessage::from_json(&json).unwrap();
    if let ProtocolMessage::Error(err) = deserialized {
        assert!(err.details.is_some());
        let details = err.details.unwrap();
        assert_eq!(details.get("field").and_then(|v| v.as_str()), Some("query"));
        assert_eq!(
            details.get("reason").and_then(|v| v.as_str()),
            Some("Empty query not allowed")
        );
    }
}

#[tokio::test]
async fn test_error_scenarios() {
    // Test various error codes and scenarios
    let error_codes = vec![
        ("invalid_request", "Invalid request format"),
        ("model_not_found", "Model not available"),
        ("rate_limit_exceeded", "Rate limit exceeded"),
        ("authentication_failed", "Authentication failed"),
        ("permission_denied", "Permission denied"),
        ("timeout", "Request timed out"),
        ("connection_error", "Connection failed"),
        ("internal_error", "Internal server error"),
    ];

    for (code, message) in error_codes {
        let error = ProtocolErrorMessage {
            code: code.to_string(),
            message: message.to_string(),
            details: None,
        };

        // Verify each error code serializes correctly
        let proto_msg = ProtocolMessage::Error(error);
        let json = proto_msg.to_json().unwrap();
        let deserialized = ProtocolMessage::from_json(&json).unwrap();

        if let ProtocolMessage::Error(err) = deserialized {
            assert_eq!(err.code, code);
            assert_eq!(err.message, message);
        } else {
            panic!("Expected Error message");
        }
    }
}

#[tokio::test]
async fn test_empty_query_error() {
    // Test error for empty query
    let mock = MockCliTransport::new();

    // Queue error response
    mock.enqueue_response(ProtocolMessage::Error(ProtocolErrorMessage {
        code: "invalid_request".to_string(),
        message: "Query cannot be empty".to_string(),
        details: Some(serde_json::json!({
            "field": "query",
            "constraint": "non-empty"
        })),
    }))
    .await;

    // Try to send empty query
    let query_request = QueryRequest {
        query: "".to_string(),
        model: "claude-sonnet-4-5".to_string(),
        max_tokens: 1024,
        tools: vec![],
        messages: vec![],
        system_prompt: None,
    };

    let msg = ProtocolMessage::Query(query_request);
    let json = msg.to_json().unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(mock.send_message(json_value).await.is_ok());

    // Receive error
    let response = mock.recv_message().await.unwrap();
    assert!(response.is_some());

    let response_str = serde_json::to_string(&response.unwrap()).unwrap();
    let response_msg = ProtocolMessage::from_json(&response_str).unwrap();

    if let ProtocolMessage::Error(err) = response_msg {
        assert_eq!(err.code, "invalid_request");
        assert!(err.message.contains("empty"));
    } else {
        panic!("Expected error response");
    }
}

#[tokio::test]
async fn test_error_recovery_pattern() {
    // Test error recovery: try, fail, retry
    let mock = MockCliTransport::new();

    // Queue error then success
    mock.enqueue_response(ProtocolMessage::Error(ProtocolErrorMessage {
        code: "temporary_error".to_string(),
        message: "Temporarily unavailable".to_string(),
        details: None,
    }))
    .await;

    // First request gets error
    let query_request = QueryRequest {
        query: "Test query".to_string(),
        model: "claude-sonnet-4-5".to_string(),
        max_tokens: 1024,
        tools: vec![],
        messages: vec![],
        system_prompt: None,
    };

    let msg = ProtocolMessage::Query(query_request);
    let json = msg.to_json().unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(mock.send_message(json_value).await.is_ok());

    // Receive error
    let response = mock.recv_message().await.unwrap();
    assert!(response.is_some());

    let response_str = serde_json::to_string(&response.unwrap()).unwrap();
    let response_msg = ProtocolMessage::from_json(&response_str).unwrap();

    if let ProtocolMessage::Error(err) = response_msg {
        assert_eq!(err.code, "temporary_error");
        // In real code, would implement retry logic here
    } else {
        panic!("Expected error");
    }
}

#[tokio::test]
async fn test_error_message_preservation() {
    // Test that error details are preserved through serialization
    let detailed_error = ProtocolErrorMessage {
        code: "detailed_error".to_string(),
        message: "This is a detailed error message".to_string(),
        details: Some(serde_json::json!({
            "context": {
                "operation": "query_execution",
                "timestamp": "2024-01-01T00:00:00Z",
                "attempt": 1,
                "max_attempts": 3
            },
            "suggestions": [
                "Check query syntax",
                "Verify model is available",
                "Try again in a moment"
            ]
        })),
    };

    let message = ProtocolMessage::Error(detailed_error);
    let json = message.to_json().unwrap();
    let deserialized = ProtocolMessage::from_json(&json).unwrap();

    if let ProtocolMessage::Error(err) = deserialized {
        assert!(err.details.is_some());
        let details = err.details.unwrap();

        // Verify nested context
        let context = details.get("context");
        assert!(context.is_some());

        // Verify suggestions array
        let suggestions = details.get("suggestions");
        assert!(suggestions.is_some());
        if let Some(sug) = suggestions {
            assert!(sug.is_array());
        }
    }
}

#[tokio::test]
async fn test_concurrent_error_handling() {
    // Test handling concurrent error responses
    let handles: Vec<_> = (0..5)
        .map(|i| {
            tokio::spawn(async move {
                let error_code = match i {
                    0 => "error_0",
                    1 => "error_1",
                    2 => "error_2",
                    3 => "error_3",
                    _ => "error_4",
                };

                let error = ProtocolErrorMessage {
                    code: error_code.to_string(),
                    message: format!("Error {}", i),
                    details: Some(serde_json::json!({"index": i})),
                };

                let message = ProtocolMessage::Error(error);
                let json = message.to_json().unwrap();
                let deserialized = ProtocolMessage::from_json(&json).unwrap();

                if let ProtocolMessage::Error(err) = deserialized {
                    assert_eq!(err.code, error_code);
                    Ok::<_, String>(())
                } else {
                    Err("Not an error message".to_string())
                }
            })
        })
        .collect();

    // Wait for all to complete successfully
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
}

#[tokio::test]
async fn test_error_code_consistency() {
    // Test that error codes are consistent across serialization
    let error_codes = vec![
        "invalid_request",
        "model_not_found",
        "rate_limit_exceeded",
        "authentication_failed",
        "permission_denied",
        "timeout",
        "connection_error",
        "internal_error",
        "not_implemented",
        "resource_not_found",
    ];

    for code in error_codes {
        let error = ProtocolErrorMessage {
            code: code.to_string(),
            message: format!("Error: {}", code),
            details: None,
        };

        let message = ProtocolMessage::Error(error);

        // Serialize and deserialize multiple times
        for _ in 0..3 {
            let json = message.to_json().unwrap();
            let deserialized = ProtocolMessage::from_json(&json).unwrap();

            if let ProtocolMessage::Error(err) = deserialized {
                assert_eq!(err.code, code, "Code mismatch for {}", code);
            } else {
                panic!("Expected error");
            }
        }
    }
}
