//! End-to-end tests for hook-related functionality with real Claude API calls
//!
//! These tests validate that the session can handle queries that would trigger hooks,
//! testing the full stack integration with actual Claude API calls.
//!
//! Run with: cargo test --test e2e_hooks -- --nocapture --ignored
//!
//! Python SDK parity: test_tool_callbacks.py (TestHookCallbacks class)
//!
//! Note: For detailed hook callback tests (allow/deny decisions, input modification),
//! see integration_hook_callbacks.rs which uses MockTransport.

mod e2e;

use e2e::common::*;

/// Test that session successfully handles simple queries
///
/// Python parity: test_hook_output_fields() - basic execution
#[tokio::test]
#[ignore] // Requires API key
async fn test_session_basic_query() {
    require_api_key();

    let session = create_test_session().await;

    let response = session
        .query_str("What is 2+2? Just respond with the number.")
        .await
        .expect("Query failed");

    println!("✅ Query response: {:?}", response);

    // Consume response stream
    let message_count = consume_response_stream(&session).await;
    assert!(
        message_count > 0,
        "Should have received at least one message"
    );

    println!("✅ TEST PASSED: Basic query execution successful");
}

/// Test session handles multiple sequential queries
///
/// Python parity: test_hook_execution() - sequential hook calls
#[tokio::test]
#[ignore] // Requires API key
async fn test_session_sequential_queries() {
    require_api_key();

    let session = create_test_session().await;

    for i in 1..=3 {
        println!("📍 Query {}", i);

        let response = session
            .query_str(format!(
                "What is {}+{}? Just respond with the number.",
                i, i
            ))
            .await
            .expect("Query failed");

        println!("✅ Query {} response: {:?}", i, response);

        let count = consume_response_stream(&session).await;
        assert!(count > 0, "Query {} should have received messages", i);
    }

    println!("✅ TEST PASSED: Sequential queries successful");
}

/// Test session handles queries with different content types
///
/// Additional test - validates various message types
#[tokio::test]
#[ignore] // Requires API key
async fn test_session_various_message_types() {
    require_api_key();

    let session = create_test_session().await;

    let queries = vec![
        "What is 3+3? Just the number.",
        "List three colors.",
        "Explain gravity in one sentence.",
    ];

    for query in queries {
        println!("📍 Query: {}", query);

        let response = session.query_str(query).await.expect("Query failed");

        println!("✅ Response: {:?}", response);

        let count = consume_response_stream(&session).await;
        assert!(count > 0, "Should have received messages");
    }

    println!("✅ TEST PASSED: Various message types handled");
}
