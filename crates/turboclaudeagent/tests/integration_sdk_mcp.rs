//! Integration tests for SDK MCP server functionality.
//!
//! These tests verify that SDK MCP servers work correctly within the
//! AgentSession and configuration system.

use serde::{Deserialize, Serialize};
use turboclaudeagent::SessionConfig;
use turboclaudeagent::mcp::sdk::*;

#[derive(Deserialize, Clone)]
struct TestInput {
    value: i32,
}

#[derive(Serialize, PartialEq, Debug)]
struct TestOutput {
    result: i32,
}

#[tokio::test]
async fn test_sdk_server_in_config() {
    let server = SdkMcpServerBuilder::new("test")
        .tool("double", "Double a number", |input: TestInput| async move {
            Ok(TestOutput {
                result: input.value * 2,
            })
        })
        .build();

    let config = SessionConfig::new().add_sdk_server(server);

    assert_eq!(config.sdk_servers.len(), 1);
    assert_eq!(config.sdk_servers[0].name(), "test");
    assert_eq!(config.sdk_servers[0].tool_count(), 1);
}

#[tokio::test]
async fn test_multiple_sdk_servers() {
    let server1 = SdkMcpServerBuilder::new("server1")
        .tool("tool1", "First tool", |input: TestInput| async move {
            Ok(TestOutput {
                result: input.value + 1,
            })
        })
        .build();

    let server2 = SdkMcpServerBuilder::new("server2")
        .tool("tool2", "Second tool", |input: TestInput| async move {
            Ok(TestOutput {
                result: input.value + 2,
            })
        })
        .build();

    let config = SessionConfig::new()
        .add_sdk_server(server1)
        .add_sdk_server(server2);

    assert_eq!(config.sdk_servers.len(), 2);

    // Verify both servers are present
    assert!(config.sdk_servers.iter().any(|s| s.name() == "server1"));
    assert!(config.sdk_servers.iter().any(|s| s.name() == "server2"));

    // Verify tools are accessible
    let s1 = &config.sdk_servers[0];
    let s2 = &config.sdk_servers[1];

    let result1 = s1
        .execute_tool("tool1", serde_json::json!({"value": 10}))
        .await
        .expect("tool1 execution failed");
    assert_eq!(result1, serde_json::json!({"result": 11}));

    let result2 = s2
        .execute_tool("tool2", serde_json::json!({"value": 10}))
        .await
        .expect("tool2 execution failed");
    assert_eq!(result2, serde_json::json!({"result": 12}));
}

#[tokio::test]
async fn test_with_sdk_servers_method() {
    let server1 = SdkMcpServerBuilder::new("s1")
        .tool("t1", "Tool 1", |input: TestInput| async move {
            Ok(TestOutput {
                result: input.value,
            })
        })
        .build();

    let server2 = SdkMcpServerBuilder::new("s2")
        .tool("t2", "Tool 2", |input: TestInput| async move {
            Ok(TestOutput {
                result: input.value,
            })
        })
        .build();

    let config = SessionConfig::new().with_sdk_servers(vec![server1, server2]);

    assert_eq!(config.sdk_servers.len(), 2);
}

#[tokio::test]
async fn test_sdk_server_cloning() {
    let server = SdkMcpServerBuilder::new("original")
        .tool("tool", "A tool", |input: TestInput| async move {
            Ok(TestOutput {
                result: input.value * 3,
            })
        })
        .build();

    // Clone the server
    let cloned = server.clone();

    // Both should work independently
    let result1 = server
        .execute_tool("tool", serde_json::json!({"value": 5}))
        .await
        .expect("original execution failed");
    let result2 = cloned
        .execute_tool("tool", serde_json::json!({"value": 5}))
        .await
        .expect("cloned execution failed");

    assert_eq!(result1, result2);
    assert_eq!(result1, serde_json::json!({"result": 15}));
}

#[tokio::test]
async fn test_sdk_server_with_async_state() {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // Create shared state
    let counter = Arc::new(Mutex::new(0));

    // Capture state in tool closures
    let counter_clone = counter.clone();
    let server = SdkMcpServerBuilder::new("stateful")
        .tool(
            "increment",
            "Increment counter",
            move |_input: TestInput| {
                let counter = counter_clone.clone();
                async move {
                    let mut count = counter.lock().await;
                    *count += 1;
                    Ok(TestOutput { result: *count })
                }
            },
        )
        .build();

    // Execute tool multiple times
    for i in 1..=3 {
        let result = server
            .execute_tool("increment", serde_json::json!({"value": 0}))
            .await
            .expect("execution failed");
        assert_eq!(result, serde_json::json!({"result": i}));
    }

    // Verify final state
    let final_count = *counter.lock().await;
    assert_eq!(final_count, 3);
}

#[tokio::test]
async fn test_tool_not_found_error() {
    let server = SdkMcpServerBuilder::new("test")
        .tool("exists", "Existing tool", |input: TestInput| async move {
            Ok(TestOutput {
                result: input.value,
            })
        })
        .build();

    let result = server
        .execute_tool("nonexistent", serde_json::json!({"value": 1}))
        .await;

    assert!(result.is_err());
    match result {
        Err(SdkToolError::InvalidInput(msg)) => {
            assert!(msg.contains("not found"));
            assert!(msg.contains("nonexistent"));
        }
        _ => panic!("Expected InvalidInput error for missing tool"),
    }
}

#[tokio::test]
async fn test_sdk_server_debug_format() {
    let server = SdkMcpServerBuilder::new("debug-test")
        .tool("t1", "Tool 1", |input: TestInput| async move {
            Ok(TestOutput {
                result: input.value,
            })
        })
        .tool("t2", "Tool 2", |input: TestInput| async move {
            Ok(TestOutput {
                result: input.value,
            })
        })
        .build();

    let debug_str = format!("{:?}", server);
    assert!(debug_str.contains("debug-test"));
    assert!(debug_str.contains("tool_count"));
    assert!(debug_str.contains("2")); // Two tools
}

#[tokio::test]
async fn test_config_default_has_empty_sdk_servers() {
    let config = SessionConfig::default();
    assert!(config.sdk_servers.is_empty());
}

#[tokio::test]
async fn test_tool_execution_with_error() {
    let server = SdkMcpServerBuilder::new("error-test")
        .tool("failing", "Always fails", |_input: TestInput| async move {
            Err::<TestOutput, _>(SdkToolError::ExecutionFailed(
                "Tool failed intentionally".to_string(),
            ))
        })
        .build();

    let result = server
        .execute_tool("failing", serde_json::json!({"value": 1}))
        .await;

    assert!(result.is_err());
    match result {
        Err(SdkToolError::ExecutionFailed(msg)) => {
            assert_eq!(msg, "Tool failed intentionally");
        }
        _ => panic!("Expected ExecutionFailed error"),
    }
}

#[tokio::test]
async fn test_sdk_server_has_tool() {
    let server = SdkMcpServerBuilder::new("test")
        .tool("exists", "Exists", |input: TestInput| async move {
            Ok(TestOutput {
                result: input.value,
            })
        })
        .build();

    assert!(server.has_tool("exists"));
    assert!(!server.has_tool("does_not_exist"));
}

#[tokio::test]
async fn test_empty_sdk_server() {
    let server = SdkMcpServerBuilder::new("empty").build();

    assert_eq!(server.tool_count(), 0);
    assert!(server.list_tools().is_empty());

    let result = server.execute_tool("any", serde_json::json!({})).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_sdk_server_builder_fluent_api() {
    // Test that the builder can chain multiple tools fluently
    let server = SdkMcpServerBuilder::new("fluent")
        .tool("t1", "Tool 1", |input: TestInput| async move {
            Ok(TestOutput {
                result: input.value + 1,
            })
        })
        .tool("t2", "Tool 2", |input: TestInput| async move {
            Ok(TestOutput {
                result: input.value + 2,
            })
        })
        .tool("t3", "Tool 3", |input: TestInput| async move {
            Ok(TestOutput {
                result: input.value + 3,
            })
        })
        .build();

    assert_eq!(server.tool_count(), 3);
    assert!(server.has_tool("t1"));
    assert!(server.has_tool("t2"));
    assert!(server.has_tool("t3"));

    // Verify each tool works correctly
    let r1 = server
        .execute_tool("t1", serde_json::json!({"value": 10}))
        .await
        .unwrap();
    assert_eq!(r1, serde_json::json!({"result": 11}));

    let r2 = server
        .execute_tool("t2", serde_json::json!({"value": 10}))
        .await
        .unwrap();
    assert_eq!(r2, serde_json::json!({"result": 12}));

    let r3 = server
        .execute_tool("t3", serde_json::json!({"value": 10}))
        .await
        .unwrap();
    assert_eq!(r3, serde_json::json!({"result": 13}));
}
