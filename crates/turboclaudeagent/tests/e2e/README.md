# End-to-End Tests for TurboClaude Agent SDK

## Overview

This directory contains end-to-end tests that make **real API calls to Claude** to verify production behavior. These tests are modeled after the Python SDK's e2e-tests directory and ensure feature parity.

## Running E2E Tests

### Prerequisites

1. **API Key Required**: Set `ANTHROPIC_API_KEY` environment variable:
 ```bash
 export ANTHROPIC_API_KEY=your-key-here
 ```

2. **Cost Warning**: These tests make real API calls and will consume API credits. Run sparingly.

### Execute Tests

```bash
# Run all E2E tests (requires API key)
cargo test --test e2e_* -- --test-threads=1 --nocapture

# Run specific E2E test file
cargo test --test e2e_dynamic_control -- --nocapture

# Run with logging
RUST_LOG=debug cargo test --test e2e_hooks -- --nocapture
```

### CI/CD Configuration

E2E tests are **disabled by default** in CI to avoid API costs. They can be enabled by:

1. Setting `RUN_E2E_TESTS=true` environment variable
2. Providing `ANTHROPIC_API_KEY` secret in CI

## Test Categories

### 1. Dynamic Control (`e2e_dynamic_control.rs`)

Tests runtime configuration changes:
- `test_set_permission_mode`: Change permission mode during session
- `test_set_model`: Switch between Claude models dynamically
- `test_interrupt`: Send interrupt signals

**Python Parity**: Complete parity with `test_dynamic_control.py`

### 2. Hooks (`e2e_hooks.rs`)

Tests hook callback system:
- `test_hook_with_permission_decision_and_reason`: PreToolUse hooks with allow/deny
- `test_hook_with_continue_and_stop_reason`: PostToolUse hooks that halt execution
- `test_hook_with_additional_context`: Hooks providing hookSpecificOutput

**Python Parity**: Complete parity with `test_hooks.py`

### 3. Tool Permissions (`e2e_tool_permissions.rs`)

Tests tool permission callbacks:
- `test_permission_callback_gets_called`: Verify can_use_tool invocation

**Python Parity**: Complete parity with `test_tool_permissions.py`

### 4. Agents and Settings (`e2e_agents_and_settings.rs`)

Tests agent configuration:
- `test_agent_definition`: Custom agent definitions
- `test_setting_sources_default`: Default settings behavior
- `test_setting_sources_user_only`: User-only setting sources
- `test_setting_sources_project_included`: Project setting sources

**Python Parity**: Complete parity with `test_agents_and_settings.py`

## Infrastructure

### Shared Utilities (`common/mod.rs`)

- `require_api_key()`: Skip tests if API key not present
- `create_test_client()`: Factory for E2E test clients
- `wait_for_response()`: Helper to consume response streams

### Design Principles

1. **Real Subprocess Transport**: Use actual Claude CLI subprocess, not mocks
2. **API Key Validation**: Tests automatically skip if `ANTHROPIC_API_KEY` not set
3. **Minimal Assertions**: Focus on "does it work?" not "exact output validation"
4. **Idempotent**: Tests should be runnable multiple times without side effects
5. **Isolated**: Each test creates independent session, no shared state

## Comparison with Python SDK

| Python E2E Test | Rust E2E Test | Status |
|----------------|---------------|--------|
| `test_dynamic_control.py` | `e2e_dynamic_control.rs` | Parity |
| `test_hooks.py` | `e2e_hooks.rs` | Parity |
| `test_tool_permissions.py` | `e2e_tool_permissions.rs` | Parity |
| `test_agents_and_settings.py` | `e2e_agents_and_settings.rs` | Parity |
| `test_stderr_callback.py` | ️ TODO | Not implemented |
| `test_sdk_mcp_tools.py` | ️ TODO | Not implemented |
| `test_include_partial_messages.py` | ️ TODO | Not implemented |

## Test Development Guidelines

### Adding New E2E Tests

1. **Create test file**: `tests/e2e/e2e_<category>.rs`
2. **Add to mod.rs**: Include in `tests/e2e/mod.rs`
3. **Use common helpers**: Import from `common::*`
4. **Require API key**: Call `require_api_key()` at test start
5. **Keep it simple**: Test real-world scenarios, not edge cases
6. **Document parity**: Note corresponding Python SDK test

### Example Test Structure

```rust
use super::common::*;

#[tokio::test]
async fn test_example_scenario() {
 require_api_key();

 let config = SessionConfig::default();
 let mut session = AgentSession::new(config).await.unwrap();

 session.query("Simple test query").await.unwrap();

 while let Some(message) = session.receive_response().await {
 println!("Got message: {:?}", message);
 }

 // Minimal assertion - just verify it worked
 assert!(session.is_complete());
}
```

## Debugging E2E Tests

### Common Issues

1. **API Key Not Found**
 - Error: Test skipped or panics
 - Fix: `export ANTHROPIC_API_KEY=your-key`

2. **Subprocess Spawn Failure**
 - Error: "Failed to spawn subprocess"
 - Fix: Ensure Claude CLI is installed and in PATH

3. **Timeout**
 - Error: Test hangs
 - Fix: Check API rate limits, network connectivity

4. **Flaky Tests**
 - Issue: Non-deterministic Claude responses
 - Fix: Reduce assertions to structural checks, not content

### Verbose Output

```bash
# Enable all logging
RUST_LOG=trace cargo test --test e2e_hooks -- --nocapture

# Enable only agent SDK logging
RUST_LOG=turboclaudeagent=debug cargo test --test e2e_hooks -- --nocapture
```

## Cost Management

**Estimated Costs (as of 2024):**
- Full E2E suite: ~$0.10-0.50 per run (depends on models used)
- Individual test: ~$0.01-0.05

**Best Practices:**
1. Run E2E tests **before releases only**, not on every commit
2. Use integration tests (with mocks) for rapid development
3. Consider using `max_turns=1` to limit API usage in tests
4. Set up CI to run E2E tests on manual trigger or scheduled basis

## Future Enhancements

- [ ] Add remaining Python SDK E2E test parity (stderr, MCP tools, partial messages)
- [ ] Implement E2E test result caching (avoid redundant API calls)
- [ ] Add performance benchmarks (latency, token usage)
- [ ] Create E2E test matrix for different models (Opus, Sonnet, Haiku)
- [ ] Add E2E tests for error recovery and retry logic
