//! Built-in tool support
//!
//! This module provides support for Anthropic's built-in tools like the memory tool.
//! These tools have special handling by the API and don't require explicit schema definition.

use super::traits::{Tool, ToolExecutionResult};
use async_trait::async_trait;
use serde_json::Value;

/// Trait for built-in tools provided by Anthropic
///
/// Built-in tools are special tools provided by the Anthropic API that have
/// predefined schemas and behaviors. Examples include the memory tool.
///
/// Unlike regular tools that require explicit schemas, built-in tools use
/// special type identifiers recognized by the API.
#[async_trait]
pub trait BuiltinTool: Tool {
    /// Get the built-in tool type identifier
    ///
    /// This is the special type string recognized by the API (e.g., "memory_20250818").
    fn tool_type(&self) -> &str;

    /// Convert to a tool parameter for API requests
    ///
    /// Built-in tools may have different parameter structures than regular tools,
    /// including special fields like cache_control.
    fn to_param(&self) -> Value {
        serde_json::json!({
            "type": self.tool_type(),
            "name": self.name(),
        })
    }
}

/// Base trait for memory tool implementations
///
/// The memory tool allows Claude to persist information across conversations.
/// This trait provides the interface for implementing custom memory backends.
///
/// # Example
///
/// ```rust,ignore
/// use anthropic::tools::MemoryTool;
///
/// struct MyMemoryTool {
///     // Your storage implementation
/// }
///
/// #[async_trait]
/// impl MemoryTool for MyMemoryTool {
///     async fn view(&self, path: Option<String>) -> ToolExecutionResult {
///         // Implement viewing memory
///         Ok("Memory contents".into())
///     }
///
///     async fn create(&self, path: String, content: String) -> ToolExecutionResult {
///         // Implement creating memory
///         Ok("Created successfully".into())
///     }
///
///     // ... implement other methods
/// }
/// ```
#[async_trait]
pub trait MemoryTool: BuiltinTool {
    /// View memory contents at a path
    ///
    /// # Arguments
    ///
    /// * `path` - Optional path to view. If None, shows root contents.
    async fn view(&self, path: Option<String>) -> ToolExecutionResult;

    /// Create a new memory file with content
    ///
    /// # Arguments
    ///
    /// * `path` - Path for the new memory file
    /// * `content` - Content to write
    async fn create(&self, path: String, content: String) -> ToolExecutionResult;

    /// Replace text in a memory file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the memory file
    /// * `old_str` - Text to find
    /// * `new_str` - Text to replace with
    async fn str_replace(
        &self,
        path: String,
        old_str: String,
        new_str: String,
    ) -> ToolExecutionResult;

    /// Insert text at a specific line
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the memory file
    /// * `line` - Line number to insert at
    /// * `content` - Content to insert
    async fn insert(&self, path: String, line: usize, content: String) -> ToolExecutionResult;

    /// Delete a memory file or directory
    ///
    /// # Arguments
    ///
    /// * `path` - Path to delete
    async fn delete(&self, path: String) -> ToolExecutionResult;

    /// Rename or move a memory file or directory
    ///
    /// # Arguments
    ///
    /// * `old_path` - Current path
    /// * `new_path` - New path
    async fn rename(&self, old_path: String, new_path: String) -> ToolExecutionResult;

    /// Clear all memory data
    ///
    /// This is a destructive operation that removes all stored memory.
    async fn clear_all(&self) -> ToolExecutionResult {
        Err("clear_all not implemented".into())
    }
}

/// Abstract base implementation for memory tools
///
/// This struct provides a base implementation of the memory tool that dispatches
/// commands to the appropriate trait methods.
///
/// # Example
///
/// ```rust,ignore
/// use anthropic::tools::{AbstractMemoryTool, MemoryTool};
/// use serde_json::Value;
///
/// struct MyMemory;
///
/// #[async_trait]
/// impl MemoryTool for MyMemory {
///     async fn view(&self, path: Option<String>) -> ToolExecutionResult {
///         Ok(format!("Viewing: {:?}", path).into())
///     }
///     // ... implement other methods
/// }
///
/// let memory = AbstractMemoryTool::new(MyMemory);
/// ```
pub struct AbstractMemoryTool<T: MemoryTool> {
    inner: T,
    cache_control: Option<Value>,
}

impl<T: MemoryTool> AbstractMemoryTool<T> {
    /// Create a new abstract memory tool wrapping a concrete implementation
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            cache_control: None,
        }
    }

    /// Set cache control configuration
    ///
    /// This allows enabling prompt caching for the memory tool.
    pub fn with_cache_control(mut self, cache_control: Value) -> Self {
        self.cache_control = Some(cache_control);
        self
    }

    /// Execute a memory command by dispatching to the appropriate method
    async fn execute_command(&self, command: MemoryCommand) -> ToolExecutionResult {
        match command {
            MemoryCommand::View { path } => self.inner.view(path).await,
            MemoryCommand::Create { path, content } => self.inner.create(path, content).await,
            MemoryCommand::StrReplace {
                path,
                old_str,
                new_str,
            } => self.inner.str_replace(path, old_str, new_str).await,
            MemoryCommand::Insert {
                path,
                line,
                content,
            } => self.inner.insert(path, line, content).await,
            MemoryCommand::Delete { path } => self.inner.delete(path).await,
            MemoryCommand::Rename { old_path, new_path } => {
                self.inner.rename(old_path, new_path).await
            }
        }
    }
}

#[async_trait]
impl<T: MemoryTool + Send + Sync> Tool for AbstractMemoryTool<T> {
    fn name(&self) -> &str {
        "memory"
    }

    fn description(&self) -> &str {
        "Claude's memory tool for persisting information"
    }

    fn input_schema(&self) -> Value {
        // Memory tools don't use explicit schemas - the API handles them specially
        Value::Null
    }

    async fn call(&self, input: Value) -> ToolExecutionResult {
        // Parse the input as a memory command
        let command: MemoryCommand = serde_json::from_value(input).map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Failed to parse memory command: {}", e),
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;

        self.execute_command(command).await
    }
}

#[async_trait]
impl<T: MemoryTool + Send + Sync> BuiltinTool for AbstractMemoryTool<T> {
    fn tool_type(&self) -> &str {
        "memory_20250818"
    }

    fn to_param(&self) -> Value {
        let mut param = serde_json::json!({
            "type": self.tool_type(),
            "name": "memory",
        });

        if let Some(cache_control) = &self.cache_control {
            if let Some(obj) = param.as_object_mut() {
                obj.insert("cache_control".to_string(), cache_control.clone());
            }
        }

        param
    }
}

/// Memory command types
#[derive(Debug, serde::Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
enum MemoryCommand {
    View {
        #[serde(default)]
        path: Option<String>,
    },
    Create {
        path: String,
        content: String,
    },
    StrReplace {
        path: String,
        old_str: String,
        new_str: String,
    },
    Insert {
        path: String,
        line: usize,
        content: String,
    },
    Delete {
        path: String,
    },
    Rename {
        old_path: String,
        new_path: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockMemory;

    #[async_trait]
    impl MemoryTool for MockMemory {
        async fn view(&self, _path: Option<String>) -> ToolExecutionResult {
            Ok("mock view".into())
        }

        async fn create(&self, _path: String, _content: String) -> ToolExecutionResult {
            Ok("mock create".into())
        }

        async fn str_replace(
            &self,
            _path: String,
            _old_str: String,
            _new_str: String,
        ) -> ToolExecutionResult {
            Ok("mock str_replace".into())
        }

        async fn insert(
            &self,
            _path: String,
            _line: usize,
            _content: String,
        ) -> ToolExecutionResult {
            Ok("mock insert".into())
        }

        async fn delete(&self, _path: String) -> ToolExecutionResult {
            Ok("mock delete".into())
        }

        async fn rename(&self, _old_path: String, _new_path: String) -> ToolExecutionResult {
            Ok("mock rename".into())
        }
    }

    #[async_trait]
    impl BuiltinTool for MockMemory {
        fn tool_type(&self) -> &str {
            "memory_20250818"
        }
    }

    #[async_trait]
    impl Tool for MockMemory {
        fn name(&self) -> &str {
            "memory"
        }

        fn description(&self) -> &str {
            "Mock memory tool"
        }

        fn input_schema(&self) -> Value {
            Value::Null
        }

        async fn call(&self, _input: Value) -> ToolExecutionResult {
            Ok("mock".into())
        }
    }

    #[tokio::test]
    async fn test_abstract_memory_tool_creation() {
        let memory = AbstractMemoryTool::new(MockMemory);
        assert_eq!(memory.name(), "memory");
        assert_eq!(memory.tool_type(), "memory_20250818");
    }

    #[tokio::test]
    async fn test_memory_tool_to_param() {
        let memory = AbstractMemoryTool::new(MockMemory);
        let param = memory.to_param();

        assert_eq!(param["type"], "memory_20250818");
        assert_eq!(param["name"], "memory");
    }

    #[tokio::test]
    async fn test_memory_tool_with_cache_control() {
        let cache_control = serde_json::json!({"type": "ephemeral"});
        let memory = AbstractMemoryTool::new(MockMemory).with_cache_control(cache_control.clone());

        let param = memory.to_param();
        assert_eq!(param["cache_control"], cache_control);
    }

    #[tokio::test]
    async fn test_memory_command_view() {
        let memory = AbstractMemoryTool::new(MockMemory);
        let input = serde_json::json!({"command": "view", "path": "/test"});

        let result = memory.call(input).await.unwrap();
        assert_eq!(result.as_string(), "mock view");
    }

    #[tokio::test]
    async fn test_memory_command_create() {
        let memory = AbstractMemoryTool::new(MockMemory);
        let input = serde_json::json!({
            "command": "create",
            "path": "/test.txt",
            "content": "Hello, World!"
        });

        let result = memory.call(input).await.unwrap();
        assert_eq!(result.as_string(), "mock create");
    }
}
