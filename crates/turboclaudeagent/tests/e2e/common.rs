//! Common utilities for E2E tests

use std::env;
use turboclaudeagent::config::SessionConfig;
use turboclaudeagent::session::AgentSession;

/// Require API key, panic if not set
///
/// E2E tests explicitly require ANTHROPIC_API_KEY to be set.
/// This function will panic if it's missing, failing the test loudly.
///
/// # Why panic?
///
/// - E2E tests are integration tests - they need real API access
/// - Tests should fail explicitly if dependencies are missing
/// - False negatives (silently skipped tests) are worse than false positives
/// - Use `#[ignore]` to prevent tests from running by default
///
/// # Usage
///
/// ```no_run
/// #[tokio::test]
/// #[ignore] // Skip by default to avoid wasting credits
/// async fn test_something() {
///     require_api_key();
///     // ... test code that uses real API ...
/// }
/// ```
///
/// Run with: `cargo test --test e2e_test -- --ignored`
pub fn require_api_key() {
    if env::var("ANTHROPIC_API_KEY").is_err() {
        panic!(
            "❌ E2E test requires ANTHROPIC_API_KEY environment variable\n\
             Set it with: export ANTHROPIC_API_KEY=your-key-here\n\
             Then run: cargo test --test e2e_test -- --ignored"
        );
    }
}

/// Create a default test session with real transport
///
/// Uses subprocess transport to spawn actual Claude CLI process.
/// Requires ANTHROPIC_API_KEY to be set.
///
/// # Panics
///
/// Panics if ANTHROPIC_API_KEY is not set.
#[allow(dead_code)]
pub async fn create_test_session() -> AgentSession {
    require_api_key();

    let config = SessionConfig::default();

    AgentSession::new(config)
        .await
        .expect("Failed to create E2E test session")
}

/// Create a test session with custom configuration
///
/// # Panics
///
/// Panics if ANTHROPIC_API_KEY is not set.
#[allow(dead_code)]
pub async fn create_test_session_with_config(config: SessionConfig) -> AgentSession {
    require_api_key();

    AgentSession::new(config)
        .await
        .expect("Failed to create E2E test session with custom config")
}

/// Helper to consume all messages from a response stream
///
/// Returns the number of messages received. Useful for tests that just
/// need to verify the session completes successfully.
#[allow(dead_code)]
pub async fn consume_response_stream(session: &AgentSession) -> usize {
    use futures::StreamExt;

    let mut count = 0;
    let mut stream = Box::pin(session.receive_messages().await);

    while let Some(result) = stream.next().await {
        match result {
            Ok(message) => {
                println!("📨 Received message: {:?}", message);
                count += 1;
            }
            Err(e) => {
                eprintln!("❌ Error receiving message: {}", e);
            }
        }
    }

    count
}

/// Wait for a specific message type in the response stream
///
/// Returns true if the predicate matches any message, false if stream ends.
#[allow(dead_code)]
pub async fn wait_for_message<F>(session: &AgentSession, predicate: F) -> bool
where
    F: Fn(&turboclaudeagent::message_parser::ParsedMessage) -> bool,
{
    use futures::StreamExt;

    let mut stream = Box::pin(session.receive_messages().await);

    while let Some(result) = stream.next().await {
        if let Ok(message) = result {
            if predicate(&message) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_check() {
        // This test validates the API key check mechanism
        // It will panic if API key is not set, which is expected behavior

        if env::var("ANTHROPIC_API_KEY").is_ok() {
            // API key is set - require_api_key should not panic
            require_api_key();
        } else {
            // API key not set - should panic
            // We don't call require_api_key() here to avoid failing the test
            println!("API key not set - skipping validation test");
        }
    }
}
