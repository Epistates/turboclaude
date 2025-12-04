# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-12-04

### Added

- **Microsoft Azure Foundry Provider** (`providers::foundry`)
  - Complete Azure Foundry integration for accessing Claude models through Microsoft Azure
  - Supports API key authentication via `ANTHROPIC_FOUNDRY_API_KEY`
  - Supports Azure AD token authentication for enterprise deployments
  - Full `HttpProvider` trait implementation with streaming support
  - Automatic request/response translation for Foundry API format

- **HookMatcher Timeout Support** (`turboclaude-protocol`)
  - Added `timeout: Option<f64>` field to `HookMatcher` for hook execution timeouts
  - New helper methods: `with_timeout()`, `with_timeout_duration()`, `timeout_or_default()`
  - Default timeout of 60 seconds when not specified

- **AssistantMessageError Type** (`turboclaude-protocol`)
  - New error enum for categorizing assistant message generation failures
  - Variants: `AuthenticationFailed`, `BillingError`, `RateLimit`, `InvalidRequest`, `ServerError`, `Unknown`
  - Added `error` and `parent_tool_use_id` fields to `AssistantMessage`
  - Helper methods: `has_error()`, `is_retryable()`, `with_error()`

- **Clear Tool Uses Context Management** (`types::beta::context_management`)
  - New `BetaClearToolUses20250919EditParam` for clearing tool use blocks from context
  - Added `ClearToolUses` variant to `ContextManagementEdit` enum
  - Added `ClearToolUses` variant to `ContextManagementEditResponse` enum
  - New helper methods: `is_clear_tool_uses()`, `cleared_input_tokens()`

- **SandboxSettings for Agent Isolation** (`turboclaudeagent::sandbox`)
  - New `SandboxSettings` struct for configuring agent sandbox behavior
  - `SandboxNetworkConfig` for network isolation settings
  - `SandboxIgnoreViolations` for selective violation ignoring
  - Builder pattern with comprehensive configuration options
  - Integration with `SessionConfig::with_sandbox()`

### Changed

- **Beta Headers Updated**
  - Updated `structured-outputs` beta header from `2025-09-17` to `2025-11-13`

- **Model Type Updates**
  - `Model.display_name` is now `Option<String>` (was `String`)
  - `Model.model_type` renamed to `Model.r#type` for Rust keyword compliance
  - `Model.created_at` is now `String` (ISO 8601 format) instead of `DateTime<Utc>`

### Fixed

- Fixed non-exhaustive pattern matches for new `ContextManagementEdit` variants
- Fixed doc tests that referenced outdated type signatures
- Disabled integration tests that accessed private implementation details

## [0.2.0] - 2025-11-XX

### Added

- Initial release with full Python SDK feature parity
- AWS Bedrock provider support
- Google Vertex AI provider support
- Structured outputs with JSON schema validation
- Extended thinking (chain-of-thought reasoning)
- Context management for conversation optimization
- Tool runner with automatic tool execution loops
- Message batching API
- Streaming responses with Server-Sent Events
- Comprehensive error handling with retry logic
- Connection pooling and rate limiting
- Proxy support (HTTP, HTTPS, SOCKS5)

## [0.1.0] - 2025-10-XX

### Added

- Core SDK implementation
- Basic message creation API
- Token counting
- Models API
