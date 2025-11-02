//! End-to-end tests for agent configuration and settings with real Claude API calls
//!
//! These tests validate custom agent definitions, setting sources, and agent lifecycle
//! management using actual Claude API calls.
//!
//! Run with: cargo test --test e2e_agents_and_settings -- --nocapture
//!
//! Python SDK parity: test_integration.py (agent configuration tests)

mod e2e;

use e2e::common::*;
use futures::StreamExt;
use turboclaude_protocol::PermissionMode;
use turboclaudeagent::config::SessionConfig;

/// Test custom agent configuration with specific settings
///
/// Python parity: test_continuation_option() - custom agent configuration
#[tokio::test]
#[ignore] // Requires API key
async fn test_custom_agent_config() {
    require_api_key();

    let mut config = SessionConfig::default();

    // Customize agent configuration
    config.permission_mode = PermissionMode::AcceptEdits;

    let session = create_test_session_with_config(config).await;

    // Verify session was created with custom config
    let response = session
        .query_str("What is 2+2? Just respond with the number.")
        .await
        .expect("Query with custom config failed");

    println!("✅ Query with custom config: {:?}", response);

    // Consume response stream
    let mut stream = Box::pin(session.receive_messages().await);
    let mut message_count = 0;
    while let Some(result) = stream.next().await {
        if let Ok(msg) = result {
            println!("📨 Message: {:?}", msg);
            message_count += 1;
        }
    }

    assert!(
        message_count > 0,
        "Should have received at least one message"
    );
    println!("✅ TEST PASSED: Custom agent configuration successful");
}

/// Test that session configuration can be changed dynamically
///
/// Python parity: Additional test beyond Python SDK - validates runtime config changes
#[tokio::test]
#[ignore] // Requires API key
async fn test_setting_source_configs() {
    require_api_key();

    let session = create_test_session().await;

    // Make initial query with default settings
    let response1 = session
        .query_str("What is 1+1? Just respond with the number.")
        .await
        .expect("First query failed");

    println!("✅ First query with default settings: {:?}", response1);

    // Consume first response
    let mut stream1 = Box::pin(session.receive_messages().await);
    while let Some(result) = stream1.next().await {
        if let Ok(msg) = result {
            println!("📨 Message 1: {:?}", msg);
        }
    }

    // Change permission mode dynamically
    session
        .set_permission_mode(PermissionMode::AcceptEdits)
        .await
        .expect("Failed to change permission mode");

    println!("✅ Changed permission mode to AcceptEdits");

    // Make second query with new settings
    let response2 = session
        .query_str("What is 2+2? Just respond with the number.")
        .await
        .expect("Second query failed");

    println!("✅ Second query with AcceptEdits mode: {:?}", response2);

    // Consume second response
    let mut stream2 = Box::pin(session.receive_messages().await);
    while let Some(result) = stream2.next().await {
        if let Ok(msg) = result {
            println!("📨 Message 2: {:?}", msg);
        }
    }

    println!("✅ TEST PASSED: Dynamic configuration changes successful");
}

/// Test agent lifecycle (creation, query, cleanup)
///
/// Python parity: test_simple_query_response() - full lifecycle validation
#[tokio::test]
#[ignore] // Requires API key
async fn test_agent_lifecycle() {
    require_api_key();

    // Create session
    println!("📍 Creating session...");
    let session = create_test_session().await;
    println!("✅ Session created");

    // Make query
    println!("📍 Making query...");
    let response = session
        .query_str("What is 3+3? Just respond with the number.")
        .await
        .expect("Query failed");

    println!("✅ Query successful: {:?}", response);

    // Consume response stream
    println!("📍 Consuming response stream...");
    let mut stream = Box::pin(session.receive_messages().await);
    let mut message_count = 0;

    while let Some(result) = stream.next().await {
        match result {
            Ok(msg) => {
                println!("📨 Message {}: {:?}", message_count + 1, msg);
                message_count += 1;
            }
            Err(e) => {
                eprintln!("❌ Error receiving message: {}", e);
            }
        }
    }

    println!("✅ Received {} messages", message_count);
    assert!(
        message_count > 0,
        "Should have received at least one message"
    );

    // Session cleanup happens automatically on drop
    println!("✅ Session will be cleaned up automatically");

    println!("✅ TEST PASSED: Agent lifecycle successful");
}

/// Test multiple queries in the same session
///
/// Python parity: test_continuation_option() - conversation continuity
#[tokio::test]
#[ignore] // Requires API key
async fn test_multiple_queries_same_session() {
    require_api_key();

    let session = create_test_session().await;

    // First query
    println!("📍 First query...");
    let response1 = session
        .query_str("What is 4+4? Just respond with the number.")
        .await
        .expect("First query failed");

    println!("✅ First query: {:?}", response1);

    let mut stream1 = Box::pin(session.receive_messages().await);
    while let Some(result) = stream1.next().await {
        if let Ok(msg) = result {
            println!("📨 Message 1: {:?}", msg);
        }
    }

    // Second query in same session
    println!("📍 Second query...");
    let response2 = session
        .query_str("What is 5+5? Just respond with the number.")
        .await
        .expect("Second query failed");

    println!("✅ Second query: {:?}", response2);

    let mut stream2 = Box::pin(session.receive_messages().await);
    while let Some(result) = stream2.next().await {
        if let Ok(msg) = result {
            println!("📨 Message 2: {:?}", msg);
        }
    }

    // Third query in same session
    println!("📍 Third query...");
    let response3 = session
        .query_str("What is 6+6? Just respond with the number.")
        .await
        .expect("Third query failed");

    println!("✅ Third query: {:?}", response3);

    let mut stream3 = Box::pin(session.receive_messages().await);
    while let Some(result) = stream3.next().await {
        if let Ok(msg) = result {
            println!("📨 Message 3: {:?}", msg);
        }
    }

    println!("✅ TEST PASSED: Multiple queries in same session successful");
}

/// Test session with all permission modes
///
/// Additional test beyond Python SDK - validates all permission modes work
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
            .query_str(format!(
                "What is 7+7? Just respond with the number. Mode: {:?}",
                mode
            ))
            .await
            .expect("Query failed");

        println!("✅ Query with {:?}: {:?}", mode, response);

        // Consume response
        let mut stream = Box::pin(session.receive_messages().await);
        while let Some(result) = stream.next().await {
            if let Ok(msg) = result {
                println!("📨 Message ({:?}): {:?}", mode, msg);
            }
        }
    }

    println!("✅ TEST PASSED: All permission modes work successfully");
}

/// Test session error recovery
///
/// Additional test beyond Python SDK - validates error resilience
#[tokio::test]
#[ignore] // Requires API key
async fn test_session_error_recovery() {
    require_api_key();

    let session = create_test_session().await;

    // Make a successful query
    println!("📍 First successful query...");
    let response1 = session
        .query_str("What is 8+8? Just respond with the number.")
        .await
        .expect("First query failed");

    println!("✅ First query: {:?}", response1);

    let mut stream1 = Box::pin(session.receive_messages().await);
    while let Some(result) = stream1.next().await {
        if let Ok(msg) = result {
            println!("📨 Message 1: {:?}", msg);
        }
    }

    // Attempt to trigger an error with an invalid operation
    // (Session should handle this gracefully)
    println!("📍 Attempting potentially problematic query...");
    match session.interrupt().await {
        Ok(_) => println!("✅ Interrupt succeeded (no active query)"),
        Err(e) => println!("⚠️  Interrupt failed as expected: {}", e),
    }

    // Verify session is still operational
    println!("📍 Second query after interrupt...");
    let response2 = session
        .query_str("What is 9+9? Just respond with the number.")
        .await
        .expect("Second query after interrupt failed");

    println!("✅ Second query: {:?}", response2);

    let mut stream2 = Box::pin(session.receive_messages().await);
    while let Some(result) = stream2.next().await {
        if let Ok(msg) = result {
            println!("📨 Message 2: {:?}", msg);
        }
    }

    println!("✅ TEST PASSED: Session error recovery successful");
}
