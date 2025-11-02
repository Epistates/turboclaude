# TurboClaude - Production-Grade Rust SDK for Anthropic Claude

[![Crates.io](https://img.shields.io/crates/v/turboclaude.svg)](https://crates.io/crates/turboclaude)
[![Docs.rs](https://docs.rs/turboclaude/badge.svg)](https://docs.rs/turboclaude)
[![License](https://img.shields.io/crates/l/turboclaude.svg)](LICENSE)

**100% feature parity with the official Python SDK** — TurboClaude brings production-grade Claude API access to Rust with multi-cloud support (AWS Bedrock, Google Vertex AI), complete agent systems, and comprehensive tooling.

## Features

### 🌐 Multi-Cloud Support
- **Anthropic Direct**: Use Claude via official API
- **AWS Bedrock**: Access Claude models via Amazon Bedrock
- **Google Vertex AI**: Access Claude models via Google Cloud
- **Unified Interface**: Same `Client` API across all providers

### 🤖 Agentic Capabilities
- **Built-in Agent Framework** (turboclaudeagent crate)
- **Hooks System**: Intercept and modify messages
- **Permissions System**: Control what agents can do
- **Message Routing**: Sophisticated message handling
- **Skills Framework**: Register and execute dynamic skills

### 🛠️ Comprehensive Tooling
- **Tool System**: Define and use custom tools
- **Computer Use**: GUI interaction (beta)
- **Bash Execution**: Shell command running (beta)
- **Web Search**: Real-time information retrieval (beta)
- **Code Execution**: Execute code safely (beta)

### 💾 Advanced Features
- **Streaming**: Real-time token streaming
- **Prompt Caching**: Reduce API costs
- **File Uploads**: Process documents and images
- **Vision**: Claude 3.5 vision capabilities
- **Memory API**: Persistent context across conversations (beta)

### 📊 Beta APIs
- Extended Thinking
- Vision (citations)
- Files API
- Models API
- Memory API (persistent context)

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

✅ **Production Ready** — All tests passing, comprehensive feature coverage, suitable for production use.

## License

MIT/Apache-2.0

## Contributing

Contributions welcome! Please see CONTRIBUTING.md (when created).

## Support

For issues and feature requests, see the GitHub repository.

---

**Part of the TurboClaude Rust Ecosystem** 🦀
