# TurboClaude Real-World Tests

## Overview

This directory contains real-world integration tests that make actual API calls to Anthropic's Claude API. These tests validate that TurboClaude works correctly in production scenarios, beyond unit tests with mocked responses.

## ️ Important Notes

- **These tests use your actual API key** and will make real API calls
- **Tests cost money** (~$0.50-$1.00 per full run, see cost breakdown below)
- **Tests are marked `#[ignore]`** to prevent accidental execution
- **Tests require `ANTHROPIC_API_KEY`** environment variable to be set

## Quick Start

### 1. Set Your API Key

```bash
export ANTHROPIC_API_KEY="your-api-key-here"
```

### 2. Run Tests

```bash
# Run all real-world tests
cargo test --ignored real_world

# Run specific category
cargo test --ignored real_world_messages
cargo test --ignored real_world_streaming
cargo test --ignored real_world_errors

# Run with output (recommended)
cargo test --ignored real_world -- --nocapture

# Run single-threaded to avoid rate limits
cargo test --ignored real_world -- --test-threads=1 --nocapture
```

## Test Categories

### Implemented & Working

#### 1. Messages API (`tests/real_world/messages.rs`)

Tests core message functionality with actual Claude API calls.

**Tests**:
- `real_world_messages_simple` - Basic request/response
- `real_world_messages_conversation` - Multi-turn conversation with context
- `real_world_messages_system_prompt` - System prompt behavior
- `real_world_messages_haiku` - Claude 3 Haiku model
- `real_world_messages_metadata` - Response metadata validation
- `real_world_messages_max_tokens_limit` - Token limiting behavior

**Run**:
```bash
cargo test --ignored real_world_messages -- --nocapture
```

**Estimated cost**: ~$0.10 (6 tests with small token counts)

#### 2. Streaming API (`tests/real_world/streaming.rs`)

Tests server-sent events (SSE) streaming with real responses.

**Tests**:
- `real_world_streaming_basic` - Basic streaming functionality
- `real_world_streaming_long_response` - Long response with multiple chunks
- `real_world_streaming_all_events` - Validate all event types
- `real_world_streaming_metadata` - Stream metadata extraction
- `real_world_streaming_performance` - TTFB and latency metrics

**Run**:
```bash
cargo test --ignored real_world_streaming -- --nocapture
```

**Estimated cost**: ~$0.10 (5 tests with streaming responses)

**Metrics collected**:
- Time to first byte (TTFB)
- Chunk count and sizes
- Total streaming duration
- Event counts

#### 3. Error Handling (`tests/real_world/errors.rs`)

Tests error scenarios and edge cases.

**Tests**:
- `real_world_errors_invalid_api_key` - Authentication failure
- `real_world_errors_invalid_model` - Model not found
- `real_world_errors_missing_required_field` - Validation errors
- `real_world_errors_invalid_max_tokens` - Parameter validation
- `real_world_errors_timeout_simulation` - Network timeout handling
- `real_world_errors_invalid_base_url` - URL validation
- `real_world_errors_empty_content` - Edge case handling

**Run**:
```bash
cargo test --ignored real_world_errors -- --nocapture
```

**Estimated cost**: ~$0.05 (most tests fail quickly before using tokens)

### Draft/TODO

The following test suites have been created but require type/API fixes before they can be enabled:

#### 4. Tool Use (`tests/real_world/tools.rs.draft`)

**Status**: Requires `schema` feature flag and type fixes

**Planned tests**:
- Basic tool calling with JSON parameters
- Tool result continuation
- Multiple tool definitions
- Forced tool use
- Complex nested schemas

**Enable when ready**: Uncomment in `tests/real_world.rs` and enable `schema` feature

#### 5. Prompt Caching (`tests/real_world/caching.rs.draft`)

**Status**: Requires SystemPrompt API fixes

**Planned tests**:
- Basic cache creation and reuse
- Large document caching
- Multiple cache breakpoints
- Mixed cached/uncached content
- Cost comparison analysis

**Enable when ready**: Fix `SystemPrompt` / `SystemPromptBlock` usage patterns

#### 6. Batch API (`tests/real_world/batches.rs.draft`)

**Status**: Requires Batch API implementation in SDK

**Planned tests**:
- Small batch processing (2 requests)
- Batch listing and pagination
- Batch cancellation
- Error handling in batches
- Medium batch (10+ requests)

**Enable when ready**: Implement `client.batches()` API

## Test Structure

### Common Utilities (`tests/real_world/common.rs`)

Shared test infrastructure:

```rust
/// Test configuration from environment
pub struct TestConfig {
 pub api_key: String,
}

/// Metrics collection for test runs
pub struct TestMetrics {
 pub start_time: Option<Instant>,
 pub input_tokens: u32,
 pub output_tokens: u32,
 pub cache_creation_tokens: u32,
 pub cache_read_tokens: u32,
 pub event_count: usize,
}
```

### Test Pattern

All tests follow this pattern:

```rust
#[tokio::test]
#[ignore] // Prevent accidental execution
async fn real_world_test_name() -> Result<(), Box<dyn std::error::Error>> {
 // 1. Load config
 let config = TestConfig::from_env()?;
 let client = Client::new(&config.api_key);
 let mut metrics = TestMetrics::new();

 // 2. Execute test
 println!("\n Testing: Description");
 let response = client.messages().create(...).await?;

 // 3. Validate response
 assert!(...);
 println!(" Validation passed");

 // 4. Report metrics
 metrics.finish();
 metrics.print_summary();

 Ok(())
}
```

## Cost Estimation

Approximate costs per test category (as of 2025):

| Category | Tests | Tokens (avg) | Est. Cost |
|----------|-------|--------------|-----------|
| Messages | 6 | ~2,000 | $0.10 |
| Streaming | 5 | ~2,500 | $0.10 |
| Errors | 7 | ~500 | $0.05 |
| Tools (draft) | 6 | ~4,000 | $0.20 |
| Caching (draft) | 5 | ~6,000 | $0.25 |
| Batches (draft) | 6 | ~5,000 | $0.30 |
| **Total** | **35** | **~20,000** | **~$1.00** |

**Notes**:
- Costs based on Claude 3.5 Sonnet pricing (~$3/MTok input, ~$15/MTok output)
- Caching tests cost more initially (cache creation) but save on subsequent calls
- Error tests cost less (most fail before generating output)
- Batch tests bundle multiple requests (cheaper per request)

## Best Practices

### Running Tests Safely

1. **Use a test-specific API key** with budget limits
2. **Run incrementally** - test one category at a time
3. **Use `--test-threads=1`** to avoid rate limiting
4. **Monitor output** with `--nocapture` flag
5. **Check costs** in Anthropic console after running

### Rate Limiting

Anthropic enforces rate limits. To avoid hitting them:

```bash
# Single-threaded execution (recommended)
cargo test --ignored real_world -- --test-threads=1

# Add delays between tests if needed (edit test code)
tokio::time::sleep(Duration::from_secs(1)).await;
```

### CI/CD Integration

**Option 1: Scheduled Runs**

```yaml
# .github/workflows/real-world-tests.yml
name: Real World Tests
on:
 schedule:
 - cron: '0 0 * * 0' # Weekly on Sunday
 workflow_dispatch: # Manual trigger

jobs:
 test:
 runs-on: ubuntu-latest
 steps:
 - uses: actions/checkout@v3
 - uses: actions-rs/toolchain@v1
 - name: Run real-world tests
 env:
 ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
 run: |
 cargo test --ignored real_world -- --test-threads=1
```

**Option 2: Pre-Release Verification**

Run before each release to ensure production readiness:

```bash
# In release script
export ANTHROPIC_API_KEY="$RELEASE_TEST_API_KEY"
cargo test --ignored real_world -- --test-threads=1 || exit 1
```

## Troubleshooting

### "ANTHROPIC_API_KEY not set"

```bash
export ANTHROPIC_API_KEY="sk-ant-your-key-here"
```

Or create `.env` file (don't commit!):
```
ANTHROPIC_API_KEY=sk-ant-your-key-here
```

### Rate Limit Errors (429)

Reduce concurrency:
```bash
cargo test --ignored real_world -- --test-threads=1
```

Or add delays in test code.

### Authentication Errors (401)

- Check API key is valid
- Verify key has correct permissions
- Ensure no extra whitespace in key

### Timeout Errors

Increase timeout in client builder:
```rust
let client = Client::builder()
 .api_key(&api_key)
 .timeout(Duration::from_secs(30)) // Increase from default
 .build()?;
```

## Metrics & Reporting

Each test prints detailed metrics:

```
=== Test Metrics ===
Duration: 1.234s
Input tokens: 156
Output tokens: 89
Stream events: 12
TTFB: 234ms
====================
```

### Collecting Results

Redirect output to file for analysis:
```bash
cargo test --ignored real_world -- --nocapture 2>&1 | tee test_results.txt
```

Parse results:
```bash
grep "" test_results.txt # All validations
grep "Input tokens" test_results.txt | awk '{sum+=$3} END {print sum}' # Total tokens
```

## Contributing

### Adding New Tests

1. Create test function in appropriate file
2. Follow the standard test pattern (see above)
3. Add `#[ignore]` attribute
4. Document expected behavior and cost
5. Add to test count in this README

### Test Naming Convention

```
real_world_<category>_<specific_test>
```

Examples:
- `real_world_messages_simple`
- `real_world_streaming_performance`
- `real_world_tools_multiple`

## Future Work

- [ ] Enable tool use tests (requires `schema` feature)
- [ ] Fix prompt caching tests (SystemPrompt API)
- [ ] Implement batch API support
- [ ] Add response time assertions (SLA validation)
- [ ] Create test result dashboard
- [ ] Add cost tracking/reporting
- [ ] Implement test data fixtures
- [ ] Add retry logic for flaky network

## References

- [TurboClaude Documentation](../README.md)
- [Real-World Testing Strategy](../REAL_WORLD_TESTING.md)
- [Anthropic API Documentation](https://docs.anthropic.com/)
- [Claude Pricing](https://www.anthropic.com/pricing)
