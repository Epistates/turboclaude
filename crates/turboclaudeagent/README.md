# TurboClaude Agent Framework

Agent framework built on TurboClaude for creating AI agents with message hooks, permissions, routing, and skill integration.

## Documentation

- **[API Reference](./API_REFERENCE.md)** - Complete API documentation with examples
- **[Performance Tuning](./PERFORMANCE_TUNING.md)** - Optimization guide and best practices
- **[Troubleshooting](./TROUBLESHOOTING.md)** - Common issues and solutions

## Features

- Agent client with routing and message parsing
- Message hooks for interception and modification
- Permission system for capability control
- Skills integration and execution
- Message routing and filtering
- Session management

## Quick Start

```rust
use turboclaudeagent::Agent;
use turboclaude::types::MessageRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
 let agent = Agent::builder()
 .api_key("sk-ant-...")
 .name("MyAgent")
 .build()?;

 let response = agent.query("What is 2+2?").await?;
 println!("{}", response);

 Ok(())
}
```

## Components

### Agent Client
Main entry point for agent interactions with configurable behavior.

### Hooks
Intercept and modify:
- `message_start`: Before sending messages
- `message_stop`: After receiving messages
- `tool_call`: Before executing tools
- `tool_result`: After tool execution

### Permissions
Control agent capabilities:
- Tool execution permissions
- Resource access control
- Rate limiting

### Skills
Dynamic skill registration and execution:
- Register skills from external systems
- Execute skills based on agent needs
- Manage skill lifecycle

## Architecture

```
turboclaudeagent
├── client (Agent client implementation)
├── config (Configuration builder)
├── hooks (Extensible hook system)
├── permissions (Permission control)
├── routing (Message routing logic)
├── skills (Skills integration)
└── message_parser (Advanced message parsing)
```

## Examples

See `examples/` for:
- `simple_query.rs` - Basic agent usage
- `with_hooks.rs` - Hook system demonstration
- `with_permissions.rs` - Permission configuration
- `with_skills.rs` - Skills integration

## Testing

```bash
cargo test --all
```

See `tests/` for comprehensive test coverage including E2E tests.

## Documentation

Full documentation: `cargo doc --open`

---

**Part of TurboClaude** 
