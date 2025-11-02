//! Tool system for Anthropic SDK
//!
//! Provides tool system with automatic tool execution loops and schema generation.
//!
//! # Features
//!
//! - **Automatic Schema Generation**: Use `#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]`
//!   on your input types to automatically generate JSON schemas
//! - **Tool Trait**: Implement the `Tool` trait to create custom tools
//! - **Tool Runner**: Automatic tool execution loop with error handling
//! - **Function Tools**: Easy tool creation from functions
//!
//! # Example
//!
//! ```rust,ignore
//! use turboclaude::tools::{Tool, ToolRunner, FunctionTool};
//! use schemars::JsonSchema;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Deserialize, JsonSchema)]
//! struct WeatherInput {
//!     /// The location to get weather for
//!     location: String,
//!     /// Temperature units (celsius or fahrenheit)
//!     #[serde(default = "default_units")]
//!     units: String,
//! }
//!
//! fn default_units() -> String {
//!     "celsius".to_string()
//! }
//!
//! async fn get_weather(input: WeatherInput) -> Result<String, Box<dyn std::error::Error>> {
//!     Ok(format!("The weather in {} is 72°{}", input.location, if input.units == "celsius" { "C" } else { "F" }))
//! }
//!
//! // Create a tool from the function
//! let weather_tool = FunctionTool::new(
//!     "get_weather",
//!     "Get the current weather for a location",
//!     get_weather,
//! );
//!
//! // Run tool loop automatically
//! let runner = ToolRunner::new(client)
//!     .add_tool(weather_tool)
//!     .with_max_iterations(5);
//!
//! let final_message = runner.run(request).await?;
//! ```

pub mod builtin;
mod function;
mod runner;
mod traits;

pub use builtin::{AbstractMemoryTool, BuiltinTool, MemoryTool};
pub use function::FunctionTool;
pub use runner::{ToolRunner, ToolRunnerError};
pub use traits::{Tool, ToolExecutionResult, ToolResult};

// Re-export commonly used types
#[cfg(feature = "schema")]
pub use schemars::JsonSchema;
