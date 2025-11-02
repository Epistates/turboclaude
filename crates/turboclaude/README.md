# TurboClaude Rust SDK

[![Crates.io](https://img.shields.io/crates/v/turboclaude.svg)](https://crates.io/crates/turboclaude)
[![Docs.rs](https://docs.rs/turboclaude/badge.svg)](https://docs.rs/turboclaude)
[![License](https://img.shields.io/crates/l/turboclaude.svg)](LICENSE)

Rust SDK for Anthropic's Claude API with support for multi-cloud providers (AWS Bedrock, Google Vertex AI), agent framework, and tool system.

## Features

### Supported Providers
- Anthropic (official API)
- AWS Bedrock
- Google Vertex AI

### API Coverage
- Messages API with streaming
- Batch processing
- Tool use and function calling
- Prompt caching
- Document and PDF analysis
- Vision capabilities
- Token counting
- Models API

### Agent Framework
- Agent client and configuration
- Message hooks and interception
- Permission system
- Message routing
- Skills integration

### Tools & Extensions
- Custom tool definitions
- Computer use (beta)
- Bash execution (beta)
- Web search (beta)
- Code execution (beta)

### Advanced Features
- Real-time streaming
- Prompt caching for cost reduction
- Persistent memory (beta)
- Extended thinking (beta)

## Quick Start

```rust
use turboclaude::Client;
use turboclaude::types::{MessageRequest, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
 // Create client (uses ANTHROPIC_API_KEY environment variable)
 let client = Client::new("sk-ant-...");

 // Send a message
 let response = client.messages()
 .create(MessageRequest::builder()
 .model(turboclaude::types::Models::CLAUDE_SONNET_4_5)
 .max_tokens(1024u32)
 .messages(vec![Message::user("Hello, Claude!")])
 .build()?)
 .await?;

 // Print response
 for content in response.content {
 if let turboclaude::types::ContentBlock::Text { text, .. } = content {
 println!("{}", text);
 }
 }

 Ok(())
}
```

## Multi-Cloud Usage

### AWS Bedrock
```rust
use turboclaude::{Client, providers::bedrock::BedrockHttpProvider};
use std::sync::Arc;

let provider = Arc::new(
 BedrockHttpProvider::builder()
 .region("us-east-1")
 .build()
 .await?
);

let client = Client::from_provider(provider);
// Use same client API!
```

### Google Vertex AI
```rust
use turboclaude::{Client, providers::vertex::VertexHttpProvider};
use std::sync::Arc;

let provider = Arc::new(
 VertexHttpProvider::builder()
 .project_id("my-gcp-project")
 .region("us-east5")
 .access_token(token)
 .build()
 .await?
);

let client = Client::from_provider(provider);
// Use same client API!
```

## Features & Flags

Enable optional functionality via Cargo features:

```toml
[dependencies]
turboclaude = { version = "0.1", features = ["bedrock", "vertex", "schema"] }
```

Available features:
- `env`: Load API key from environment variables (default)
- `bedrock`: AWS Bedrock provider support
- `vertex`: Google Vertex AI provider support
- `schema`: JSON schema generation for tools
- `mcp`: Model Context Protocol integration
- `trace`: Tracing/logging support
- `full`: All features except blocking

## Architecture

TurboClaude is built on a modular architecture:

- **turboclaude**: Main SDK with multi-cloud providers
- **turboclaude-transport**: HTTP and subprocess transport
- **turboclaude-protocol**: Protocol types and messages
- **turboclaude-core**: Core abstractions and traits
- **turboclaudeagent**: Agent framework with hooks/permissions
- **turboclaude-skills**: Skills system for dynamic capabilities
- **turboclaude-mcp**: Model Context Protocol integration

## Examples

Check the `examples/` directory for:
- `basic.rs` - Simple message sending
- `streaming.rs` - Real-time streaming
- `tools.rs` - Tool use and function calling
- `bedrock_basic.rs` - AWS Bedrock usage
- `vertex_basic.rs` - Google Vertex AI usage
- And more...

Run examples with:
```bash
cargo run --example basic --features env,trace
```

## Testing

```bash
# Run all tests
cargo test --all

# Run with output
cargo test --all -- --nocapture

# Run specific test
cargo test --lib test_name
```

## Documentation

- Full API documentation: `cargo doc --open`
- Architecture guide: See module-level documentation
- Implementation details: Check individual crate READMEs

## Performance

TurboClaude is optimized for production:
- Zero-copy message handling
- Streaming support to reduce memory
- Configurable timeouts and retries
- Connection pooling via reqwest

## Status

v0.1.0 - All tests passing. See main README for test results.

## License

MIT/Apache-2.0

## Contributing

Contributions welcome! Please see CONTRIBUTING.md (when created).

## Support

For issues and feature requests, see the GitHub repository.

---

**Part of the TurboClaude Rust Ecosystem** 
