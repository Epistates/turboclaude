//! Integration tests for permission callbacks using MockTransport
//!
//! Tests permission evaluation, callbacks, and access control in isolation
//! without requiring a real Claude CLI subprocess.

mod common;

use common::mock_transport::MockTransport;
use serde_json::json;
use turboclaude_protocol::{
    PermissionCheckRequest, PermissionMode, PermissionResponse,
};
use turboclaudeagent::permissions::PermissionEvaluator;

#[tokio::test]
async fn test_permission_callback_allow() {
    // Test callback that allows tool execution
    let evaluator = PermissionEvaluator::new(PermissionMode::Default);

    // Register callback that allows all tools
    evaluator
        .register(|req| {
            Box::pin(async move {
                Ok(PermissionResponse {
                    allow: true,
                    modified_input: None,
                    reason: Some(format!("Tool {} allowed", req.tool)),
                })
            })
        })
        .await;

    // Check permission
    let request = PermissionCheckRequest {
        tool: "read_file".to_string(),
        input: json!({"path": "/tmp/test.txt"}),
        suggestion: "Read the file".to_string(),
    };

    let response = evaluator.check(request).await.unwrap();

    // Verify permission granted
    assert!(response.allow);
    assert!(response.reason.unwrap().contains("allowed"));
}

#[tokio::test]
async fn test_permission_callback_deny() {
    // Test callback that denies tool execution
    let evaluator = PermissionEvaluator::new(PermissionMode::Default);

    // Register callback that denies dangerous tools
    evaluator
        .register(|req| {
            Box::pin(async move {
                let is_dangerous = req.tool == "delete_file" || req.tool == "execute_command";

                if is_dangerous {
                    Ok(PermissionResponse {
                        allow: false,
                        modified_input: None,
                        reason: Some("Dangerous tool blocked".to_string()),
                    })
                } else {
                    Ok(PermissionResponse {
                        allow: true,
                        modified_input: None,
                        reason: None,
                    })
                }
            })
        })
        .await;

    // Try dangerous tool
    let request = PermissionCheckRequest {
        tool: "delete_file".to_string(),
        input: json!({"path": "/important/file.txt"}),
        suggestion: "Delete the file".to_string(),
    };

    let response = evaluator.check(request).await.unwrap();

    // Verify permission denied
    assert!(!response.allow);
    assert_eq!(response.reason.unwrap(), "Dangerous tool blocked");
}

#[tokio::test]
async fn test_permission_callback_modify_input() {
    // Test callback that modifies tool input for safety
    let evaluator = PermissionEvaluator::new(PermissionMode::Default);

    // Register callback that sanitizes file paths
    evaluator
        .register(|req| {
            Box::pin(async move {
                let mut modified_input = req.input.clone();

                // Restrict paths to /tmp
                if let Some(obj) = modified_input.as_object_mut() {
                    if let Some(path) = obj.get("path").and_then(|v| v.as_str()) {
                        if !path.starts_with("/tmp") {
                            obj.insert("path".to_string(), json!("/tmp/safe.txt"));
                        }
                    }
                    obj.insert("readonly".to_string(), json!(true));
                }

                Ok(PermissionResponse {
                    allow: true,
                    modified_input: Some(modified_input),
                    reason: Some("Input sanitized for safety".to_string()),
                })
            })
        })
        .await;

    // Request with potentially unsafe path
    let request = PermissionCheckRequest {
        tool: "read_file".to_string(),
        input: json!({"path": "/etc/passwd"}),
        suggestion: "Read the file".to_string(),
    };

    let response = evaluator.check(request).await.unwrap();

    // Verify input was modified
    assert!(response.allow);
    let modified = response.modified_input.unwrap();
    assert_eq!(modified["path"], "/tmp/safe.txt");
    assert_eq!(modified["readonly"], true);
}

#[tokio::test]
async fn test_permission_callback_exception_handling() {
    // Test that callback exceptions are handled gracefully
    let evaluator = PermissionEvaluator::new(PermissionMode::Default);

    // Register callback that returns error
    evaluator
        .register(|_req| {
            Box::pin(async move {
                Err(turboclaudeagent::error::AgentError::Config(
                    "Permission callback error".to_string(),
                ))
            })
        })
        .await;

    let request = PermissionCheckRequest {
        tool: "test_tool".to_string(),
        input: json!({}),
        suggestion: "Test".to_string(),
    };

    let result = evaluator.check(request).await;

    // Verify error is returned
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Permission callback error")
    );
}

#[tokio::test]
async fn test_permission_with_mock_transport() {
    // Test transport integration with permissions
    let transport = MockTransport::new();

    // Queue a permission check request
    transport
        .queue_response(json!({
            "tool": "web_search",
            "input": {"query": "test query"},
            "suggestion": "Use web search?"
        }))
        .await;

    // Simulate receiving the permission check
    let msg = transport.recv_message().await.unwrap();
    assert!(msg.is_some());

    let msg_value = msg.unwrap();
    assert_eq!(msg_value["tool"], "web_search");

    // Parse as PermissionCheckRequest (directly)
    let check: PermissionCheckRequest = serde_json::from_value(msg_value).unwrap();
    assert_eq!(check.tool, "web_search");
    assert_eq!(check.input["query"], "test query");
}

#[tokio::test]
async fn test_permission_mode_bypass() {
    // Test that BypassPermissions mode always allows
    let evaluator = PermissionEvaluator::new(PermissionMode::BypassPermissions);

    let request = PermissionCheckRequest {
        tool: "dangerous_tool".to_string(),
        input: json!({}),
        suggestion: "Use dangerous tool?".to_string(),
    };

    let response = evaluator.check(request).await.unwrap();

    // BypassPermissions mode should always allow
    assert!(response.allow);
    assert!(response.reason.unwrap().contains("bypassed"));
}

#[tokio::test]
async fn test_permission_mode_accept_edits() {
    // Test that AcceptEdits mode allows but can modify
    let evaluator = PermissionEvaluator::new(PermissionMode::AcceptEdits);

    // Set callback that modifies input
    evaluator
        .register(|req| {
            Box::pin(async move {
                let mut modified = req.input.clone();
                if let Some(obj) = modified.as_object_mut() {
                    obj.insert("safe_mode".to_string(), json!(true));
                }

                Ok(PermissionResponse {
                    allow: true,
                    modified_input: Some(modified),
                    reason: Some("Modified for safety".to_string()),
                })
            })
        })
        .await;

    let request = PermissionCheckRequest {
        tool: "safe_tool".to_string(),
        input: json!({"param": "value"}),
        suggestion: "Use tool?".to_string(),
    };

    let response = evaluator.check(request).await.unwrap();
    assert!(response.allow);
    // In AcceptEdits mode, modifications are applied
    assert!(response.modified_input.is_some());
}

#[tokio::test]
async fn test_permission_response_serialization() {
    // Test that PermissionResponse serializes correctly
    let response = PermissionResponse {
        allow: true,
        modified_input: Some(json!({"safe": true})),
        reason: Some("Allowed with modifications".to_string()),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&response).unwrap();

    // Verify fields
    assert!(json.contains(r#""allow":true"#));
    assert!(json.contains("modified_input"));
    assert!(json.contains("Allowed with modifications"));

    // Verify roundtrip
    let deserialized: PermissionResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.allow, true);
    assert!(deserialized.modified_input.is_some());
}

#[tokio::test]
async fn test_permission_check_serialization() {
    // Test that PermissionCheckRequest serializes correctly
    let request = PermissionCheckRequest {
        tool: "file_write".to_string(),
        input: json!({"path": "/tmp/test.txt", "content": "data"}),
        suggestion: "Write to file?".to_string(),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&request).unwrap();

    // Verify fields
    assert!(json.contains("file_write"));
    assert!(json.contains("/tmp/test.txt"));
    assert!(json.contains("Write to file?"));

    // Verify roundtrip
    let deserialized: PermissionCheckRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.tool, "file_write");
    assert_eq!(deserialized.input["path"], "/tmp/test.txt");
}

#[tokio::test]
async fn test_permission_multiple_tools() {
    // Test permission checks for multiple different tools
    let evaluator = PermissionEvaluator::new(PermissionMode::Default);

    // Callback with tool-specific logic
    evaluator
        .register(|req| {
            Box::pin(async move {
                let allow = match req.tool.as_str() {
                    "read_file" => true,
                    "write_file" => false,
                    "execute" => false,
                    _ => true,
                };

                Ok(PermissionResponse {
                    allow,
                    modified_input: None,
                    reason: if !allow {
                        Some(format!("Tool {} not allowed", req.tool))
                    } else {
                        None
                    },
                })
            })
        })
        .await;

    // Test read_file (should allow)
    let request = PermissionCheckRequest {
        tool: "read_file".to_string(),
        input: json!({}),
        suggestion: "".to_string(),
    };
    let response = evaluator.check(request).await.unwrap();
    assert!(response.allow);

    // Test write_file (should deny)
    let request = PermissionCheckRequest {
        tool: "write_file".to_string(),
        input: json!({}),
        suggestion: "".to_string(),
    };
    let response = evaluator.check(request).await.unwrap();
    assert!(!response.allow);

    // Test execute (should deny)
    let request = PermissionCheckRequest {
        tool: "execute".to_string(),
        input: json!({}),
        suggestion: "".to_string(),
    };
    let response = evaluator.check(request).await.unwrap();
    assert!(!response.allow);
}

#[tokio::test]
async fn test_permission_input_validation() {
    // Test callback that validates input structure
    let evaluator = PermissionEvaluator::new(PermissionMode::Default);

    // Callback that requires specific input fields
    evaluator
        .register(|req| {
            Box::pin(async move {
                // Require 'path' field for file operations
                if req.tool.contains("file") {
                    if req.input.get("path").is_none() {
                        return Ok(PermissionResponse {
                            allow: false,
                            modified_input: None,
                            reason: Some("Missing required 'path' field".to_string()),
                        });
                    }
                }

                Ok(PermissionResponse {
                    allow: true,
                    modified_input: None,
                    reason: None,
                })
            })
        })
        .await;

    // Test with missing path (should deny)
    let request = PermissionCheckRequest {
        tool: "read_file".to_string(),
        input: json!({}),
        suggestion: "".to_string(),
    };
    let response = evaluator.check(request).await.unwrap();
    assert!(!response.allow);
    assert!(response.reason.unwrap().contains("Missing required 'path'"));

    // Test with path (should allow)
    let request = PermissionCheckRequest {
        tool: "read_file".to_string(),
        input: json!({"path": "/tmp/test.txt"}),
        suggestion: "".to_string(),
    };
    let response = evaluator.check(request).await.unwrap();
    assert!(response.allow);
}
