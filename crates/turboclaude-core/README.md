# TurboClaude Core - Core Abstractions and Traits

Fundamental abstractions, traits, and building blocks underlying all TurboClaude components.

## Features

- **Core Traits**: HttpProvider, Transport abstractions
- **Builder Patterns**: Type-safe configuration builders
- **Error Types**: Comprehensive error handling
- **Utilities**: Common functionality and helpers
- **Extensibility**: Foundation for custom implementations

## Quick Start

```rust
use turboclaude_core::traits::HttpProvider;

#[async_trait::async_trait]
impl HttpProvider for MyProvider {
    async fn request(&self, method: Method, path: &str, body: Option<&Body>)
        -> Result<Response> {
        // Custom implementation
    }
}
```

## Components

### Core Traits
Abstract interfaces for all TurboClaude systems.

### Builder Patterns
Type-safe, ergonomic configuration APIs.

### Error Handling
Rich error types with context information.

### Utilities
Common helpers and utilities for SDK development.

## Architecture

```
turboclaude-core
├── traits     (Core trait definitions)
├── builder    (Builder pattern implementations)
├── error      (Error types)
└── utils      (Utility functions)
```

## Testing

```bash
cargo test
```

31 tests covering core functionality.

## Documentation

Full API docs: `cargo doc --open`

---

**Part of TurboClaude** 🏗️
