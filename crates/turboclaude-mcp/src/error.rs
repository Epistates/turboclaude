//! Error types for SDK-agnostic MCP operations

use thiserror::Error;

/// Result type for MCP operations
pub type McpResult<T> = Result<T, McpError>;

/// Error types for MCP operations
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum McpError {
    /// MCP protocol error
    #[error("MCP protocol error: {0}")]
    ProtocolError(String),

    /// Client initialization failed
    #[error("Failed to initialize MCP client: {0}")]
    InitializationError(String),

    /// Tool not found
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    /// Resource not found
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    /// Prompt not found
    #[error("Prompt not found: {0}")]
    PromptNotFound(String),

    /// Invalid arguments for tool/prompt
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),

    /// Tool execution error
    #[error("Tool execution error: {0}")]
    ToolExecutionError(String),

    /// Resource read error
    #[error("Failed to read resource: {0}")]
    ResourceReadError(String),

    /// Prompt execution error
    #[error("Failed to execute prompt: {0}")]
    PromptExecutionError(String),

    /// Transport error
    #[error("Transport error: {0}")]
    TransportError(String),

    /// Request timeout
    #[error("Request timeout")]
    Timeout,

    /// Client closed or disconnected
    #[error("MCP client closed or disconnected")]
    ClientClosed,

    /// Request cancelled
    #[error("Request cancelled")]
    Cancelled,

    /// SDK-specific error (for adapter-specific errors)
    #[error("SDK error: {0}")]
    SdkError(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Adapter not found
    #[error("Adapter not found for SDK: {0}")]
    AdapterNotFound(String),

    /// Invalid adapter configuration
    #[error("Invalid adapter configuration: {0}")]
    InvalidAdapterConfig(String),

    /// Feature not supported by this SDK
    #[error("Feature not supported: {0}")]
    FeatureNotSupported(String),

    /// Invalid input provided
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl McpError {
    /// Create a protocol error
    pub fn protocol(msg: impl Into<String>) -> Self {
        Self::ProtocolError(msg.into())
    }

    /// Create an initialization error
    pub fn init(msg: impl Into<String>) -> Self {
        Self::InitializationError(msg.into())
    }

    /// Create a tool execution error
    pub fn tool_error(msg: impl Into<String>) -> Self {
        Self::ToolExecutionError(msg.into())
    }

    /// Create a serialization error
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::SerializationError(msg.into())
    }
}
