# TurboClaude Transport - HTTP & Subprocess Transport Layer

Low-level transport abstraction for TurboClaude, supporting both HTTP and subprocess-based communication with retry, timeout, and rate-limiting capabilities.

## Features

- **HTTP Transport**: RESTful API communication with retries
- **Subprocess Transport**: Local process communication
- **Retry Logic**: Configurable exponential backoff
- **Timeouts**: Per-request timeout handling
- **Rate Limiting**: Built-in rate limit management
- **Connection Pooling**: Efficient connection reuse

## Quick Start

```rust
use turboclaude_transport::http::HttpTransport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let transport = HttpTransport::builder()
        .base_url("https://api.anthropic.com")
        .timeout(30)
        .build()?;

    let response = transport.request("GET", "/models", None).await?;
    println!("{:?}", response);

    Ok(())
}
```

## Components

### HTTP Transport
Standard HTTP/HTTPS communication with automatic retry and timeout handling.

### Subprocess Transport
Local subprocess communication for sandboxed execution.

### Retry Strategy
Exponential backoff with jitter for reliable API communication.

### Error Handling
Comprehensive error types for transport-level failures.

## Architecture

```
turboclaude-transport
├── http      (HTTP client implementation)
├── subprocess (Process communication)
├── retry     (Retry strategy)
└── error     (Error types)
```

## Testing

```bash
cargo test
```

68 tests covering all transport scenarios.

## Documentation

Full API docs: `cargo doc --open`

---

**Part of TurboClaude** 🚚
