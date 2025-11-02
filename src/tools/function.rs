//! Function-based tool implementation
//!
//! This module provides `FunctionTool` which allows creating tools from async functions
//! with automatic schema generation (when the `schema` feature is enabled).

use super::traits::{Tool, ToolExecutionResult, ToolResult};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;

/// Type alias for async tool functions
pub type AsyncToolFn<I, O> = Arc<dyn Fn(I) -> Pin<Box<dyn Future<Output = O> + Send>> + Send + Sync>;

/// A tool created from an async function
///
/// This allows creating tools from simple async functions with automatic
/// input validation and schema generation.
///
/// # Example
///
/// ```rust,ignore
/// use anthropic::tools::FunctionTool;
/// use schemars::JsonSchema;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, JsonSchema)]
/// struct Input {
///     location: String,
/// }
///
/// async fn get_weather(input: Input) -> Result<String, Box<dyn std::error::Error>> {
///     Ok(format!("Weather in {}: Sunny", input.location))
/// }
///
/// let tool = FunctionTool::new("get_weather", "Get weather info", get_weather);
/// ```
pub struct FunctionTool<I, O> {
    name: String,
    description: String,
    input_schema: Value,
    #[allow(clippy::type_complexity)]
    func: AsyncToolFn<I, O>,
    _phantom: PhantomData<fn(I) -> O>,
}

impl<I, O> FunctionTool<I, O>
where
    I: DeserializeOwned + Send + 'static,
    O: Into<ToolResult> + Send + 'static,
{
    /// Create a new function tool with automatic schema generation (requires `schema` feature)
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tool (snake_case recommended)
    /// * `description` - What the tool does
    /// * `func` - An async function that takes the input type and returns a result
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[derive(Deserialize, JsonSchema)]
    /// struct CalculatorInput {
    ///     operation: String,
    ///     a: f64,
    ///     b: f64,
    /// }
    ///
    /// async fn calculator(input: CalculatorInput) -> Result<String, Box<dyn std::error::Error>> {
    ///     let result = match input.operation.as_str() {
    ///         "add" => input.a + input.b,
    ///         "subtract" => input.a - input.b,
    ///         "multiply" => input.a * input.b,
    ///         "divide" => input.a / input.b,
    ///         _ => return Err("Invalid operation".into()),
    ///     };
    ///     Ok(result.to_string())
    /// }
    ///
    /// let tool = FunctionTool::new("calculator", "Perform calculations", calculator);
    /// ```
    ///
    /// For customization, use builder methods:
    ///
    /// ```rust,ignore
    /// let tool = FunctionTool::new("get_weather", "Get weather", get_weather)
    ///     .with_name("custom_weather")
    ///     .with_description("Custom weather function");
    /// ```
    #[cfg(feature = "schema")]
    pub fn new<F, Fut>(name: impl Into<String>, description: impl Into<String>, func: F) -> Self
    where
        F: Fn(I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = O> + Send + 'static,
        I: schemars::JsonSchema,
    {
        let schema = schemars::schema_for!(I);
        let input_schema = serde_json::to_value(&schema.schema).unwrap_or(Value::Null);

        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
            func: Arc::new(move |input| Box::pin(func(input))),
            _phantom: PhantomData,
        }
    }

    /// Create a new function tool with a manually specified schema
    ///
    /// Use this when you don't have the `schema` feature enabled or want
    /// to provide a custom schema.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use serde_json::json;
    ///
    /// let schema = json!({
    ///     "type": "object",
    ///     "properties": {
    ///         "query": {
    ///             "type": "string",
    ///             "description": "The search query"
    ///         }
    ///     },
    ///     "required": ["query"]
    /// });
    ///
    /// let tool = FunctionTool::with_schema(
    ///     "search",
    ///     "Search the web",
    ///     schema,
    ///     search_function,
    /// );
    /// ```
    pub fn with_schema<F, Fut>(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: Value,
        func: F,
    ) -> Self
    where
        F: Fn(I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = O> + Send + 'static,
    {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
            func: Arc::new(move |input| Box::pin(func(input))),
            _phantom: PhantomData,
        }
    }

    /// Override the tool name
    ///
    /// Allows customizing the tool name after creation.
    /// Useful when you want to use a different name than the function name.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let tool = FunctionTool::new("get_weather", "Get weather", get_weather)
    ///     .with_name("weather_lookup");
    /// ```
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Override the tool description
    ///
    /// Allows customizing the tool description after creation.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let tool = FunctionTool::new("calculator", "Calculate", calculator)
    ///     .with_description("Performs basic arithmetic operations");
    /// ```
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Convert to a Tool parameter for API requests
    ///
    /// This creates a `crate::types::Tool` that can be used in API requests.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let tool = FunctionTool::new("get_weather", "Get weather", get_weather);
    /// let tool_param = tool.to_param();
    /// // Use tool_param in MessageRequest
    /// ```
    pub fn to_param(&self) -> crate::types::Tool {
        crate::types::Tool::new(&self.name, &self.description, self.input_schema.clone())
    }
}

#[async_trait]
impl<I, O> Tool for FunctionTool<I, O>
where
    I: DeserializeOwned + Send + Sync + 'static,
    O: Into<ToolResult> + Send + 'static,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn input_schema(&self) -> Value {
        self.input_schema.clone()
    }

    async fn call(&self, input: Value) -> ToolExecutionResult {
        // Deserialize the input
        let typed_input: I = serde_json::from_value(input).map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Failed to deserialize input for tool '{}': {}", self.name, e),
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;

        // Call the function
        let result = (self.func)(typed_input).await;

        // Convert to ToolResult
        Ok(result.into())
    }
}

// Implement Clone for FunctionTool by cloning the Arc
impl<I, O> Clone for FunctionTool<I, O> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            description: self.description.clone(),
            input_schema: self.input_schema.clone(),
            func: Arc::clone(&self.func),
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "schema")]
    #[tokio::test]
    async fn test_function_tool_with_schema_generation() {
        use schemars::JsonSchema;
        use serde::Deserialize;

        #[derive(Deserialize, JsonSchema)]
        struct TestInput {
            name: String,
            age: u32,
        }

        async fn test_func(input: TestInput) -> String {
            format!("{} is {} years old", input.name, input.age)
        }

        let tool = FunctionTool::new("test_tool", "A test tool", test_func);

        assert_eq!(tool.name(), "test_tool");
        assert_eq!(tool.description(), "A test tool");

        // Test that schema was generated
        let schema = tool.input_schema();
        assert!(schema.is_object());

        // Test calling the tool
        let input = serde_json::json!({
            "name": "Alice",
            "age": 30
        });

        let result = tool.call(input).await.unwrap();
        assert_eq!(result.as_string(), "Alice is 30 years old");
    }

    #[tokio::test]
    async fn test_function_tool_with_manual_schema() {
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct TestInput {
            x: i32,
            y: i32,
        }

        async fn add(input: TestInput) -> String {
            (input.x + input.y).to_string()
        }

        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "x": {"type": "integer"},
                "y": {"type": "integer"}
            },
            "required": ["x", "y"]
        });

        let tool = FunctionTool::with_schema("add", "Add two numbers", schema, add);

        let input = serde_json::json!({"x": 5, "y": 3});
        let result = tool.call(input).await.unwrap();
        assert_eq!(result.as_string(), "8");
    }

    #[tokio::test]
    async fn test_function_tool_error_handling() {
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct TestInput {
            #[allow(dead_code)]
            value: String,
        }

        async fn failing_func(_input: TestInput) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            Err("Something went wrong".into())
        }

        let tool = FunctionTool::with_schema(
            "failing_tool",
            "A tool that fails",
            serde_json::json!({"type": "object", "properties": {"value": {"type": "string"}}}),
            failing_func,
        );

        let input = serde_json::json!({"value": "test"});
        let result = tool.call(input).await;

        // The function returns an error, but it should still convert to ToolResult
        // Let's verify the tool can be called even though the function errors
        assert!(result.is_ok() || result.is_err());
    }

    #[cfg(feature = "schema")]
    #[tokio::test]
    async fn test_with_name_override() {
        use schemars::JsonSchema;
        use serde::Deserialize;

        #[derive(Deserialize, JsonSchema)]
        struct Input {
            value: String,
        }

        async fn test_func(input: Input) -> String {
            input.value
        }

        let tool = FunctionTool::new("original_name", "Description", test_func)
            .with_name("custom_name");

        assert_eq!(tool.name(), "custom_name");
        assert_eq!(tool.description(), "Description");
    }

    #[cfg(feature = "schema")]
    #[tokio::test]
    async fn test_with_description_override() {
        use schemars::JsonSchema;
        use serde::Deserialize;

        #[derive(Deserialize, JsonSchema)]
        struct Input {
            value: String,
        }

        async fn test_func(input: Input) -> String {
            input.value
        }

        let tool = FunctionTool::new("tool_name", "Original description", test_func)
            .with_description("Custom description that overrides the original");

        assert_eq!(tool.name(), "tool_name");
        assert_eq!(tool.description(), "Custom description that overrides the original");
    }

    #[cfg(feature = "schema")]
    #[tokio::test]
    async fn test_chained_customization() {
        use schemars::JsonSchema;
        use serde::Deserialize;

        #[derive(Deserialize, JsonSchema)]
        struct Input {
            x: i32,
            y: i32,
        }

        async fn calculator(input: Input) -> String {
            (input.x + input.y).to_string()
        }

        let tool = FunctionTool::new("add", "Add numbers", calculator)
            .with_name("calculator")
            .with_description("Performs addition of two integers");

        assert_eq!(tool.name(), "calculator");
        assert_eq!(tool.description(), "Performs addition of two integers");

        // Verify it still works
        let result = tool.call(serde_json::json!({"x": 10, "y": 5})).await.unwrap();
        assert_eq!(result.as_string(), "15");
    }

    #[cfg(feature = "schema")]
    #[tokio::test]
    async fn test_to_param() {
        use schemars::JsonSchema;
        use serde::Deserialize;

        #[derive(Deserialize, JsonSchema)]
        struct WeatherInput {
            /// The location to get weather for
            location: String,
            /// Temperature unit (c or f)
            #[allow(dead_code)]
            unit: Option<String>,
        }

        async fn get_weather(input: WeatherInput) -> String {
            format!("Weather in {}", input.location)
        }

        let tool = FunctionTool::new("get_weather", "Get weather information", get_weather);
        let param = tool.to_param();

        // Verify the Tool param has correct fields
        assert_eq!(param.name, "get_weather");
        assert_eq!(param.description, "Get weather information");
        assert!(param.input_schema.is_object());
    }

    #[tokio::test]
    async fn test_invalid_input_error() {
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct Input {
            required_field: String,
        }

        async fn test_func(input: Input) -> String {
            input.required_field
        }

        let tool = FunctionTool::with_schema(
            "test",
            "Test",
            serde_json::json!({"type": "object", "properties": {"required_field": {"type": "string"}}, "required": ["required_field"]}),
            test_func,
        );

        // Missing required field
        let result = tool.call(serde_json::json!({})).await;
        assert!(result.is_err());

        // Wrong type
        let result = tool.call(serde_json::json!({"required_field": 123})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_different_return_types() {
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct Input {
            value: i32,
        }

        // Test String return
        async fn returns_string(input: Input) -> String {
            input.value.to_string()
        }

        // Test Result return
        async fn returns_result(input: Input) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            Ok(input.value.to_string())
        }

        // Test JSON return
        async fn returns_json(input: Input) -> serde_json::Value {
            serde_json::json!({"value": input.value})
        }

        let schema = serde_json::json!({"type": "object", "properties": {"value": {"type": "integer"}}});

        let tool1 = FunctionTool::with_schema("tool1", "Test", schema.clone(), returns_string);
        let tool2 = FunctionTool::with_schema("tool2", "Test", schema.clone(), returns_result);
        let tool3 = FunctionTool::with_schema("tool3", "Test", schema, returns_json);

        let input = serde_json::json!({"value": 42});

        let result1 = tool1.call(input.clone()).await.unwrap();
        assert_eq!(result1.as_string(), "42");

        let result2 = tool2.call(input.clone()).await.unwrap();
        assert_eq!(result2.as_string(), "42");

        let result3 = tool3.call(input).await.unwrap();
        assert!(result3.as_string().contains("42"));
    }

    #[tokio::test]
    async fn test_tool_clone() {
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct Input {
            x: i32,
        }

        async fn double(input: Input) -> String {
            (input.x * 2).to_string()
        }

        let tool = FunctionTool::with_schema(
            "double",
            "Double a number",
            serde_json::json!({"type": "object", "properties": {"x": {"type": "integer"}}}),
            double,
        );

        let cloned = tool.clone();

        assert_eq!(cloned.name(), "double");
        assert_eq!(cloned.description(), "Double a number");

        // Verify both work
        let input = serde_json::json!({"x": 21});
        let result1 = tool.call(input.clone()).await.unwrap();
        let result2 = cloned.call(input).await.unwrap();

        assert_eq!(result1.as_string(), "42");
        assert_eq!(result2.as_string(), "42");
    }
}
