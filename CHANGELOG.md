# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-01-07

### Added

- **Beta Tool Types** (`types::beta::tools`)
  - `ToolSearchBm25` - BM25-based search tool with configurable parameters
  - `ToolSearchRegex` - Regex-based search tool with match limit and context lines
  - `ComputerUse20251124` - Updated computer use tool with zoom capability
  - `WebFetch` - Web fetching tool for retrieving URL content
  - `McpToolset` - MCP toolset integration with server configuration
  - `Memory` - Memory management tool for persistent context
  - `AllowedCaller` enum - Direct vs CodeExecution caller contexts
  - `McpToolConfig` and `McpToolDefaultConfig` for MCP tool configuration

- **Content Block Types** (`types::content`)
  - `ServerToolUse` - Server-side tool use blocks
  - `ToolReference` - Tool reference blocks with ref_id support
  - `ToolSearchToolResult` - Search tool result blocks with error handling
  - `McpToolUse` - MCP tool invocation blocks
  - `McpToolResult` - MCP tool result blocks with multi-content support
  - `ToolCaller` enum for tracking tool invocation source
  - `ToolSearchErrorCode` enum for categorizing search errors

- **MCP Server Configuration** (`types::beta::mcp_config`)
  - `McpServerToolConfiguration` - Per-server tool allowlisting
  - `McpServerUrlDefinition` - MCP server via URL endpoint
  - `McpToolUseBlock`, `McpToolResultBlock`, `McpToolResultContent` types

- **Output Configuration** (`types::beta::output_config`)
  - `OutputEffort` enum (Low, Medium, High) for response effort levels
  - `OutputConfig` struct with builder methods for response configuration

- **Compaction Control** (`types::beta::compaction`)
  - `CompactionControl` for automatic conversation summarization
  - `CompactionSummary` for tracking compaction results
  - `CompactionResult` enum for operation status
  - Configurable threshold, target tokens, and preserved message count

- **Agent SDK Parity** (`turboclaudeagent`)
  - `ToolsOption` enum - Preset vs explicit tool lists
  - `ToolsPreset` enum - AllTools, AllDefaultTools, None presets
  - `SessionConfig` fields: `tools`, `output_config`, `compaction`
  - Builder methods: `with_tools()`, `with_output_config()`, `with_compaction()`

- **Type Alias for Retry Operations** (`turboclaudeagent::retry`)
  - `RetryOperation<'a, T>` type alias for cleaner async operation signatures

### Changed

- `ToolChoice` now derives `Default` with `Auto` as the default variant
- `CacheTTL` now derives `Default` with `FiveMinutes` as the default variant
- `CompactionControl` now derives `Default` (disabled by default)
- Improved code quality across workspace (all clippy lints resolved)

### Fixed

- Fixed `map_or(false, ...)` patterns replaced with `is_some_and(...)`
- Fixed manual string prefix stripping to use `strip_prefix()`
- Fixed collapsible `if` statements using let-chains
- Fixed `len() > 0` comparisons to use `is_empty()`
- Fixed redundant pattern matching (`while let Some(_)` → `.is_some()`)
- Fixed field assignment after `Default::default()` to struct initialization
- Fixed `assert_eq!(x, true)` to `assert!(x)`
- Fixed useless `vec![]` to array literals where appropriate
- Fixed unused imports and dead code warnings
- Fixed benchmark types to use correct `MessageParam`/`ContentBlockParam`
- Added `required-features` for schema-dependent examples
- Added `#[allow(deprecated)]` for backward compatibility tests

## [0.2.0] - 2025-12-04

### Added

- **Full Python SDK Feature Parity**
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

## [0.1.0] - 2025-10-XX

### Added

- Core SDK implementation
- Basic message creation API
- Token counting
- Models API
