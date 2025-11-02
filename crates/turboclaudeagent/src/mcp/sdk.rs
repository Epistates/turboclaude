//! SDK MCP Server implementation for in-process tool execution.
//!
//! This module provides a builder API for creating MCP servers that run within
//! the same process as your application, eliminating subprocess overhead.
//!
//! # Example
//!
//! ```rust
//! use turboclaudeagent::mcp::sdk::*;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Deserialize)]
//! struct CalcInput { a: i32, b: i32 }
//!
//! #[derive(Serialize)]
//! struct CalcOutput { result: i32 }
//!
//! # async fn example() -> Result<(), SdkToolError> {
//! let server = SdkMcpServerBuilder::new("calculator")
//!     .tool("add", "Add two numbers", |input: CalcInput| async move {
//!         Ok(CalcOutput { result: input.a + input.b })
//!     })
//!     .build();
//!
//! // Execute a tool
//! let result = server.execute_tool(
//!     "add",
//!     serde_json::json!({"a": 5, "b": 3})
//! ).await?;
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;
use thiserror::Error;

/// Errors that can occur during SDK tool execution.
#[derive(Debug, Error)]
pub enum SdkToolError {
    /// Input JSON doesn't match expected schema
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Tool execution failed
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// An in-process MCP tool that can be executed synchronously.
///
/// Implement this trait to create custom tools, or use the builder API
/// with closures for simpler use cases.
#[async_trait]
pub trait SdkTool: Send + Sync {
    /// Unique identifier for this tool.
    fn name(&self) -> &str;

    /// Human-readable description of what this tool does.
    fn description(&self) -> &str;

    /// JSON Schema describing the expected input format.
    ///
    /// For MVP, this can return a basic schema. Future versions may
    /// support automatic schema generation from Rust types.
    fn input_schema(&self) -> Value;

    /// Execute the tool with the given input.
    ///
    /// # Arguments
    ///
    /// * `input` - JSON value matching the `input_schema()`
    ///
    /// # Returns
    ///
    /// JSON value representing the tool's output, or an error if execution failed.
    async fn execute(&self, input: Value) -> Result<Value, SdkToolError>;
}

/// Type-safe wrapper for function-based tools.
///
/// This struct implements `SdkTool` for closures that accept a typed input
/// and return a typed output. It handles JSON serialization/deserialization
/// automatically.
pub struct FunctionTool<F, Fut, I, O> {
    name: String,
    description: String,
    handler: F,
    _phantom: PhantomData<(Fut, I, O)>,
}

impl<F, Fut, I, O> FunctionTool<F, Fut, I, O> {
    /// Create a new function-based tool.
    ///
    /// # Arguments
    ///
    /// * `name` - Unique identifier for the tool
    /// * `description` - Human-readable description
    /// * `handler` - Async function that executes the tool logic
    pub fn new(name: String, description: String, handler: F) -> Self {
        Self {
            name,
            description,
            handler,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<F, Fut, I, O> SdkTool for FunctionTool<F, Fut, I, O>
where
    F: Fn(I) -> Fut + Send + Sync,
    Fut: Future<Output = Result<O, SdkToolError>> + Send + Sync,
    I: DeserializeOwned + Send + Sync,
    O: Serialize + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn input_schema(&self) -> Value {
        // MVP: Return a permissive schema that accepts any object.
        // Future enhancement: Use `schemars` crate for type-based schema generation.
        serde_json::json!({
            "type": "object",
            "properties": {},
            "additionalProperties": true
        })
    }

    async fn execute(&self, input: Value) -> Result<Value, SdkToolError> {
        // Deserialize input to typed struct
        let typed_input: I = serde_json::from_value(input).map_err(|e| {
            SdkToolError::InvalidInput(format!("Failed to deserialize input: {}", e))
        })?;

        // Call the handler function
        let output = (self.handler)(typed_input).await?;

        // Serialize output to JSON
        let json_output = serde_json::to_value(output)?;

        Ok(json_output)
    }
}

/// Builder for creating SDK MCP servers with a fluent API.
///
/// # Example
///
/// ```rust
/// use turboclaudeagent::mcp::sdk::*;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Deserialize)]
/// struct Input { value: String }
///
/// #[derive(Serialize)]
/// struct Output { processed: String }
///
/// let server = SdkMcpServerBuilder::new("my-tools")
///     .tool("process", "Process a value", |input: Input| async move {
///         Ok(Output { processed: input.value.to_uppercase() })
///     })
///     .build();
/// ```
pub struct SdkMcpServerBuilder {
    name: String,
    tools: HashMap<String, Arc<dyn SdkTool>>,
}

impl SdkMcpServerBuilder {
    /// Create a new builder with the given server name.
    ///
    /// # Arguments
    ///
    /// * `name` - Unique identifier for this MCP server
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tools: HashMap::new(),
        }
    }

    /// Add a function-based tool with type-safe input/output.
    ///
    /// # Type Parameters
    ///
    /// * `F` - The closure type (inferred)
    /// * `Fut` - The future returned by the closure (inferred)
    /// * `I` - Input type that implements `DeserializeOwned`
    /// * `O` - Output type that implements `Serialize`
    ///
    /// # Arguments
    ///
    /// * `name` - Unique identifier for the tool
    /// * `description` - Human-readable description
    /// * `handler` - Async closure that implements the tool logic
    ///
    /// # Example
    ///
    /// ```rust
    /// # use turboclaudeagent::mcp::sdk::*;
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Deserialize)]
    /// # struct Input { x: i32 }
    /// # #[derive(Serialize)]
    /// # struct Output { result: i32 }
    /// let builder = SdkMcpServerBuilder::new("math")
    ///     .tool("double", "Double a number", |input: Input| async move {
    ///         Ok(Output { result: input.x * 2 })
    ///     });
    /// ```
    pub fn tool<F, Fut, I, O>(mut self, name: &str, description: &str, handler: F) -> Self
    where
        F: Fn(I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<O, SdkToolError>> + Send + Sync + 'static,
        I: DeserializeOwned + Send + Sync + 'static,
        O: Serialize + Send + Sync + 'static,
    {
        let tool = FunctionTool::new(name.to_string(), description.to_string(), handler);
        self.tools.insert(name.to_string(), Arc::new(tool));
        self
    }

    /// Add a custom tool implementation.
    ///
    /// Use this method if you've implemented the `SdkTool` trait yourself
    /// and want more control than the closure-based API provides.
    ///
    /// # Arguments
    ///
    /// * `tool` - An implementation of the `SdkTool` trait
    ///
    /// # Example
    ///
    /// ```rust
    /// # use turboclaudeagent::mcp::sdk::*;
    /// # use async_trait::async_trait;
    /// # use serde_json::Value;
    /// # use std::sync::Arc;
    /// struct MyTool;
    ///
    /// #[async_trait]
    /// impl SdkTool for MyTool {
    ///     fn name(&self) -> &str { "my_tool" }
    ///     fn description(&self) -> &str { "Custom tool" }
    ///     fn input_schema(&self) -> Value { serde_json::json!({}) }
    ///     async fn execute(&self, _input: Value) -> Result<Value, SdkToolError> {
    ///         Ok(serde_json::json!({"status": "ok"}))
    ///     }
    /// }
    ///
    /// let builder = SdkMcpServerBuilder::new("custom")
    ///     .add_tool(Arc::new(MyTool));
    /// ```
    pub fn add_tool(mut self, tool: Arc<dyn SdkTool>) -> Self {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
        self
    }

    /// Build the SDK MCP server.
    ///
    /// Consumes the builder and returns a ready-to-use `SdkMcpServer`.
    pub fn build(self) -> SdkMcpServer {
        SdkMcpServer {
            name: self.name,
            tools: self.tools,
        }
    }
}

/// An in-process MCP server that executes tools without subprocess overhead.
///
/// This server runs within the same process as your application, providing:
/// - Zero IPC overhead
/// - Direct access to application state via closures
/// - Simpler deployment (single process)
/// - Easier debugging
///
/// # Thread Safety
///
/// `SdkMcpServer` is immutable after construction and can be safely shared
/// across threads using `Arc` or cloned directly (implements `Clone`).
#[derive(Clone)]
pub struct SdkMcpServer {
    name: String,
    tools: HashMap<String, Arc<dyn SdkTool>>,
}

impl std::fmt::Debug for SdkMcpServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SdkMcpServer")
            .field("name", &self.name)
            .field("tool_count", &self.tools.len())
            .finish()
    }
}

impl SdkMcpServer {
    /// Get the server's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get a tool by name.
    ///
    /// Returns `None` if no tool with the given name exists.
    pub fn get_tool(&self, name: &str) -> Option<&Arc<dyn SdkTool>> {
        self.tools.get(name)
    }

    /// List all available tools.
    ///
    /// Returns a vector of references to the tools registered with this server.
    pub fn list_tools(&self) -> Vec<&Arc<dyn SdkTool>> {
        self.tools.values().collect()
    }

    /// Execute a tool by name with the given input.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tool to execute
    /// * `input` - JSON value matching the tool's input schema
    ///
    /// # Returns
    ///
    /// The tool's output as a JSON value, or an error if:
    /// - The tool doesn't exist
    /// - The input doesn't match the schema
    /// - The tool execution fails
    ///
    /// # Example
    ///
    /// ```rust
    /// # use turboclaudeagent::mcp::sdk::*;
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Deserialize)]
    /// # struct Input { value: i32 }
    /// # #[derive(Serialize)]
    /// # struct Output { result: i32 }
    /// # async fn example() -> Result<(), SdkToolError> {
    /// # let server = SdkMcpServerBuilder::new("test")
    /// #     .tool("double", "Double a number", |input: Input| async move {
    /// #         Ok(Output { result: input.value * 2 })
    /// #     })
    /// #     .build();
    /// let result = server.execute_tool(
    ///     "double",
    ///     serde_json::json!({"value": 21})
    /// ).await?;
    /// assert_eq!(result, serde_json::json!({"result": 42}));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_tool(&self, name: &str, input: Value) -> Result<Value, SdkToolError> {
        match self.get_tool(name) {
            Some(tool) => tool.execute(input).await,
            None => Err(SdkToolError::InvalidInput(format!(
                "Tool '{}' not found in server '{}'",
                name, self.name
            ))),
        }
    }

    /// Check if a tool exists in this server.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tool to check
    ///
    /// # Returns
    ///
    /// `true` if a tool with this name exists, `false` otherwise.
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get the number of tools in this server.
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    struct TestInput {
        value: i32,
    }

    #[derive(Serialize, PartialEq, Debug)]
    struct TestOutput {
        result: i32,
    }

    #[tokio::test]
    async fn test_function_tool_execution() {
        let tool = FunctionTool::new(
            "double".to_string(),
            "Double a number".to_string(),
            |input: TestInput| async move {
                Ok(TestOutput {
                    result: input.value * 2,
                })
            },
        );

        let input = serde_json::json!({"value": 21});
        let output = tool.execute(input).await.expect("execution failed");

        assert_eq!(output, serde_json::json!({"result": 42}));
    }

    #[tokio::test]
    async fn test_sdk_server_builder() {
        let server = SdkMcpServerBuilder::new("test-server")
            .tool("add", "Add two numbers", |input: TestInput| async move {
                Ok(TestOutput {
                    result: input.value + 10,
                })
            })
            .tool(
                "multiply",
                "Multiply by two",
                |input: TestInput| async move {
                    Ok(TestOutput {
                        result: input.value * 2,
                    })
                },
            )
            .build();

        assert_eq!(server.name(), "test-server");
        assert_eq!(server.tool_count(), 2);
        assert!(server.has_tool("add"));
        assert!(server.has_tool("multiply"));
        assert!(!server.has_tool("nonexistent"));
    }

    #[tokio::test]
    async fn test_server_execute_tool() {
        let server = SdkMcpServerBuilder::new("calculator")
            .tool("double", "Double a number", |input: TestInput| async move {
                Ok(TestOutput {
                    result: input.value * 2,
                })
            })
            .build();

        let result = server
            .execute_tool("double", serde_json::json!({"value": 5}))
            .await
            .expect("execution failed");

        assert_eq!(result, serde_json::json!({"result": 10}));
    }

    #[tokio::test]
    async fn test_tool_not_found() {
        let server = SdkMcpServerBuilder::new("empty").build();

        let result = server.execute_tool("missing", serde_json::json!({})).await;

        assert!(result.is_err());
        match result {
            Err(SdkToolError::InvalidInput(msg)) => {
                assert!(msg.contains("not found"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[tokio::test]
    async fn test_invalid_input_deserialization() {
        let server = SdkMcpServerBuilder::new("test")
            .tool("strict", "Strict input", |input: TestInput| async move {
                Ok(TestOutput {
                    result: input.value,
                })
            })
            .build();

        // Missing required field
        let result = server.execute_tool("strict", serde_json::json!({})).await;

        assert!(result.is_err());
        match result {
            Err(SdkToolError::InvalidInput(_)) => {}
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[tokio::test]
    async fn test_tool_execution_error() {
        let server = SdkMcpServerBuilder::new("test")
            .tool("failing", "Always fails", |_input: TestInput| async move {
                Err::<TestOutput, _>(SdkToolError::ExecutionFailed(
                    "intentional failure".to_string(),
                ))
            })
            .build();

        let result = server
            .execute_tool("failing", serde_json::json!({"value": 1}))
            .await;

        assert!(result.is_err());
        match result {
            Err(SdkToolError::ExecutionFailed(msg)) => {
                assert_eq!(msg, "intentional failure");
            }
            _ => panic!("Expected ExecutionFailed error"),
        }
    }

    #[tokio::test]
    async fn test_server_clone() {
        let server = SdkMcpServerBuilder::new("original")
            .tool("tool1", "First tool", |input: TestInput| async move {
                Ok(TestOutput {
                    result: input.value,
                })
            })
            .build();

        let cloned = server.clone();

        assert_eq!(server.name(), cloned.name());
        assert_eq!(server.tool_count(), cloned.tool_count());
        assert!(cloned.has_tool("tool1"));
    }

    #[tokio::test]
    async fn test_list_tools() {
        let server = SdkMcpServerBuilder::new("test")
            .tool("tool1", "First", |input: TestInput| async move {
                Ok(TestOutput {
                    result: input.value,
                })
            })
            .tool("tool2", "Second", |input: TestInput| async move {
                Ok(TestOutput {
                    result: input.value,
                })
            })
            .build();

        let tools = server.list_tools();
        assert_eq!(tools.len(), 2);

        let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
        assert!(names.contains(&"tool1"));
        assert!(names.contains(&"tool2"));
    }
}
