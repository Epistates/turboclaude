# TurboClaude MCP - Model Context Protocol Integration

Seamless integration with Model Context Protocol (MCP) servers, supporting both official Anthropic SDK and TurboMCP implementations.

## Features

- **Unified Interface**: Single API for all MCP implementations
- **Dual SDK Support**: Works with official Anthropic SDK and TurboMCP
- **Adapter Pattern**: Clean abstraction over different MCP implementations
- **Factory Pattern**: Easy MCP client creation
- **Registry Pattern**: Manage multiple MCP servers

## Quick Start

```rust
use turboclaude_mcp::McpClientFactory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create MCP client
    let client = McpClientFactory::create("official")
        .await?;

    // Use MCP client
    client.initialize().await?;

    Ok(())
}
```

## Components

### McpClientFactory
Create MCP clients for different implementations.

### McpAdapters
Adapt between official and TurboMCP SDKs:
- `OfficialSdkAdapter`: Official Anthropic SDK
- `TurboMcpAdapter`: TurboMCP implementation

### McpRegistry
Manage multiple MCP server connections.

### McpBridge
Core bridging logic between implementations.

## Supported Implementations

- **Official Anthropic SDK**: Full support
- **TurboMCP**: Complete bridge implementation

## Architecture

```
turboclaude-mcp
├── adapters     (SDK adapters)
├── factory      (Client factory)
├── registry     (Server registry)
├── bridge       (Bridging logic)
└── error        (Error types)
```

## Testing

```bash
cargo test
```

## Documentation

Full API docs: `cargo doc --open`

---

**Part of TurboClaude** 🔌
