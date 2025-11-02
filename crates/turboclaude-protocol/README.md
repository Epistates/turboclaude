# TurboClaude Protocol - Type System and Protocol Definitions

Comprehensive type definitions and protocol structures for Claude API communication, including messages, content types, tools, and agent-specific constructs.

## Features

- **Message Types**: Complete request/response types
- **Content Blocks**: Text, images, documents, tool use
- **Tool Definitions**: Function calling specifications
- **Agent Types**: Hooks, permissions, routing
- **Streaming Support**: Event-based message streaming
- **Type Safety**: Full Rust type system

## Quick Start

```rust
use turboclaude_protocol::types::{MessageRequest, Message, Role};

fn build_request() -> Result<MessageRequest, Box<dyn std::error::Error>> {
    let request = MessageRequest::builder()
        .model(turboclaude_protocol::types::Models::CLAUDE_SONNET_4_5)
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello, Claude!")])
        .build()?;

    Ok(request)
}
```

## Components

### Message Types
Request and response types for API communication.

### Content Types
Text, images, documents, tool use, and tool results.

### Tool Definitions
Function calling specifications and tool schemas.

### Agent Types
Hooks, permissions, and routing constructs.

### Streaming Events
Real-time message streaming event types.

## Architecture

```
turboclaude-protocol
├── message      (Message types)
├── content      (Content block types)
├── tool         (Tool definitions)
├── agent        (Agent-specific types)
├── hooks        (Hook system types)
├── permissions  (Permission types)
└── error        (Error types)
```

## Testing

```bash
cargo test
```

47 tests covering all protocol types.

## Documentation

Full API docs: `cargo doc --open`

---

**Part of TurboClaude** 📋
