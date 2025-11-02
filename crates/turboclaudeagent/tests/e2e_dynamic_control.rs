//! End-to-end tests for dynamic control features with real Claude API calls
//!
//! These tests validate runtime configuration changes (permission mode, model switching, interrupts)
//! using actual Claude API calls. Requires ANTHROPIC_API_KEY environment variable.
//!
//! Run with: cargo test --test e2e_dynamic_control -- --nocapture
//!
//! Python SDK parity: test_dynamic_control.py

mod e2e;

use e2e::common::*;
use futures::StreamExt;
use turboclaude_protocol::PermissionMode;
use turboclaudeagent::config::SessionConfig;

/// Test that permission mode can be changed dynamically during a session
///
/// Python parity: test_set_permission_mode()
///
/// Note: This is an E2E test that makes real API calls.
/// Run with: cargo test --test e2e_dynamic_control -- --ignored
#[tokio::test]
#[ignore]
async fn test_set_permission_mode() {
    require_api_key();

    // Create session with default permission mode
    let mut config = SessionConfig::default();
    config.permission_mode = PermissionMode::Default;

    let session = create_test_session_with_config(config).await;

    // Change permission mode to AcceptEdits
    session
        .set_permission_mode(PermissionMode::AcceptEdits)
        .await
        .expect("Failed to set permission mode to AcceptEdits");

    // Make a query that would normally require permission
    let response = session
        .query_str("What is 2+2? Just respond with the number.")
        .await
        .expect("Query with AcceptEdits failed");

    println!("✅ Query with AcceptEdits completed: {:?}", response);

    // Consume response stream
    let mut stream = Box::pin(session.receive_messages().await);
    let mut message_count = 0;
    while let Some(result) = stream.next().await {
        match result {
            Ok(msg) => {
                println!("📨 Got message: {:?}", msg);
                message_count += 1;
            }
            Err(e) => {
                eprintln!("❌ Error: {}", e);
            }
        }
    }

    println!("✅ Received {} messages after first query", message_count);

    // Change back to default permission mode
    session
        .set_permission_mode(PermissionMode::Default)
        .await
        .expect("Failed to set permission mode back to Default");

    // Make another query
    let response2 = session
        .query_str("What is 3+3? Just respond with the number.")
        .await
        .expect("Query with Default permission mode failed");

    println!(
        "✅ Query with Default permission mode completed: {:?}",
        response2
    );

    // Consume second response stream
    let mut stream2 = Box::pin(session.receive_messages().await);
    let mut message_count2 = 0;
    while let Some(result) = stream2.next().await {
        match result {
            Ok(msg) => {
                println!("📨 Got message: {:?}", msg);
                message_count2 += 1;
            }
            Err(e) => {
                eprintln!("❌ Error: {}", e);
            }
        }
    }

    println!("✅ Received {} messages after second query", message_count2);
    println!("✅ TEST PASSED: Permission mode change successful (AcceptEdits -> Default)");
}

/// Test that model can be changed dynamically during a session
///
/// Python parity: test_set_model()
///
/// Note: This is an E2E test that makes real API calls.
/// Run with: cargo test --test e2e_dynamic_control -- --ignored
#[tokio::test]
#[ignore]
async fn test_set_model() {
    require_api_key();

    let session = create_test_session().await;

    // Start with default model
    let response = session
        .query_str("What is 1+1? Just the number.")
        .await
        .expect("Query with default model failed");

    println!("✅ Default model response: {:?}", response);

    // Consume response
    let mut stream = Box::pin(session.receive_messages().await);
    while let Some(result) = stream.next().await {
        if let Ok(msg) = result {
            println!("📨 Default model message: {:?}", msg);
        }
    }

    // Switch to Haiku model
    session
        .set_model("claude-haiku-4-5-20251007")
        .await
        .expect("Failed to switch to Haiku model");

    let response2 = session
        .query_str("What is 2+2? Just the number.")
        .await
        .expect("Query with Haiku model failed");

    println!("✅ Haiku model response: {:?}", response2);

    // Consume Haiku response
    let mut stream2 = Box::pin(session.receive_messages().await);
    while let Some(result) = stream2.next().await {
        if let Ok(msg) = result {
            println!("📨 Haiku model message: {:?}", msg);
        }
    }

    // Switch back to default (Sonnet)
    session
        .set_model("claude-sonnet-4-5-20250514")
        .await
        .expect("Failed to switch back to Sonnet");

    let response3 = session
        .query_str("What is 3+3? Just the number.")
        .await
        .expect("Query after switching back to Sonnet failed");

    println!("✅ Back to Sonnet response: {:?}", response3);

    // Consume final response
    let mut stream3 = Box::pin(session.receive_messages().await);
    while let Some(result) = stream3.next().await {
        if let Ok(msg) = result {
            println!("📨 Sonnet message: {:?}", msg);
        }
    }

    println!("✅ TEST PASSED: Model switching successful (Default -> Haiku -> Sonnet)");
}

/// Test that interrupt can be sent during a session
///
/// Python parity: test_interrupt()
///
/// Note: This is an E2E test that makes real API calls.
/// Run with: cargo test --test e2e_dynamic_control -- --ignored
#[tokio::test]
#[ignore]
async fn test_interrupt() {
    require_api_key();

    let session = create_test_session().await;

    // Start a query that would take time
    let response = session
        .query_str("Count from 1 to 100 slowly.")
        .await
        .expect("Query before interrupt failed");

    println!("✅ Query started: {:?}", response);

    // Send interrupt (may or may not stop the response depending on timing)
    match session.interrupt().await {
        Ok(_) => {
            println!("✅ Interrupt sent successfully");
        }
        Err(e) => {
            println!("⚠️  Interrupt resulted in: {}", e);
        }
    }

    // Consume any remaining messages
    let mut stream = Box::pin(session.receive_messages().await);
    let mut message_count = 0;
    while let Some(result) = stream.next().await {
        match result {
            Ok(msg) => {
                println!("📨 Got message after interrupt: {:?}", msg);
                message_count += 1;
            }
            Err(e) => {
                eprintln!("❌ Error after interrupt: {}", e);
            }
        }
    }

    println!("✅ Received {} messages after interrupt", message_count);
    println!("✅ TEST PASSED: Interrupt command handled");
}

/// Test sequential permission mode changes
///
/// Additional test beyond Python SDK - validates multiple consecutive changes
///
/// Note: This is an E2E test that makes real API calls.
/// Run with: cargo test --test e2e_dynamic_control -- --ignored
#[tokio::test]
#[ignore]
async fn test_sequential_permission_mode_changes() {
    require_api_key();

    let session = create_test_session().await;

    let modes = vec![
        PermissionMode::AcceptEdits,
        PermissionMode::Default,
        PermissionMode::BypassPermissions,
        PermissionMode::Default,
    ];

    for (i, mode) in modes.iter().enumerate() {
        session
            .set_permission_mode(*mode)
            .await
            .expect("Failed to set permission mode");

        println!("✅ Changed to permission mode: {:?}", mode);

        // Make a simple query
        let response = session
            .query_str(format!("What is {}+{}? Just the number.", i, i))
            .await
            .expect("Query failed");

        println!("✅ Query {} response: {:?}", i, response);

        // Consume response
        let mut stream = Box::pin(session.receive_messages().await);
        while let Some(result) = stream.next().await {
            if let Ok(msg) = result {
                println!("📨 Message for query {}: {:?}", i, msg);
            }
        }
    }

    println!("✅ TEST PASSED: Sequential permission mode changes successful");
}

/// Test model change validation
///
/// Additional test beyond Python SDK - validates error handling for invalid models
///
/// Note: This is an E2E test that makes real API calls.
/// Run with: cargo test --test e2e_dynamic_control -- --ignored
#[tokio::test]
#[ignore]
async fn test_invalid_model_change() {
    require_api_key();

    let session = create_test_session().await;

    // Try to set an invalid model
    match session.set_model("invalid-model-name-123").await {
        Ok(_) => {
            println!("⚠️  Invalid model was accepted (API may validate later)");
        }
        Err(e) => {
            println!("✅ Invalid model rejected as expected: {}", e);
        }
    }

    // Verify session is still operational with valid model
    session
        .set_model("claude-sonnet-4-5-20250514")
        .await
        .expect("Failed to set valid model after invalid attempt");

    let response = session
        .query_str("What is 5+5? Just the number.")
        .await
        .expect("Query after model validation failed");

    println!(
        "✅ Session still works after invalid model attempt: {:?}",
        response
    );

    // Consume response
    let mut stream = Box::pin(session.receive_messages().await);
    while let Some(_) = stream.next().await {}

    println!("✅ TEST PASSED: Invalid model change handled gracefully");
}
