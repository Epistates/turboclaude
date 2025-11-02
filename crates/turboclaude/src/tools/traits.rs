//! Core tool traits

use async_trait::async_trait;
use serde_json::Value;
use std::error::Error;
use std::fmt;

/// Result type for tool execution
pub type ToolExecutionResult = Result<ToolResult, Box<dyn Error + Send + Sync>>;

/// Result from a tool execution
#[derive(Debug, Clone)]
pub enum ToolResult {
    /// Simple text result
    Text(String),

    /// JSON result
    Json(Value),

    /// Multiple content blocks
    ContentBlocks(Vec<ToolContentBlock>),
}

impl ToolResult {
    /// Create a text result
    pub fn text(s: impl Into<String>) -> Self {
        Self::Text(s.into())
    }

    /// Create a JSON result
    pub fn json(value: Value) -> Self {
        Self::Json(value)
    }

    /// Convert to string representation
    pub fn as_string(&self) -> String {
        match self {
            Self::Text(s) => s.clone(),
            Self::Json(v) => serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string()),
            Self::ContentBlocks(blocks) => blocks
                .iter()
                .map(|b| match b {
                    ToolContentBlock::Text { text } => text.clone(),
                    ToolContentBlock::Image { source } => {
                        format!("[Image: {}]", source.media_type)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}

impl From<String> for ToolResult {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<&str> for ToolResult {
    fn from(s: &str) -> Self {
        Self::Text(s.to_string())
    }
}

impl From<Value> for ToolResult {
    fn from(v: Value) -> Self {
        Self::Json(v)
    }
}

// Support for Result types
impl<T, E> From<Result<T, E>> for ToolResult
where
    T: Into<ToolResult>,
    E: std::fmt::Display,
{
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(value) => value.into(),
            Err(e) => ToolResult::Text(format!("Error: {}", e)),
        }
    }
}

/// Content block for tool results
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ToolContentBlock {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "image")]
    Image { source: ToolImageSource },
}

/// Image source for tool results
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolImageSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub media_type: String,
    pub data: String,
}

/// Core trait for tools that can be used with Claude
///
/// This trait defines the interface for tools that can be provided to Claude.
/// Tools can be implemented manually or created using helper types like `FunctionTool`.
///
/// # Example
///
/// ```rust,ignore
/// use turboclaude::tools::{Tool, ToolExecutionResult};
/// use async_trait::async_trait;
/// use serde_json::{json, Value};
///
/// struct WeatherTool;
///
/// #[async_trait]
/// impl Tool for WeatherTool {
///     fn name(&self) -> &str {
///         "get_weather"
///     }
///
///     fn description(&self) -> &str {
///         "Get the current weather for a location"
///     }
///
///     fn input_schema(&self) -> Value {
///         json!({
///             "type": "object",
///             "properties": {
///                 "location": {
///                     "type": "string",
///                     "description": "The city and state, e.g. San Francisco, CA"
///                 },
///                 "units": {
///                     "type": "string",
///                     "enum": ["celsius", "fahrenheit"],
///                     "description": "Temperature unit"
///                 }
///             },
///             "required": ["location"]
///         })
///     }
///
///     async fn call(&self, input: Value) -> ToolExecutionResult {
///         let location = input["location"].as_str().unwrap_or("unknown");
///         Ok(format!("The weather in {} is sunny and 72°F", location).into())
///     }
/// }
/// ```
#[async_trait]
pub trait Tool: Send + Sync {
    /// The name of the tool
    ///
    /// This will be used to identify the tool in API calls.
    /// Should be lowercase with underscores (snake_case).
    fn name(&self) -> &str;

    /// A description of what the tool does
    ///
    /// This helps Claude understand when to use the tool.
    /// Should be clear and concise.
    fn description(&self) -> &str;

    /// JSON Schema for the tool's input parameters
    ///
    /// This defines what parameters the tool accepts.
    /// When using `#[derive(JsonSchema)]` from schemars, this can be
    /// automatically generated.
    fn input_schema(&self) -> Value;

    /// Execute the tool with the given input
    ///
    /// # Arguments
    ///
    /// * `input` - The input parameters as a JSON value
    ///
    /// # Returns
    ///
    /// Returns a `ToolExecutionResult` which can be a text response,
    /// JSON data, or structured content blocks.
    ///
    /// # Errors
    ///
    /// Should return an error if the input is invalid or execution fails.
    async fn call(&self, input: Value) -> ToolExecutionResult;
}

/// Error that occurred during tool execution
#[allow(dead_code)] // Will be used in future enhancements
#[derive(Debug)]
pub struct ToolError {
    pub tool_name: String,
    pub message: String,
    pub source: Option<Box<dyn Error + Send + Sync>>,
}

impl ToolError {
    /// Create a new tool error
    #[allow(dead_code)] // Part of public API, not yet used internally
    pub fn new(tool_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Create a tool error with a source error
    #[allow(dead_code)] // Part of public API, not yet used internally
    pub fn with_source(
        tool_name: impl Into<String>,
        message: impl Into<String>,
        source: Box<dyn Error + Send + Sync>,
    ) -> Self {
        Self {
            tool_name: tool_name.into(),
            message: message.into(),
            source: Some(source),
        }
    }
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tool '{}' error: {}", self.tool_name, self.message)
    }
}

impl Error for ToolError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn Error + 'static))
    }
}
