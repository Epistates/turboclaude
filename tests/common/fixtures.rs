//! Test fixtures using rstest
//!
//! Provides reusable test fixtures matching Python SDK's pytest fixtures:
//! - Mock servers for HTTP testing
//! - Pre-configured clients
//! - Common test data

use rstest::*;
use wiremock::MockServer;
use turboclaude::Client;
use std::time::Duration;

/// Fixture providing a wiremock HTTP server
/// Matches Python SDK's respx.MockRouter fixture
#[fixture]
pub async fn mock_server() -> MockServer {
    MockServer::start().await
}

/// Fixture providing a test client configured with mock server
/// Matches Python SDK's client fixture
#[fixture]
pub async fn test_client(#[future] mock_server: MockServer) -> Client {
    let server = mock_server.await;

    Client::builder()
        .api_key("test-api-key")
        .base_url(server.uri())
        .timeout(Duration::from_secs(5))
        .max_retries(0) // Disable retries in tests for predictability
        .build()
        .expect("Failed to build test client")
}

/// Test API key constant
pub const TEST_API_KEY: &str = "test-api-key-12345";

/// Test model constant
pub const TEST_MODEL: &str = "claude-3-5-sonnet-20241022";
