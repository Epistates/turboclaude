//! End-to-end tests for tool permission handling with real Claude API calls
//!
//! These tests validate that the session properly handles tool permission scenarios
//! using actual Claude API calls.
//!
//! Run with: cargo test --test e2e_tool_permissions -- --nocapture --ignored
//!
//! Python SDK parity: test_tool_callbacks.py (TestToolPermissionCallbacks class)
//!
//! Note: For detailed permission callback tests (allow/deny, input modification),
//! see integration_permissions.rs which uses MockTransport.

mod e2e;

use e2e::common::*;
use turboclaude_protocol::PermissionMode;
use turboclaudeagent::config::SessionConfig;

/// Test session with default permission mode
///
/// Python parity: test_permission_callback_allow()
#[tokio::test]
#[ignore] // Requires API key
async fn test_session_default_permission_mode() {
    require_api_key();

    let mut config = SessionConfig::default();
    config.permission_mode = PermissionMode::Default;

    let session = create_test_session_with_config(config).await;

    let response = session
        .query_str("What is 2+2? Just respond with the number.")
        .await
        .expect("Query with default permission mode failed");

    println!("✅ Query with Default mode: {:?}", response);

    let count = consume_response_stream(&session).await;
    assert!(count > 0, "Should have received messages");

    println!("✅ TEST PASSED: Default permission mode works");
}

/// Test session with AcceptEdits permission mode
///
/// Python parity: test_accept_edits_mode_allows_modification()
#[tokio::test]
#[ignore] // Requires API key
async fn test_session_accept_edits_mode() {
    require_api_key();

    let mut config = SessionConfig::default();
    config.permission_mode = PermissionMode::AcceptEdits;

    let session = create_test_session_with_config(config).await;

    let response = session
        .query_str("What is 3+3? Just respond with the number.")
        .await
        .expect("Query with AcceptEdits mode failed");

    println!("✅ Query with AcceptEdits mode: {:?}", response);

    let count = consume_response_stream(&session).await;
    assert!(count > 0, "Should have received messages");

    println!("✅ TEST PASSED: AcceptEdits permission mode works");
}

/// Test session with BypassPermissions mode
///
/// Python parity: test_bypass_permissions_mode_skips_callback()
#[tokio::test]
#[ignore] // Requires API key
async fn test_session_bypass_permissions_mode() {
    require_api_key();

    let mut config = SessionConfig::default();
    config.permission_mode = PermissionMode::BypassPermissions;

    let session = create_test_session_with_config(config).await;

    let response = session
        .query_str("What is 4+4? Just respond with the number.")
        .await
        .expect("Query with BypassPermissions mode failed");

    println!("✅ Query with BypassPermissions mode: {:?}", response);

    let count = consume_response_stream(&session).await;
    assert!(count > 0, "Should have received messages");

    println!("✅ TEST PASSED: BypassPermissions mode works");
}

/// Test switching permission modes dynamically
///
/// Python parity: test_set_permission_mode() in e2e_dynamic_control.rs
#[tokio::test]
#[ignore] // Requires API key
async fn test_session_permission_mode_switching() {
    require_api_key();

    let session = create_test_session().await;

    // Test with default mode
    let response1 = session
        .query_str("What is 5+5? Just the number.")
        .await
        .expect("First query failed");

    println!("✅ Query with initial mode: {:?}", response1);
    consume_response_stream(&session).await;

    // Switch to AcceptEdits
    session
        .set_permission_mode(PermissionMode::AcceptEdits)
        .await
        .expect("Failed to switch to AcceptEdits");

    let response2 = session
        .query_str("What is 6+6? Just the number.")
        .await
        .expect("Second query failed");

    println!("✅ Query after mode switch: {:?}", response2);
    consume_response_stream(&session).await;

    println!("✅ TEST PASSED: Permission mode switching works");
}

/// Test that all permission modes are functional
///
/// Additional test - validates all three modes
#[tokio::test]
#[ignore] // Requires API key
async fn test_all_permission_modes() {
    require_api_key();

    let modes = vec![
        PermissionMode::Default,
        PermissionMode::AcceptEdits,
        PermissionMode::BypassPermissions,
    ];

    for mode in modes {
        println!("📍 Testing permission mode: {:?}", mode);

        let mut config = SessionConfig::default();
        config.permission_mode = mode;

        let session = create_test_session_with_config(config).await;

        let response = session
            .query_str("What is 7+7? Just respond with the number.")
            .await
            .expect("Query failed");

        println!("✅ Query with {:?}: {:?}", mode, response);

        let count = consume_response_stream(&session).await;
        assert!(count > 0, "Should have received messages with {:?}", mode);
    }

    println!("✅ TEST PASSED: All permission modes work");
}
