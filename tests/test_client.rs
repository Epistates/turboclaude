//! Client configuration and initialization tests
//!
//! Matches Python SDK's test_client.py with comprehensive coverage:
//! - Builder pattern validation
//! - Authentication (API key, auth token)
//! - Custom headers and timeouts
//! - Base URL configuration
//! - Resource lazy initialization

use turboclaude::{Client, Error};
use rstest::*;
use std::time::Duration;

mod common;

#[test]
fn test_client_new_with_api_key() {
    let client = Client::new("test-key");
    // Should not panic
    let _ = client.messages();
}

#[test]
fn test_client_builder_with_api_key() {
    let result = Client::builder()
        .api_key("test-key")
        .build();

    assert!(result.is_ok());
}

#[test]
fn test_client_builder_with_auth_token() {
    let result = Client::builder()
        .auth_token("test-token")
        .build();

    assert!(result.is_ok());
}

#[test]
fn test_client_builder_custom_base_url() {
    let result = Client::builder()
        .api_key("test-key")
        .base_url("https://custom.anthropic.com")
        .build();

    assert!(result.is_ok());
}

#[test]
fn test_client_builder_custom_api_version() {
    let result = Client::builder()
        .api_key("test-key")
        .api_version("2024-01-01")
        .build();

    assert!(result.is_ok());
}

#[test]
fn test_client_builder_custom_timeout() {
    let result = Client::builder()
        .api_key("test-key")
        .timeout(Duration::from_secs(60))
        .build();

    assert!(result.is_ok());
}

#[test]
fn test_client_builder_custom_max_retries() {
    let result = Client::builder()
        .api_key("test-key")
        .max_retries(5)
        .build();

    assert!(result.is_ok());
}

#[test]
fn test_client_builder_custom_headers() {
    let result = Client::builder()
        .api_key("test-key")
        .default_header("x-custom-header", "custom-value")?
        .default_header("x-another-header", "another-value")?
        .build();

    assert!(result.is_ok());
}

#[test]
fn test_client_builder_all_options() {
    let result = Client::builder()
        .api_key("test-key")
        .base_url("https://custom.anthropic.com")
        .api_version("2024-01-01")
        .timeout(Duration::from_secs(60))
        .max_retries(5)
        .default_header("x-custom", "value")?
        .build();

    assert!(result.is_ok());
}

#[test]
fn test_client_clone() {
    let client1 = Client::new("test-key");
    let client2 = client1.clone();

    // Both clients should work independently
    let _ = client1.messages();
    let _ = client2.messages();
}

#[test]
fn test_client_lazy_resource_initialization() {
    let client = Client::new("test-key");

    // Resources should be created on first access
    let messages1 = client.messages();
    let messages2 = client.messages();

    // Should return the same reference (OnceLock behavior)
    assert!(std::ptr::eq(messages1, messages2));
}

#[test]
fn test_client_all_resources() {
    let client = Client::new("test-key");

    // All resources should be accessible
    let _ = client.messages();
    let _ = client.completions();
    let _ = client.models();
    let _ = client.beta();
}

/// Test environment variable loading (requires env feature)
#[cfg(feature = "env")]
#[test]
fn test_client_from_env_missing_key() {
    // Use temp-env to safely clear environment variables (Rust 2024 compliant)
    temp_env::with_var_unset("ANTHROPIC_API_KEY", || {
        temp_env::with_var_unset("ANTHROPIC_AUTH_TOKEN", || {
            let result = Client::builder().build();

            assert!(result.is_err());
            match result {
                Err(Error::Authentication(_)) => {}
                _ => panic!("Expected Authentication error"),
            }
        });
    });
}

/// Parametrized test for invalid base URLs
#[rstest]
#[case("not-a-url")]
#[case("ftp://invalid.com")]
#[case("")]
fn test_client_builder_invalid_base_url(#[case] base_url: &str) {
    let result = Client::builder()
        .api_key("test-key")
        .base_url(base_url)
        .build();

    assert!(result.is_err());
    match result {
        Err(Error::InvalidUrl(_)) => {},
        _ => panic!("Expected InvalidUrl error for: {}", base_url),
    }
}

/// Parametrized test for valid timeouts
#[rstest]
#[case(Duration::from_secs(1))]
#[case(Duration::from_secs(30))]
#[case(Duration::from_secs(120))]
#[case(Duration::from_millis(500))]
fn test_client_builder_valid_timeouts(#[case] timeout: Duration) {
    let result = Client::builder()
        .api_key("test-key")
        .timeout(timeout)
        .build();

    assert!(result.is_ok());
}

/// Parametrized test for valid retry counts
#[rstest]
#[case(0)]
#[case(1)]
#[case(3)]
#[case(10)]
fn test_client_builder_valid_retries(#[case] max_retries: u32) {
    let result = Client::builder()
        .api_key("test-key")
        .max_retries(max_retries)
        .build();

    assert!(result.is_ok());
}
