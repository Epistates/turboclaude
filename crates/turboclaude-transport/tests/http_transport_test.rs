//! Integration tests for HTTP transport

use std::time::Duration;
use turboclaude_core::retry::BackoffStrategy;
use turboclaude_transport::http::RetryPolicy;
use turboclaude_transport::{HttpRequest, HttpTransport, Transport};

#[tokio::test]
async fn test_http_transport_creation() {
    let transport = HttpTransport::new().expect("Failed to create HTTP transport");
    assert!(transport.is_connected().await);
}

#[tokio::test]
async fn test_http_request_builder() {
    let request = HttpRequest::new("GET", "https://example.com")
        .with_header("Authorization", "Bearer token123")
        .with_header("Content-Type", "application/json");

    assert_eq!(request.method, "GET");
    assert_eq!(request.url, "https://example.com");
    assert_eq!(request.headers.len(), 2);
    assert_eq!(
        request.headers.get("Authorization"),
        Some(&"Bearer token123".to_string())
    );
}

#[tokio::test]
async fn test_http_request_with_body() {
    let body = vec![1, 2, 3, 4, 5];
    let request = HttpRequest::new("POST", "https://api.example.com").with_body(body.clone());

    assert_eq!(request.body, Some(body));
}

#[test]
fn test_retry_policy_default() {
    let policy = RetryPolicy::default();
    assert_eq!(policy.max_retries(), 3);

    // Verify it has sensible defaults for HTTP
    // Note: attempt 0 = delay before first retry (after initial attempt fails)
    let delay_0 = policy.calculate_delay(0);
    let delay_1 = policy.calculate_delay(1);

    // Default initial delay is 500ms ±10% jitter
    assert!(delay_0 >= Duration::from_millis(450));
    assert!(delay_0 <= Duration::from_millis(550));

    // Delays should increase exponentially with multiplier of 2.0
    // delay_1 ≈ 1000ms ±10% jitter
    assert!(delay_1 > delay_0);
    assert!(delay_1 >= Duration::from_millis(900));
    assert!(delay_1 <= Duration::from_millis(1100));
}

#[test]
fn test_retry_policy_builder() {
    let policy = RetryPolicy::builder()
        .max_retries(5)
        .initial_delay(Duration::from_secs(1))
        .max_delay(Duration::from_secs(120))
        .multiplier(1.5)
        .jitter(0.2)
        .build();

    assert_eq!(policy.max_retries(), 5);

    // Verify delay calculation respects configuration
    let delay_1 = policy.calculate_delay(1);
    assert!(delay_1 >= Duration::from_secs(1));
}

#[test]
fn test_retry_policy_delay_calculation() {
    let policy = RetryPolicy::default();

    let delay_0 = policy.calculate_delay(0);
    let delay_1 = policy.calculate_delay(1);
    let delay_2 = policy.calculate_delay(2);

    // Attempt numbers represent delays AFTER failures:
    // - delay_0 = delay before first retry (after initial attempt fails)
    // - delay_1 = delay before second retry (after first retry fails)
    // - delay_2 = delay before third retry (after second retry fails)

    // With initial_delay=500ms, multiplier=2.0, and jitter=0.1:
    // delay_0 ≈ 500ms (±10% jitter)
    assert!(delay_0 >= Duration::from_millis(450));
    assert!(delay_0 <= Duration::from_millis(550));

    // Delays should increase exponentially
    assert!(delay_1 > delay_0);
    assert!(delay_2 > delay_1);
}
