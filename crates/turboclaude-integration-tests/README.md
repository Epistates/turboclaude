# TurboClaude Integration Tests

End-to-end integration tests validating real-world scenarios across the TurboClaude ecosystem.

## Features

- **Real-World Scenarios**: Practical usage patterns
- **Multi-Crate Testing**: Integration across all components
- **Provider Testing**: Validates all cloud providers
- **Feature Validation**: Tests all major features
- **Performance Checks**: Basic performance validation

## Test Scenarios

- **Basic Messages**: Simple message sending
- **Streaming**: Real-time response streaming
- **Tool Usage**: Function calling workflows
- **Multi-Provider**: Same tests across all providers
- **Agent Features**: Hooks, permissions, routing
- **Skills Execution**: Dynamic skill running

## Running Tests

```bash
cargo test --test '*'
```

## Test Structure

```
tests/
├── scenarios/    (Test scenarios)
├── fixtures/     (Test data and mocks)
├── utils/        (Test utilities)
└── real_world/   (Real-world integration tests)
```

## Documentation

See `tests/real_world/README.md` for detailed integration test documentation.

---

**Part of TurboClaude** 🧪
