//! Integration tests for permission callbacks and permission system
//!
//! Tests for tool permission validation, callback handling, and permission modes

use turboclaude_protocol::{
    PermissionBehavior, PermissionCheckRequest, PermissionMode, PermissionResponse,
    PermissionRuleValue, PermissionUpdate, PermissionUpdateDestination,
};

#[tokio::test]
async fn test_permission_request_serialization() {
    // Test that permission request serializes correctly
    let request = PermissionCheckRequest {
        tool: "read_file".to_string(),
        input: serde_json::json!({"path": "/tmp/test.txt"}),
        suggestion: "Read the file".to_string(),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&request).unwrap();

    // Verify serialization
    assert!(json.contains("read_file"));
    assert!(json.contains("/tmp/test.txt"));
    assert!(json.contains("Read the file"));

    // Deserialize roundtrip
    let deserialized: PermissionCheckRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.tool, "read_file");
    assert_eq!(deserialized.suggestion, "Read the file");
}

#[tokio::test]
async fn test_permission_response_allow() {
    // Test permission allow response
    let response = PermissionResponse {
        allow: true,
        modified_input: None,
        reason: None,
    };

    // Serialize to JSON
    let json = serde_json::to_string(&response).unwrap();

    // Verify serialization
    assert!(json.contains("true"));

    // Deserialize roundtrip
    let deserialized: PermissionResponse = serde_json::from_str(&json).unwrap();
    assert!(deserialized.allow);
}

#[tokio::test]
async fn test_permission_response_deny() {
    // Test permission deny response
    let response = PermissionResponse {
        allow: false,
        modified_input: None,
        reason: Some("Tool not allowed in this context".to_string()),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&response).unwrap();

    // Verify serialization
    assert!(json.contains("false"));
    assert!(json.contains("Tool not allowed"));

    // Deserialize roundtrip
    let deserialized: PermissionResponse = serde_json::from_str(&json).unwrap();
    assert!(!deserialized.allow);
    assert_eq!(
        deserialized.reason,
        Some("Tool not allowed in this context".to_string())
    );
}

#[tokio::test]
async fn test_permission_allow_with_modified_input() {
    // Test allowing with modified input
    let response = PermissionResponse {
        allow: true,
        modified_input: Some(serde_json::json!({
            "path": "/safe/location/file.txt",
            "readonly": true
        })),
        reason: Some("Input path rewritten for safety".to_string()),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&response).unwrap();

    // Verify modified input is included
    assert!(json.contains("/safe/location/file.txt"));
    assert!(json.contains("readonly"));

    // Deserialize roundtrip
    let deserialized: PermissionResponse = serde_json::from_str(&json).unwrap();
    assert!(deserialized.allow);
    assert!(deserialized.modified_input.is_some());

    let modified = deserialized.modified_input.unwrap();
    assert_eq!(
        modified.get("path").and_then(|v| v.as_str()),
        Some("/safe/location/file.txt")
    );
    assert_eq!(
        modified.get("readonly").and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[tokio::test]
async fn test_multiple_permission_requests() {
    // Test handling multiple permission requests
    let permissions = vec![
        ("read_file", true, None),
        ("write_file", false, Some("Write not allowed".to_string())),
        (
            "execute_command",
            true,
            Some("Execution allowed".to_string()),
        ),
    ];

    // Test each permission request independently
    for (tool_name, allow, reason) in permissions {
        let request = PermissionCheckRequest {
            tool: tool_name.to_string(),
            input: serde_json::json!({}),
            suggestion: "Test".to_string(),
        };

        let response = PermissionResponse {
            allow,
            modified_input: None,
            reason: reason.clone(),
        };

        // Verify request serialization
        let request_json = serde_json::to_string(&request).unwrap();
        assert!(request_json.contains(tool_name));

        // Verify response serialization
        let response_json = serde_json::to_string(&response).unwrap();
        let deserialized: PermissionResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(deserialized.allow, allow);
        assert_eq!(deserialized.reason, reason);
    }
}

#[tokio::test]
async fn test_permission_decision_patterns() {
    // Test various permission decision patterns
    let patterns = vec![
        ("allow_all", true, None),
        ("deny_all", false, Some("Denied by policy".to_string())),
        (
            "allow_with_reason",
            true,
            Some("Allowed for auditing".to_string()),
        ),
        ("deny_specific_tool", false, None),
    ];

    for (pattern_name, allow, reason) in patterns {
        let response = PermissionResponse {
            allow,
            modified_input: None,
            reason: reason.clone(),
        };

        // Verify each pattern serializes correctly
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: PermissionResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(
            deserialized.allow, allow,
            "Pattern {} allow mismatch",
            pattern_name
        );
        assert_eq!(
            deserialized.reason, reason,
            "Pattern {} reason mismatch",
            pattern_name
        );
    }
}

#[tokio::test]
async fn test_permission_input_modification_types() {
    // Test various types of input modifications
    let modifications = vec![
        serde_json::json!({"path": "/safe/path"}),
        serde_json::json!({"readonly": true, "timeout": 30}),
        serde_json::json!({"allowed_operations": ["read", "list"]}),
        serde_json::json!({"user": "admin", "role": "operator"}),
    ];

    for modified_input in modifications {
        let response = PermissionResponse {
            allow: true,
            modified_input: Some(modified_input.clone()),
            reason: Some("Input modified".to_string()),
        };

        // Verify modification is preserved
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: PermissionResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.modified_input, Some(modified_input));
    }
}

#[tokio::test]
async fn test_concurrent_permission_checks() {
    // Test concurrent permission checks with serialization
    let handles: Vec<_> = (0..5)
        .map(|i| {
            tokio::spawn(async move {
                let request = PermissionCheckRequest {
                    tool: format!("tool_{}", i),
                    input: serde_json::json!({"index": i}),
                    suggestion: format!("Tool {} check", i),
                };

                let response = PermissionResponse {
                    allow: true,
                    modified_input: None,
                    reason: None,
                };

                // Verify request serialization
                let request_json = serde_json::to_string(&request).unwrap();
                assert!(request_json.contains(&format!("tool_{}", i)));

                // Verify response serialization
                let response_json = serde_json::to_string(&response).unwrap();
                let deserialized: PermissionResponse =
                    serde_json::from_str(&response_json).unwrap();
                assert!(deserialized.allow);

                Ok::<_, String>(())
            })
        })
        .collect();

    // Wait for all to complete
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
}

#[tokio::test]
async fn test_permission_whitelist_pattern() {
    // Test implementing a whitelist pattern
    let whitelist = ["read_file", "list_directory", "get_info"];

    let test_tools = vec![
        ("read_file", true),
        ("write_file", false),
        ("list_directory", true),
        ("delete_file", false),
        ("get_info", true),
        ("execute_command", false),
    ];

    for (tool, should_allow) in test_tools {
        let allowed = whitelist.contains(&tool);

        let response = PermissionResponse {
            allow: allowed,
            modified_input: None,
            reason: if !allowed {
                Some(format!("Tool {} not in whitelist", tool))
            } else {
                None
            },
        };

        // Verify decision matches expectation
        assert_eq!(response.allow, should_allow);
    }
}

#[tokio::test]
async fn test_permission_blacklist_pattern() {
    // Test implementing a blacklist pattern
    let blacklist = ["execute_command", "delete_file", "format_disk"];

    let test_tools = vec![
        ("read_file", true),
        ("execute_command", false),
        ("list_directory", true),
        ("delete_file", false),
        ("get_info", true),
        ("format_disk", false),
    ];

    for (tool, should_allow) in test_tools {
        let blocked = blacklist.contains(&tool);

        let response = PermissionResponse {
            allow: !blocked,
            modified_input: None,
            reason: if blocked {
                Some(format!("Tool {} is blacklisted", tool))
            } else {
                None
            },
        };

        // Verify decision matches expectation
        assert_eq!(response.allow, should_allow);
    }
}

#[tokio::test]
async fn test_permission_conditional_allow() {
    // Test conditional permission granting
    let test_cases = vec![
        ("read_file", serde_json::json!({"path": "/public"}), true),
        ("read_file", serde_json::json!({"path": "/private"}), false),
        (
            "execute_command",
            serde_json::json!({"command": "ls"}),
            true,
        ),
        (
            "execute_command",
            serde_json::json!({"command": "rm -rf /"}),
            false,
        ),
    ];

    for (tool, input, should_allow) in test_cases {
        let allowed = match tool {
            "read_file" => {
                // Allow only /public paths
                input
                    .get("path")
                    .and_then(|v| v.as_str())
                    .map(|p| p.contains("/public"))
                    .unwrap_or(false)
            }
            "execute_command" => {
                // Allow only safe commands
                input
                    .get("command")
                    .and_then(|v| v.as_str())
                    .map(|c| !c.contains("rm") && !c.contains("delete"))
                    .unwrap_or(false)
            }
            _ => true,
        };

        let response = PermissionResponse {
            allow: allowed,
            modified_input: None,
            reason: None,
        };

        // Verify conditional logic
        assert_eq!(response.allow, should_allow);
    }
}

#[tokio::test]
async fn test_permission_response_reason_messages() {
    // Test various reason messages for denied permissions
    let deny_reasons = vec![
        "Tool not allowed in this context",
        "User does not have permission to use this tool",
        "Tool disabled by administrator policy",
        "Tool requires additional authentication",
        "Resource is currently locked",
        "Tool usage limit exceeded",
    ];

    for reason in deny_reasons {
        let response = PermissionResponse {
            allow: false,
            modified_input: None,
            reason: Some(reason.to_string()),
        };

        // Verify reason is preserved
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: PermissionResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.reason, Some(reason.to_string()));
    }
}

// ===== PermissionUpdate Tests =====

#[tokio::test]
async fn test_permission_update_add_rules() {
    // Test adding permission rules
    let rules = vec![
        PermissionRuleValue::new("bash"),
        PermissionRuleValue::new("file_editor").with_rule_content("*.txt"),
    ];

    let update = PermissionUpdate::add_rules(rules, PermissionBehavior::Allow);

    // Validate
    assert!(update.validate().is_ok());

    // Serialize/deserialize
    let json = serde_json::to_string(&update).unwrap();
    assert!(json.contains("addRules"));
    assert!(json.contains("bash"));

    let deserialized: PermissionUpdate = serde_json::from_str(&json).unwrap();
    assert!(matches!(deserialized, PermissionUpdate::AddRules(_)));
}

#[tokio::test]
async fn test_permission_update_replace_rules() {
    // Test replacing permission rules
    let rules = vec![PermissionRuleValue::new("web_search")];

    let update = PermissionUpdate::replace_rules(rules, PermissionBehavior::Deny);

    assert!(update.validate().is_ok());

    let json = serde_json::to_string(&update).unwrap();
    assert!(json.contains("replaceRules"));
    assert!(json.contains("web_search"));
}

#[tokio::test]
async fn test_permission_update_remove_rules() {
    // Test removing permission rules
    let rules = vec![PermissionRuleValue::new("dangerous_tool")];

    let update = PermissionUpdate::remove_rules(rules);

    assert!(update.validate().is_ok());

    let json = serde_json::to_string(&update).unwrap();
    assert!(json.contains("removeRules"));
}

#[tokio::test]
async fn test_permission_update_set_mode() {
    // Test setting permission mode
    let update = PermissionUpdate::set_mode(PermissionMode::BypassPermissions);

    assert!(update.validate().is_ok());

    let json = serde_json::to_string(&update).unwrap();
    assert!(json.contains("setMode"));
}

#[tokio::test]
async fn test_permission_update_add_directories() {
    // Test adding directories
    let dirs = vec!["/home/user/project".to_string(), "/var/data".to_string()];

    let update = PermissionUpdate::add_directories(dirs);

    assert!(update.validate().is_ok());

    let json = serde_json::to_string(&update).unwrap();
    assert!(json.contains("addDirectories"));
    assert!(json.contains("/home/user/project"));
}

#[tokio::test]
async fn test_permission_update_remove_directories() {
    // Test removing directories
    let dirs = vec!["/tmp".to_string()];

    let update = PermissionUpdate::remove_directories(dirs);

    assert!(update.validate().is_ok());

    let json = serde_json::to_string(&update).unwrap();
    assert!(json.contains("removeDirectories"));
}

#[tokio::test]
async fn test_permission_update_with_destination() {
    // Test update with destination
    let rules = vec![PermissionRuleValue::new("bash")];
    let update = PermissionUpdate::add_rules(rules, PermissionBehavior::Allow)
        .with_destination(PermissionUpdateDestination::Session);

    assert_eq!(
        update.destination(),
        Some(PermissionUpdateDestination::Session)
    );

    let json = serde_json::to_string(&update).unwrap();
    assert!(json.contains("session"));
}

#[tokio::test]
async fn test_permission_update_validation_empty_rules() {
    // Test that empty rules fail validation
    let update = PermissionUpdate::add_rules(vec![], PermissionBehavior::Allow);

    assert!(update.validate().is_err());
}

#[tokio::test]
async fn test_permission_update_validation_empty_directories() {
    // Test that empty directories fail validation
    let update = PermissionUpdate::add_directories(vec![]);

    assert!(update.validate().is_err());
}

#[tokio::test]
async fn test_permission_update_validation_empty_tool_name() {
    // Test that empty tool name fails validation
    let rules = vec![PermissionRuleValue::new("")];
    let update = PermissionUpdate::add_rules(rules, PermissionBehavior::Allow);

    assert!(update.validate().is_err());
}

#[tokio::test]
async fn test_permission_update_multiple_behaviors() {
    // Test different permission behaviors
    let behaviors = [
        PermissionBehavior::Allow,
        PermissionBehavior::Deny,
        PermissionBehavior::Ask,
    ];

    for behavior in behaviors {
        let rules = vec![PermissionRuleValue::new("test_tool")];
        let update = PermissionUpdate::add_rules(rules, behavior);

        assert!(update.validate().is_ok());

        let json = serde_json::to_string(&update).unwrap();
        let deserialized: PermissionUpdate = serde_json::from_str(&json).unwrap();

        // Verify behavior is preserved
        match deserialized {
            PermissionUpdate::AddRules(u) => assert_eq!(u.behavior, behavior),
            _ => panic!("Expected AddRules"),
        }
    }
}

#[tokio::test]
async fn test_permission_update_concurrent_application() {
    // Test concurrent permission updates
    let updates = vec![
        PermissionUpdate::add_rules(
            vec![PermissionRuleValue::new("tool_1")],
            PermissionBehavior::Allow,
        ),
        PermissionUpdate::add_directories(vec!["/dir_1".to_string()]),
        PermissionUpdate::set_mode(PermissionMode::AcceptEdits),
    ];

    let handles: Vec<_> = updates
        .into_iter()
        .map(|update| {
            tokio::spawn(async move {
                // Validate update
                assert!(update.validate().is_ok());

                // Serialize/deserialize
                let json = serde_json::to_string(&update).unwrap();
                let _deserialized: PermissionUpdate = serde_json::from_str(&json).unwrap();

                Ok::<_, String>(())
            })
        })
        .collect();

    // Wait for all to complete
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
}

#[tokio::test]
async fn test_permission_rule_value_with_content() {
    // Test permission rule with content
    let rule = PermissionRuleValue::new("file_editor").with_rule_content("*.txt");

    assert_eq!(rule.tool_name, "file_editor");
    assert_eq!(rule.rule_content, Some("*.txt".to_string()));

    // Serialize/deserialize
    let json = serde_json::to_string(&rule).unwrap();
    let deserialized: PermissionRuleValue = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.tool_name, "file_editor");
    assert_eq!(deserialized.rule_content, Some("*.txt".to_string()));
}

#[tokio::test]
async fn test_permission_update_all_destinations() {
    // Test all destination types
    let destinations = [
        PermissionUpdateDestination::UserSettings,
        PermissionUpdateDestination::ProjectSettings,
        PermissionUpdateDestination::LocalSettings,
        PermissionUpdateDestination::Session,
    ];

    for destination in destinations {
        let rules = vec![PermissionRuleValue::new("test")];
        let update = PermissionUpdate::add_rules(rules, PermissionBehavior::Allow)
            .with_destination(destination);

        assert_eq!(update.destination(), Some(destination));

        // Verify serialization
        let json = serde_json::to_string(&update).unwrap();
        let deserialized: PermissionUpdate = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.destination(), Some(destination));
    }
}
