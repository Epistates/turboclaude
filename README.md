# TurboClaude

**⚠️ UNOFFICIAL SDK** - This is an unofficial, community-maintained Rust SDK for Anthropic's Claude API. It is not affiliated with or endorsed by Anthropic.

**Maintained by:** [epistates](https://github.com/epistates)

A fully-featured, idiomatic Rust SDK for Anthropic's Claude API with complete feature parity to the official Python SDK.

## 🚀 Quick Start

```bash
# Add to your Cargo.toml
[dependencies]
turboclaude = "0.1"

# Set your API key
export ANTHROPIC_API_KEY=sk-ant-...

# Run an example
cargo run --example basic
```

See `examples/` directory for comprehensive usage patterns.

## 📚 Which Crate Do I Need?

Coming from Python? Here's the mapping to find the right TurboClaude crate:

| Use Case | Python Package | Rust Crate | Purpose |
|----------|---|---|---|
| **Claude API Client** | `anthropic` | [`turboclaude`](./crates/turboclaude/README.md) | REST API client for Claude messages, batches, models |
| **Agent Framework** | `anthropic.agent` | [`turboclaudeagent`](./crates/turboclaudeagent/README.md) | Build interactive agents with hooks, permissions, routing |
| **Skills & Tools** | `anthropic.tools` | [`turboclaude-skills`](./crates/turboclaude-skills/README.md) | Dynamic skill registration and execution system |
| **Protocol Types** | (type definitions) | [`turboclaude-protocol`](./crates/turboclaude-protocol/README.md) | Shared message types, content blocks, tools |
| **Transport Layer** | (internal) | [`turboclaude-transport`](./crates/turboclaude-transport/README.md) | HTTP and subprocess transport abstractions |
| **Core Abstractions** | (internal) | [`turboclaude-core`](./crates/turboclaude-core/README.md) | Base traits, builders, error handling |
| **MCP Integration** | (via external) | [`turboclaude-mcp`](./crates/turboclaude-mcp/README.md) | Model Context Protocol adapter and factory |

### Examples by Use Case

**Just need to send messages?**
```toml
[dependencies]
turboclaude = "0.1"
```

**Building an AI agent with custom behavior?**
```toml
[dependencies]
turboclaudeagent = "0.1"
turboclaude-skills = "0.1"
```

**Integrating with Model Context Protocol servers?**
```toml
[dependencies]
turboclaude = { version = "0.1", features = ["mcp"] }
turboclaude-mcp = "0.1"
```

**Everything (AWS Bedrock, Vertex AI, all features)?**
```toml
[dependencies]
turboclaude = { version = "0.1", features = ["bedrock", "vertex", "schema", "mcp", "trace"] }
turboclaudeagent = "0.1"
turboclaude-skills = "0.1"
```

## 🎯 Design Philosophy

This SDK is designed to provide a Rust-native experience while maintaining feature parity with the official Python SDK. Our architecture decisions prioritize:

1. **Type Safety**: Leveraging Rust's type system for compile-time guarantees
2. **Performance**: Zero-cost abstractions and efficient streaming
3. **Ergonomics**: Intuitive API that feels natural to Rust developers
4. **Compatibility**: Feature parity with the Python SDK
5. **Extensibility**: Easy integration with MCP and other protocols

## 🏗️ Architecture

### Core Design Decisions

#### 1. **Async-First with Optional Blocking**
- **Decision**: Single async implementation with optional blocking wrapper
- **Rationale**: Rust's async is zero-cost; blocking can be added as a thin wrapper
- **Trade-off**: Slightly more complex for sync-only users, but better performance and maintainability

```rust
// Async (default)
let client = Client::new("sk-...");
let msg = client.messages().create(request).await?;

// Blocking (optional feature)
#[cfg(feature = "blocking")]
let client = turboclaude::blocking::Client::new("sk-...");
let msg = client.messages().create(request)?;
```

#### 2. **Lazy Resource Initialization (like Python's @cached_property)**
- **Decision**: Use `OnceCell` for lazy resource initialization
- **Rationale**: Avoids circular dependencies, reduces initial overhead
- **Implementation**: Resources are created on first access

```rust
// Resources are lazily initialized
client.messages()  // First call creates Messages resource
client.messages()  // Subsequent calls reuse cached instance
```

#### 3. **Three Response Modes (matching Python SDK)**
- **Decision**: Provide standard, raw, and streaming responses
- **Rationale**: Different use cases need different levels of control
- **Implementation**: Type-state pattern ensures compile-time correctness

```rust
// Standard (parsed)
let message = client.messages().create(request).await?;

// Raw (with headers)
let response = client.messages().with_raw_response().create(request).await?;

// Streaming (SSE)
let stream = client.messages().stream(request).await?;
```

#### 4. **Type-Safe Builder Pattern**
- **Decision**: Use `derive_builder` for request construction
- **Rationale**: Provides IDE autocomplete and compile-time validation
- **Trade-off**: Additional macro complexity for better ergonomics

```rust
use turboclaude::types::Models;

let request = MessageRequest::builder()
    .model(Models::CLAUDE_SONNET_4_5)
    .max_tokens(1024)
    .messages(vec![Message::user("Hello")])
    .temperature(0.7)  // Optional fields
    .build()?;         // Validates required fields
```

#### 5. **Error Hierarchy with Context**
- **Decision**: Rich error types using `thiserror` with context chains
- **Rationale**: Detailed error information for debugging
- **Pattern**: Mirrors Python exception hierarchy but uses Result<T, E>

```rust
match client.messages().create(request).await {
    Err(Error::RateLimit { retry_after, .. }) => {
        // Handle rate limiting with retry information
    }
    Err(Error::Authentication(msg)) => {
        // Handle auth errors
    }
    Ok(message) => {
        // Success
    }
}
```

#### 6. **Streaming with Futures**
- **Decision**: Use `futures::Stream` for SSE streaming
- **Rationale**: Standard async streaming interface in Rust
- **Features**: High-level conveniences like `text_stream()` and `get_final_message()`

```rust
// Low-level event stream
let mut stream = client.messages().stream(request).await?;
while let Some(event) = stream.next().await {
    match event? {
        StreamEvent::ContentDelta(delta) => print!("{}", delta.text),
        StreamEvent::MessageStop => break,
        _ => {}
    }
}

// High-level text stream
let text_stream = client.messages().stream(request).await?.text_stream();
pin_mut!(text_stream);
while let Some(text) = text_stream.next().await {
    print!("{}", text?);
}
```

#### 7. **Middleware Stack Architecture**
- **Decision**: Tower-compatible middleware stack
- **Rationale**: Composable, reusable, ecosystem-compatible
- **Examples**: Retry, rate limiting, tracing, custom middleware

```rust
// Middleware is composable and configurable
let client = Client::builder()
    .middleware(RetryMiddleware::new())
    .middleware(RateLimitMiddleware::new(10.0))
    .middleware(TracingMiddleware::new())
    .build()?;
```

#### 8. **MCP Integration (Optional)**
- **Decision**: Use TurboMCP for MCP protocol support
- **Rationale**: Best-in-class MCP implementation, maintained actively
- **Pattern**: Optional feature flag to avoid unnecessary dependencies

```rust
#[cfg(feature = "mcp")]
{
    // Consume MCP tools
    let mcp_client = McpToolClient::connect("http://mcp-server:8080").await?;
    let tools = mcp_client.list_tools().await?;

    // Use with Claude
    let message = client.messages()
        .create(MessageRequest::builder()
            .tools(tools)
            .build()?)
        .await?;
}
```

## 📦 Module Structure

```
src/
├── lib.rs              # Public API exports
├── client.rs           # Main Client implementation
├── config.rs           # Configuration management
├── error.rs            # Error types hierarchy
├── types.rs            # Core type definitions
├── http/               # HTTP layer
│   ├── mod.rs         # HTTP client trait
│   ├── request.rs     # Request builder
│   ├── response.rs    # Response wrapper
│   ├── retry.rs       # Retry logic
│   └── middleware.rs  # Middleware stack
├── resources/          # API endpoints
│   ├── messages.rs    # Messages API
│   ├── completions.rs # Legacy completions
│   ├── models.rs      # Models endpoint
│   └── beta.rs        # Beta features
└── streaming.rs        # SSE streaming support
```

## 🔄 Key Trade-offs

### 1. **No Sync/Async Duplication**
- **Pro**: Cleaner codebase, easier maintenance
- **Con**: Requires tokio for all users
- **Mitigation**: Blocking wrapper available

### 2. **Generated Types from OpenAPI**
- **Pro**: Always up-to-date with API changes
- **Con**: Build complexity with code generation
- **Future**: Add build.rs for automatic generation

### 3. **Required Dependencies**
- **Pro**: Consistent experience, well-tested paths
- **Con**: Larger binary size
- **Mitigation**: Feature flags for optional components

### 4. **Type-Safe Everything**
- **Pro**: Compile-time guarantees, IDE support
- **Con**: More verbose than dynamic languages
- **Benefit**: Catches errors before runtime

## 🚀 Usage Examples

### Basic Message Creation
```rust
use turboclaude::{Client, MessageRequest, Message, types::Models};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("sk-ant-...");

    let message = client.messages()
        .create(MessageRequest::builder()
            .model(Models::CLAUDE_SONNET_4_5)
            .max_tokens(1024)
            .messages(vec![
                Message::user("What is the capital of France?")
            ])
            .build()?)
        .await?;

    println!("{}", message.content.text());
    Ok(())
}
```

### Streaming Response
```rust
use turboclaude::{Client, MessageRequest, Message, types::Models};
use turboclaude::streaming::StreamEvent;
use futures::StreamExt;

let mut stream = client.messages()
    .stream(MessageRequest::builder()
        .model(Models::CLAUDE_SONNET_4_5)
        .messages(vec![Message::user("Tell me a story")])
        .build()?)
    .await?;

while let Some(event) = stream.next().await {
    if let Ok(StreamEvent::ContentBlockDelta(event)) = event {
        if let Some(text) = event.delta.text {
            print!("{}", text);
        }
    }
}
```

### Tool Use
```rust
use turboclaude::Tool;
use serde_json::json;

let tools = vec![
    Tool::new(
        "get_weather",
        "Get the weather for a location",
        json!({
            "type": "object",
            "properties": {
                "location": {"type": "string"}
            },
            "required": ["location"]
        })
    )
];

let message = client.messages()
    .create(MessageRequest::builder()
        .model(Models::CLAUDE_SONNET_4_5)
        .tools(tools)
        .messages(vec![
            Message::user("What's the weather in Paris?")
        ])
        .build()?)
    .await?;

// Handle tool calls in response
for block in &message.content {
    if let Some((id, name, input)) = block.as_tool_use() {
        println!("Tool call: {} with {}", name, input);
    }
}
```

### Prompt Caching (Cost Optimization)
```rust
use turboclaude::{SystemPromptBlock, CacheTTL};

// Cache static system prompts to reduce costs by ~90%
let message = client.messages()
    .create(MessageRequest::builder()
        .model(Models::CLAUDE_SONNET_4_5)
        .max_tokens(1024)
        .messages(vec![Message::user("Review this code")])
        .system(vec![
            // Large static context (cached for 1 hour)
            SystemPromptBlock::text_cached_with_ttl(
                "You are an expert code reviewer. You review for:\n\
                 - Correctness and bugs\n\
                 - Performance issues\n\
                 - Security vulnerabilities...",
                CacheTTL::OneHour,
            ),
            // Repository-specific context (cached 5 minutes)
            SystemPromptBlock::text_cached(
                "This codebase uses Rust, tokio, and follows...",
            ),
            // Dynamic context (not cached)
            SystemPromptBlock::text(
                "Current task: Performance optimization",
            ),
        ])
        .build()?)
    .await?;

// First request creates cache, subsequent requests use cached tokens
// Savings: ~90% cost reduction on cached tokens!
```

### Document and PDF Analysis
```rust
use turboclaude::{ContentBlockParam, DocumentSource, CacheControl, CacheTTL};

// Analyze a PDF from URL
let message = client.messages()
    .create(MessageRequest::builder()
        .model(Models::CLAUDE_SONNET_4_5)
        .max_tokens(1024)
        .messages(vec![MessageParam {
            role: Role::User,
            content: vec![
                ContentBlockParam::Text {
                    text: "Summarize this document".to_string(),
                },
                ContentBlockParam::Document {
                    source: DocumentSource::url_pdf("https://example.com/report.pdf"),
                    cache_control: Some(CacheControl::ephemeral_with_ttl(CacheTTL::OneHour)),
                    title: Some("Q4 Report".to_string()),
                    context: Some("Financial analysis".to_string()),
                },
            ],
        }])
        .build()?)
    .await?;

// Also supports: Base64 PDFs and plain text documents
// DocumentSource::base64_pdf(base64_data)
// DocumentSource::plain_text(text_content)
```

### Response Modes - Access Headers and Metadata

The SDK provides three response modes (matching Python SDK's approach):

1. **Standard Mode** - Parsed response body only (default)
2. **Raw Response Mode** - Body + headers + HTTP metadata (**NEW!**)
3. **Streaming Mode** - Server-Sent Events (SSE) stream

#### Standard Mode (Default)

```rust
// Just get the parsed response
let message = client.messages().create(request).await?;
println!("{}", message.content.text());
```

#### Raw Response Mode - Complete Python SDK Parity

Access HTTP headers, status codes, rate limits, and metadata:

```rust
use turboclaude::{Client, MessageRequest, Message};

// Get raw response with all HTTP metadata
let raw = client.messages()
    .with_raw_response()
    .create(request)
    .await?;

// Access HTTP metadata (Python SDK parity)
println!("Status: {}", raw.status_code());           // response.status_code
println!("Request ID: {:?}", raw.request_id());      // Custom helper
println!("Elapsed: {:?}", raw.elapsed());            // response.elapsed
println!("Retries: {}", raw.retries_taken());        // response.retries_taken

// Access rate limit information
if let Some((limit, remaining, reset)) = raw.rate_limit_info() {
    println!("Rate limit: {}/{}, resets at {}", remaining, limit, reset);

    if remaining < 10 {
        eprintln!("WARNING: Only {} requests remaining!", remaining);
    }
}

// Access any header
if let Some(content_type) = raw.get_header("content-type") {
    println!("Content-Type: {:?}", content_type);
}

// Access all headers
for (name, value) in raw.headers() {
    println!("{}: {:?}", name, value);
}

// Get the parsed response
let message = raw.into_parsed();
println!("Response: {}", message.content.text());
```

#### Works on All Endpoints

```rust
// Messages
let msg = client.messages().with_raw_response().create(req).await?;

// Token counting
let tokens = client.messages().with_raw_response().count_tokens(req).await?;

// Batches
let batch = client.messages().with_raw_response().batches().create(requests).await?;
let batch = client.messages().with_raw_response().batches().get("batch_id").await?;
let batch = client.messages().with_raw_response().batches().cancel("batch_id").await?;

// Models
let models = client.models().with_raw_response().list().await?;
let model = client.models().with_raw_response().get(Models::CLAUDE_SONNET_4_5).await?;
```

#### Use Cases

1. **Rate Limiting** - Monitor API usage and avoid hitting limits
2. **Request Tracking** - Log request IDs for support/debugging
3. **Performance Monitoring** - Track API latency with `elapsed()`
4. **Retry Visibility** - See when requests were automatically retried
5. **Audit Trails** - Save complete request metadata to database
6. **Custom Headers** - Access any custom headers from responses

See `examples/response_headers.rs` for comprehensive usage patterns.

## 🔮 Future Enhancements

1. **Code Generation from OpenAPI**
   - Automated type generation via build.rs
   - Always in sync with latest API

2. **Advanced Streaming**
   - WebSocket support
   - Multiplexed streams with unified backpressure handling

## 🤝 Contributing

This SDK follows Rust best practices:
- `cargo fmt` for formatting
- `cargo clippy` for linting
- `cargo test` for testing
- `cargo doc` for documentation

## 📄 License

MIT License

See [LICENSE](LICENSE) and [DISCLAIMER.md](DISCLAIMER.md) for full details.

## 🙏 Acknowledgments

- Anthropic for the Claude API and official Python SDK (used as reference for feature parity)
- TurboMCP team for excellent MCP implementation
- Rust async ecosystem (tokio, futures, tower)

## ⚠️ Disclaimer

This is an **unofficial, community-maintained SDK**. It is not created, maintained, or endorsed by Anthropic. For official SDKs, please visit [Anthropic's documentation](https://docs.anthropic.com/).

---

## ✅ Production Ready

**Status:** Production-ready v0.1.0 - **100% Feature & Test Parity** with Python SDK

### Test Results
```
✅ Library tests:     40 passed, 0 failed
✅ Integration tests: 132 passed, 0 failed
✅ Total:             172 tests passing (100%)
✅ Warnings:          0
✅ Documentation:     15 doctests passing
```

### What's Implemented
- ✅ **Messages API** - Complete with streaming support
- ✅ **Batch Processing** - Full batch API implementation
- ✅ **Tool Use** - Function tools with automatic execution loops (`ToolRunner`)
- ✅ **Built-in Tools** - Memory tool with cache control
- ✅ **Prompt Caching** - 90% cost reduction on cached tokens
- ✅ **Document/PDF Analysis** - Base64, URL, and plain text support
- ✅ **Raw Response Mode** - Access headers, rate limits, and metadata
- ✅ **Token Counting** - Count tokens before API calls
- ✅ **Models API** - List and get model information
- ✅ **Error Handling** - Comprehensive error types with context
- ✅ **Automatic Retries** - Configurable retry logic with exponential backoff
- ✅ **Rate Limiting** - Built-in rate limit handling
- ✅ **MCP Integration** - Full Model Context Protocol support with adapters, registry, and bridge
- ✅ **Performance** - Optional SIMD JSON parsing (sonic-rs) for high-performance scenarios

### Provider Support
- ✅ **Anthropic API** (api.anthropic.com) - Production ready
- ✅ **AWS Bedrock** - Fully implemented (see examples: `bedrock_basic.rs`, `bedrock_streaming.rs`)
- ✅ **Google Vertex AI** - Fully implemented (see examples: `vertex_basic.rs`, `vertex_streaming.rs`)

