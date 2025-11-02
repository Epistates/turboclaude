//! Integration tests for control protocol commands (interrupt, model change, permission mode)
//!
//! These tests validate control commands using MockCliTransport to simulate CLI responses

use turboclaude_protocol::hooks::{HookContext, HookMatcher};
use turboclaude_protocol::protocol::ControlRequest;
use turboclaude_protocol::{ControlCommand, ControlResponse, ProtocolMessage};
use turboclaudeagent::testing::MockCliTransport;

/// Helper to create a successful control response
fn create_control_response(success: bool, message: Option<String>) -> ControlResponse {
    ControlResponse {
        success,
        message,
        data: None,
    }
}

// ============================================================================
// INTERRUPT COMMAND TESTS
// ============================================================================

#[tokio::test]
async fn test_interrupt_command_serialization() {
    // Test that interrupt command serializes correctly
    let control_request = ControlRequest {
        command: ControlCommand::Interrupt,
    };

    let message = ProtocolMessage::ControlRequest(control_request);
    let json = message.to_json().unwrap();

    // Verify serialization
    assert!(json.contains("interrupt"));

    // Verify deserialization roundtrip
    let deserialized = ProtocolMessage::from_json(&json).unwrap();
    match deserialized {
        ProtocolMessage::ControlRequest(req) => {
            match req.command {
                ControlCommand::Interrupt => {
                    // Success - interrupt command preserved
                }
                _ => panic!("Expected Interrupt command"),
            }
        }
        _ => panic!("Expected ControlRequest"),
    }
}

#[tokio::test]
async fn test_interrupt_command_sending() {
    // Test that interrupt command can be sent via transport
    let mock = MockCliTransport::new();

    // Queue a response
    let response = create_control_response(true, Some("Interrupted".to_string()));
    mock.enqueue_response(ProtocolMessage::ControlResponse(response))
        .await;

    // Create and send interrupt
    let control_request = ControlRequest {
        command: ControlCommand::Interrupt,
    };

    let message = ProtocolMessage::ControlRequest(control_request);
    let json = message.to_json().unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Send interrupt
    assert!(mock.send_message(json_value).await.is_ok());

    // Verify sent
    assert_eq!(mock.sent_messages().await.len(), 1);

    // Receive response
    let response = mock.recv_message().await.unwrap();
    assert!(response.is_some());
}

#[tokio::test]
async fn test_interrupt_command_multiple_calls() {
    // Test that multiple interrupts can be sent
    let mock = MockCliTransport::new();

    // Queue multiple responses
    for _ in 0..3 {
        mock.enqueue_response(ProtocolMessage::ControlResponse(create_control_response(
            true, None,
        )))
        .await;
    }

    // Send multiple interrupts
    for _ in 0..3 {
        let control_request = ControlRequest {
            command: ControlCommand::Interrupt,
        };

        let message = ProtocolMessage::ControlRequest(control_request);
        let json = message.to_json().unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(mock.send_message(json_value).await.is_ok());
    }

    // Verify all sent
    assert_eq!(mock.sent_messages().await.len(), 3);
}

// ============================================================================
// MODEL CHANGE COMMAND TESTS
// ============================================================================

#[tokio::test]
async fn test_set_model_command_serialization() {
    // Test model change serialization
    let control_request = ControlRequest {
        command: ControlCommand::SetModel("claude-opus-4-1".to_string()),
    };

    let message = ProtocolMessage::ControlRequest(control_request);
    let json = message.to_json().unwrap();

    // Verify serialization
    assert!(json.contains("set_model"));
    assert!(json.contains("claude-opus-4-1"));

    // Verify deserialization roundtrip
    let deserialized = ProtocolMessage::from_json(&json).unwrap();
    match deserialized {
        ProtocolMessage::ControlRequest(req) => match req.command {
            ControlCommand::SetModel(model) => {
                assert_eq!(model, "claude-opus-4-1");
            }
            _ => panic!("Expected SetModel command"),
        },
        _ => panic!("Expected ControlRequest"),
    }
}

#[tokio::test]
async fn test_set_model_command_sending() {
    // Test sending model change command
    let mock = MockCliTransport::new();

    // Queue response
    mock.enqueue_response(ProtocolMessage::ControlResponse(create_control_response(
        true,
        Some("Model changed to claude-opus-4-1".to_string()),
    )))
    .await;

    // Send model change
    let control_request = ControlRequest {
        command: ControlCommand::SetModel("claude-opus-4-1".to_string()),
    };

    let message = ProtocolMessage::ControlRequest(control_request);
    let json = message.to_json().unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(mock.send_message(json_value).await.is_ok());

    // Receive response
    let response = mock.recv_message().await.unwrap();
    assert!(response.is_some());

    let response_str = serde_json::to_string(&response.unwrap()).unwrap();
    let response_msg = ProtocolMessage::from_json(&response_str).unwrap();

    if let ProtocolMessage::ControlResponse(resp) = response_msg {
        assert!(resp.success);
        assert!(resp.message.is_some());
    } else {
        panic!("Expected ControlResponse");
    }
}

#[tokio::test]
async fn test_set_model_different_models() {
    // Test switching between different models
    let mock = MockCliTransport::new();

    let models = vec!["claude-opus-4-1", "claude-sonnet-4-5", "claude-haiku-4"];

    for model in models {
        mock.enqueue_response(ProtocolMessage::ControlResponse(create_control_response(
            true,
            Some(format!("Model changed to {}", model)),
        )))
        .await;
    }

    // Send model changes
    for model in ["claude-opus-4-1", "claude-sonnet-4-5", "claude-haiku-4"] {
        let control_request = ControlRequest {
            command: ControlCommand::SetModel(model.to_string()),
        };

        let message = ProtocolMessage::ControlRequest(control_request);
        let json = message.to_json().unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(mock.send_message(json_value).await.is_ok());
    }

    // Verify all sent
    assert_eq!(mock.sent_messages().await.len(), 3);
}

// ============================================================================
// PERMISSION MODE CHANGE TESTS
// ============================================================================

#[tokio::test]
async fn test_set_permission_mode_command_serialization() {
    // Test permission mode change serialization
    let control_request = ControlRequest {
        command: ControlCommand::SetPermissionMode("default".to_string()),
    };

    let message = ProtocolMessage::ControlRequest(control_request);
    let json = message.to_json().unwrap();

    // Verify serialization
    assert!(json.contains("set_permission_mode"));
    assert!(json.contains("default"));

    // Verify deserialization roundtrip
    let deserialized = ProtocolMessage::from_json(&json).unwrap();
    match deserialized {
        ProtocolMessage::ControlRequest(req) => match req.command {
            ControlCommand::SetPermissionMode(mode) => {
                assert_eq!(mode, "default");
            }
            _ => panic!("Expected SetPermissionMode command"),
        },
        _ => panic!("Expected ControlRequest"),
    }
}

#[tokio::test]
async fn test_set_permission_mode_command_sending() {
    // Test sending permission mode change
    let mock = MockCliTransport::new();

    // Queue response
    mock.enqueue_response(ProtocolMessage::ControlResponse(create_control_response(
        true,
        Some("Permission mode changed to acceptEdits".to_string()),
    )))
    .await;

    // Send permission mode change
    let control_request = ControlRequest {
        command: ControlCommand::SetPermissionMode("acceptEdits".to_string()),
    };

    let message = ProtocolMessage::ControlRequest(control_request);
    let json = message.to_json().unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(mock.send_message(json_value).await.is_ok());

    // Verify sent
    assert_eq!(mock.sent_messages().await.len(), 1);
}

#[tokio::test]
async fn test_permission_mode_transitions() {
    // Test transitioning between different permission modes
    let mock = MockCliTransport::new();

    let modes = vec!["default", "acceptEdits", "bypassPermissions"];

    for mode in &modes {
        mock.enqueue_response(ProtocolMessage::ControlResponse(create_control_response(
            true,
            Some(format!("Permission mode changed to {}", mode)),
        )))
        .await;
    }

    // Send mode changes
    for mode in modes {
        let control_request = ControlRequest {
            command: ControlCommand::SetPermissionMode(mode.to_string()),
        };

        let message = ProtocolMessage::ControlRequest(control_request);
        let json = message.to_json().unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(mock.send_message(json_value).await.is_ok());
    }

    // Verify all sent
    assert_eq!(mock.sent_messages().await.len(), 3);
}

// ============================================================================
// ERROR SCENARIO TESTS
// ============================================================================

#[tokio::test]
async fn test_control_command_failure_response() {
    // Test handling failed control response
    let mock = MockCliTransport::new();

    // Queue failed response
    mock.enqueue_response(ProtocolMessage::ControlResponse(create_control_response(
        false,
        Some("Model not found".to_string()),
    )))
    .await;

    // Send invalid model
    let control_request = ControlRequest {
        command: ControlCommand::SetModel("invalid-model-xyz".to_string()),
    };

    let message = ProtocolMessage::ControlRequest(control_request);
    let json = message.to_json().unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(mock.send_message(json_value).await.is_ok());

    // Receive failure response
    let response = mock.recv_message().await.unwrap();
    assert!(response.is_some());

    let response_str = serde_json::to_string(&response.unwrap()).unwrap();
    let response_msg = ProtocolMessage::from_json(&response_str).unwrap();

    if let ProtocolMessage::ControlResponse(resp) = response_msg {
        assert!(!resp.success);
        assert_eq!(resp.message, Some("Model not found".to_string()));
    } else {
        panic!("Expected ControlResponse");
    }
}

#[tokio::test]
async fn test_concurrent_control_commands() {
    // Test sending control commands concurrently
    let mock = MockCliTransport::new();

    // Queue responses for all tasks
    for _ in 0..5 {
        mock.enqueue_response(ProtocolMessage::ControlResponse(create_control_response(
            true, None,
        )))
        .await;
    }

    // Spawn concurrent control commands
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let mock_clone = mock.clone();
            tokio::spawn(async move {
                let control_request = if i % 2 == 0 {
                    ControlRequest {
                        command: ControlCommand::Interrupt,
                    }
                } else {
                    ControlRequest {
                        command: ControlCommand::SetModel(format!("model-{}", i)),
                    }
                };

                let message = ProtocolMessage::ControlRequest(control_request);
                let json = message.to_json().unwrap();
                let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

                mock_clone.send_message(json_value).await
            })
        })
        .collect();

    // Wait for all to complete
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }

    // Verify all sent
    assert_eq!(mock.sent_messages().await.len(), 5);
}

#[tokio::test]
async fn test_get_state_command_serialization() {
    // Test GetState command (for future use)
    let control_request = ControlRequest {
        command: ControlCommand::GetState,
    };

    let message = ProtocolMessage::ControlRequest(control_request);
    let json = message.to_json().unwrap();

    // Verify serialization
    assert!(json.contains("get_state"));

    // Verify deserialization roundtrip
    let deserialized = ProtocolMessage::from_json(&json).unwrap();
    match deserialized {
        ProtocolMessage::ControlRequest(req) => {
            match req.command {
                ControlCommand::GetState => {
                    // Success
                }
                _ => panic!("Expected GetState command"),
            }
        }
        _ => panic!("Expected ControlRequest"),
    }
}

#[tokio::test]
async fn test_control_response_with_data() {
    // Test control response with data payload
    let mock = MockCliTransport::new();

    // Queue response with data
    let mut response = create_control_response(true, Some("State retrieved".to_string()));
    response.data = Some(serde_json::json!({
        "current_model": "claude-opus-4-1",
        "permission_mode": "default",
        "session_active": true
    }));

    mock.enqueue_response(ProtocolMessage::ControlResponse(response))
        .await;

    // Send GetState command
    let control_request = ControlRequest {
        command: ControlCommand::GetState,
    };

    let message = ProtocolMessage::ControlRequest(control_request);
    let json = message.to_json().unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(mock.send_message(json_value).await.is_ok());

    // Receive response with data
    let response = mock.recv_message().await.unwrap();
    assert!(response.is_some());

    let response_str = serde_json::to_string(&response.unwrap()).unwrap();
    let response_msg = ProtocolMessage::from_json(&response_str).unwrap();

    if let ProtocolMessage::ControlResponse(resp) = response_msg {
        assert!(resp.success);
        assert!(resp.data.is_some());
        let data = resp.data.unwrap();
        assert_eq!(
            data.get("current_model").and_then(|v| v.as_str()),
            Some("claude-opus-4-1")
        );
    } else {
        panic!("Expected ControlResponse");
    }
}

// ============================================================================
// HOOK MATCHER TESTS
// ============================================================================

#[tokio::test]
async fn test_hook_matcher_empty_matches_all() {
    // Empty matcher should match all contexts
    let matcher = HookMatcher::new();

    let context = HookContext::new("PreToolUse").with_tool_name("Bash");
    assert!(matcher.matches(&context));

    let context = HookContext::new("PostToolUse").with_tool_name("Write");
    assert!(matcher.matches(&context));

    let context = HookContext::new("UserPromptSubmit");
    assert!(matcher.matches(&context));
}

#[tokio::test]
async fn test_hook_matcher_exact_tool_name() {
    // Matcher with exact tool name
    let matcher = HookMatcher::new().with_tool_name("Bash");

    let context = HookContext::new("PreToolUse").with_tool_name("Bash");
    assert!(matcher.matches(&context));

    let context = HookContext::new("PreToolUse").with_tool_name("Write");
    assert!(!matcher.matches(&context));

    // Case sensitive
    let context = HookContext::new("PreToolUse").with_tool_name("bash");
    assert!(!matcher.matches(&context));
}

#[tokio::test]
async fn test_hook_matcher_regex_single_tool() {
    // Regex matching single tool with anchors
    let matcher = HookMatcher::new().with_tool_name_regex(r"^Bash$");

    let context = HookContext::new("PreToolUse").with_tool_name("Bash");
    assert!(matcher.matches(&context));

    let context = HookContext::new("PreToolUse").with_tool_name("BashScript");
    assert!(!matcher.matches(&context));
}

#[tokio::test]
async fn test_hook_matcher_regex_multiple_tools() {
    // Regex matching multiple tools (Write|Edit|MultiEdit)
    let matcher = HookMatcher::new().with_tool_name_regex(r"^(Write|Edit|MultiEdit)$");

    let context = HookContext::new("PreToolUse").with_tool_name("Write");
    assert!(matcher.matches(&context));

    let context = HookContext::new("PreToolUse").with_tool_name("Edit");
    assert!(matcher.matches(&context));

    let context = HookContext::new("PreToolUse").with_tool_name("MultiEdit");
    assert!(matcher.matches(&context));

    let context = HookContext::new("PreToolUse").with_tool_name("Bash");
    assert!(!matcher.matches(&context));
}

#[tokio::test]
async fn test_hook_matcher_regex_partial_match() {
    // Regex without anchors matches partial strings
    let matcher = HookMatcher::new().with_tool_name_regex(r"Write");

    let context = HookContext::new("PreToolUse").with_tool_name("Write");
    assert!(matcher.matches(&context));

    let context = HookContext::new("PreToolUse").with_tool_name("MultiWrite");
    assert!(matcher.matches(&context));

    let context = HookContext::new("PreToolUse").with_tool_name("WriteFile");
    assert!(matcher.matches(&context));
}

#[tokio::test]
async fn test_hook_matcher_required_input_fields() {
    // Matcher requiring specific input fields
    let matcher = HookMatcher::new()
        .with_tool_name("Bash")
        .with_required_fields(vec!["command".to_string()]);

    let input = serde_json::json!({ "command": "ls -la" });
    let context = HookContext::new("PreToolUse")
        .with_tool_name("Bash")
        .with_tool_input(input);
    assert!(matcher.matches(&context));

    // Missing required field
    let input = serde_json::json!({ "other": "value" });
    let context = HookContext::new("PreToolUse")
        .with_tool_name("Bash")
        .with_tool_input(input);
    assert!(!matcher.matches(&context));
}

#[tokio::test]
async fn test_hook_matcher_multiple_required_fields() {
    // Matcher requiring multiple input fields
    let matcher = HookMatcher::new()
        .with_tool_name("Write")
        .with_required_fields(vec!["file_path".to_string(), "content".to_string()]);

    let input = serde_json::json!({ "file_path": "/tmp/test.txt", "content": "hello" });
    let context = HookContext::new("PreToolUse")
        .with_tool_name("Write")
        .with_tool_input(input);
    assert!(matcher.matches(&context));

    // Missing one field
    let input = serde_json::json!({ "file_path": "/tmp/test.txt" });
    let context = HookContext::new("PreToolUse")
        .with_tool_name("Write")
        .with_tool_input(input);
    assert!(!matcher.matches(&context));
}

#[tokio::test]
async fn test_hook_matcher_event_types() {
    // Matcher for specific event types
    let matcher = HookMatcher::new()
        .with_event_types(vec!["PreToolUse".to_string(), "PostToolUse".to_string()]);

    let context = HookContext::new("PreToolUse");
    assert!(matcher.matches(&context));

    let context = HookContext::new("PostToolUse");
    assert!(matcher.matches(&context));

    let context = HookContext::new("UserPromptSubmit");
    assert!(!matcher.matches(&context));
}

#[tokio::test]
async fn test_hook_matcher_combined_criteria() {
    // Matcher with multiple criteria (all must match)
    let matcher = HookMatcher::new()
        .with_tool_name_regex(r"^(Write|Edit)$")
        .with_event_types(vec!["PreToolUse".to_string()])
        .with_required_fields(vec!["file_path".to_string()]);

    // All criteria match
    let input = serde_json::json!({ "file_path": "/tmp/test.txt", "content": "test" });
    let context = HookContext::new("PreToolUse")
        .with_tool_name("Write")
        .with_tool_input(input);
    assert!(matcher.matches(&context));

    // Wrong tool name
    let input = serde_json::json!({ "file_path": "/tmp/test.txt" });
    let context = HookContext::new("PreToolUse")
        .with_tool_name("Bash")
        .with_tool_input(input);
    assert!(!matcher.matches(&context));

    // Wrong event type
    let input = serde_json::json!({ "file_path": "/tmp/test.txt" });
    let context = HookContext::new("PostToolUse")
        .with_tool_name("Write")
        .with_tool_input(input);
    assert!(!matcher.matches(&context));

    // Missing required field
    let input = serde_json::json!({ "content": "test" });
    let context = HookContext::new("PreToolUse")
        .with_tool_name("Write")
        .with_tool_input(input);
    assert!(!matcher.matches(&context));
}

#[tokio::test]
async fn test_hook_matcher_no_tool_name_with_regex() {
    // Context without tool_name should not match when regex is specified
    let matcher = HookMatcher::new().with_tool_name_regex(r"^Bash$");

    let context = HookContext::new("UserPromptSubmit"); // No tool_name
    assert!(!matcher.matches(&context));
}

#[tokio::test]
async fn test_hook_matcher_no_input_with_required_fields() {
    // Context without tool_input should not match when required fields are specified
    let matcher = HookMatcher::new().with_required_fields(vec!["command".to_string()]);

    let context = HookContext::new("PreToolUse").with_tool_name("Bash");
    assert!(!matcher.matches(&context));
}

#[tokio::test]
async fn test_hook_matcher_is_empty() {
    let empty_matcher = HookMatcher::new();
    assert!(empty_matcher.is_empty());

    let matcher = HookMatcher::new().with_tool_name("Bash");
    assert!(!matcher.is_empty());

    let matcher = HookMatcher::new().with_event_types(vec!["PreToolUse".to_string()]);
    assert!(!matcher.is_empty());
}

#[tokio::test]
async fn test_hook_matcher_serialization() {
    // Test that HookMatcher can be serialized and deserialized
    let matcher = HookMatcher::new()
        .with_tool_name("Bash")
        .with_tool_name_regex(r"^Bash$");

    let json = serde_json::to_string(&matcher).unwrap();
    let deserialized: HookMatcher = serde_json::from_str(&json).unwrap();

    assert_eq!(matcher.tool_name, deserialized.tool_name);
    assert_eq!(
        matcher.tool_name_regex.as_ref().map(|r| r.as_str()),
        deserialized.tool_name_regex.as_ref().map(|r| r.as_str())
    );
}

#[tokio::test]
async fn test_hook_context_builder() {
    // Test HookContext builder pattern
    let context = HookContext::new("PreToolUse")
        .with_tool_name("Bash")
        .with_tool_input(serde_json::json!({"command": "ls"}))
        .with_session_id("session-123");

    assert_eq!(context.event_type, "PreToolUse");
    assert_eq!(context.tool_name, Some("Bash".to_string()));
    assert_eq!(context.session_id, Some("session-123".to_string()));
    assert!(context.tool_input.is_some());
}

#[tokio::test]
async fn test_hook_matcher_case_sensitivity() {
    // Tool names are case-sensitive
    let matcher = HookMatcher::new().with_tool_name("Bash");

    let context = HookContext::new("PreToolUse").with_tool_name("Bash");
    assert!(matcher.matches(&context));

    let context = HookContext::new("PreToolUse").with_tool_name("BASH");
    assert!(!matcher.matches(&context));

    let context = HookContext::new("PreToolUse").with_tool_name("bash");
    assert!(!matcher.matches(&context));
}

#[tokio::test]
async fn test_hook_matcher_regex_complex_patterns() {
    // Test complex regex patterns
    let matcher = HookMatcher::new().with_tool_name_regex(r"^[A-Z][a-z]+$");

    let context = HookContext::new("PreToolUse").with_tool_name("Bash");
    assert!(matcher.matches(&context));

    let context = HookContext::new("PreToolUse").with_tool_name("Write");
    assert!(matcher.matches(&context));

    let context = HookContext::new("PreToolUse").with_tool_name("MultiEdit");
    assert!(!matcher.matches(&context)); // Has uppercase mid-word
}

#[tokio::test]
async fn test_hook_matcher_event_types_single() {
    // Matcher for single event type
    let matcher = HookMatcher::new().with_event_types(vec!["PreToolUse".to_string()]);

    let context = HookContext::new("PreToolUse");
    assert!(matcher.matches(&context));

    let context = HookContext::new("PostToolUse");
    assert!(!matcher.matches(&context));
}
