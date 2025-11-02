/// Real-world integration tests for TurboClaude SDK
///
/// These tests make actual API calls to Anthropic's Claude API.
///
/// ## Setup
///
/// Set your API key:
/// ```bash
/// export ANTHROPIC_API_KEY="your-api-key-here"
/// ```
///
/// ## Running Tests
///
/// Run all real-world tests:
/// ```bash
/// cargo test --ignored real_world
/// ```
///
/// Run specific category:
/// ```bash
/// cargo test --ignored real_world_messages
/// cargo test --ignored real_world_streaming
/// cargo test --ignored real_world_tools
/// cargo test --ignored real_world_caching
/// cargo test --ignored real_world_batches
/// cargo test --ignored real_world_errors
/// ```
///
/// Run with output:
/// ```bash
/// cargo test --ignored real_world -- --nocapture
/// ```
///
/// Run single-threaded to avoid rate limits:
/// ```bash
/// cargo test --ignored real_world -- --test-threads=1 --nocapture
/// ```
///
/// ## Cost Estimation
///
/// Approximate costs per full test run:
/// - Messages: ~$0.10 (6 tests)
/// - Streaming: ~$0.10 (5 tests)
/// - Tools: ~$0.20 (6 tests)
/// - Caching: ~$0.25 (5 tests)
/// - Batches: ~$0.30 (6 tests, some with multiple requests)
/// - Errors: ~$0.05 (most fail quickly)
///
/// **Total**: ~$1.00 per complete test run
///
/// ## Notes
///
/// - Tests use `#[ignore]` to prevent accidental execution
/// - Batch tests can take 5-10 minutes to complete
/// - Use `--test-threads=1` to avoid rate limiting
/// - Some error tests intentionally fail API calls
/// - All tests validate response structure and behavior

mod real_world {
    pub mod common; // Common utilities

    // Test modules - currently enabled
    mod errors;
    mod messages;
    mod streaming;

    // TODO: Enable when type issues are resolved:
    // #[cfg(feature = "schema")]
    // mod tools;
    // mod caching;
    // mod batches; (requires batches API implementation)
}
