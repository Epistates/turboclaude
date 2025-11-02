//! Common test utilities and fixtures
//!
//! Uses best-in-class Rust testing libraries:
//! - rstest for fixtures (matches Python's pytest fixtures)
//! - wiremock for HTTP mocking (isolated, parallel-safe)
//! - #[tokio::test] for async testing
//! - insta for snapshot testing

pub mod fixtures;
pub mod responses;

// Re-export commonly used test utilities
pub use responses::{
    // Message responses
    message_response,
    message_with_tool_use,
    token_count_response,
    // Batch responses
    batch_create_response,
    batch_completed_response,
    batch_results_jsonl,
    // Model responses
    model_list_response,
    model_get_response,
    // Error responses
    error_invalid_request,
    error_authentication,
    error_rate_limit,
    // Streaming responses
    sse_text_stream,
    sse_tool_use_stream,
};

