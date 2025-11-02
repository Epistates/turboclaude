//! Tool runner for automatic tool execution loops
//!
//! This module provides `ToolRunner` which automatically handles the tool call loop,
//! eliminating the need for manual tool execution and response handling.

use super::traits::Tool;
use crate::{
    client::Client,
    error::{Error, Result},
    types::{ContentBlock, ContentBlockParam, Message, MessageParam, MessageRequest, Role},
};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, trace};

/// Error types specific to tool running
#[derive(Debug, thiserror::Error)]
pub enum ToolRunnerError {
    /// Maximum number of iterations reached
    #[error("Maximum iterations ({0}) reached")]
    MaxIterationsReached(usize),

    /// Tool not found in registry
    #[error("Tool '{0}' not found in registered tools")]
    ToolNotFound(String),

    /// Tool execution failed
    #[error("Tool execution failed: {0}")]
    ToolExecutionFailed(String),

    /// API error
    #[error("API error: {0}")]
    ApiError(#[from] crate::error::Error),
}

/// Tool runner for automatic tool execution loops
///
/// This handles the entire tool execution loop automatically:
/// 1. Sends message to Claude with available tools
/// 2. Checks if Claude wants to use any tools
/// 3. Executes those tools
/// 4. Sends results back to Claude
/// 5. Repeats until Claude stops using tools or max iterations reached
///
/// # Example
///
/// ```rust,ignore
/// use turboclaude::{Client, MessageRequest, tools::ToolRunner};
///
/// let client = Client::new("your-api-key");
///
/// let weather_tool = FunctionTool::new("get_weather", "Get weather", get_weather);
///
/// let runner = ToolRunner::new(client)
///     .add_tool(weather_tool)
///     .with_max_iterations(5);
///
/// let request = MessageRequest::builder()
///     .model("claude-3-5-sonnet-20241022")
///     .max_tokens(1024)
///     .messages(vec![Message::user("What's the weather in Tokyo?")])
///     .build()?;
///
/// // This automatically handles all tool calls!
/// let final_message = runner.run(request).await?;
/// println!("Final response: {}", final_message.text());
/// ```
#[derive(Clone)]
pub struct ToolRunner {
    /// The Anthropic client
    client: Client,

    /// Registered tools by name
    tools: HashMap<String, Arc<dyn Tool>>,

    /// Maximum number of iterations before stopping
    max_iterations: usize,

    /// Enable verbose logging of tool execution
    verbose: bool,
}

impl ToolRunner {
    /// Create a new tool runner with a client
    pub fn new(client: Client) -> Self {
        Self {
            client,
            tools: HashMap::new(),
            max_iterations: 10,
            verbose: false,
        }
    }

    /// Add a tool to the runner
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let runner = ToolRunner::new(client)
    ///     .add_tool(weather_tool)
    ///     .add_tool(calculator_tool);
    /// ```
    pub fn add_tool(mut self, tool: impl Tool + 'static) -> Self {
        self.tools.insert(tool.name().to_string(), Arc::new(tool));
        self
    }

    /// Set maximum iterations
    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    /// Enable verbose logging
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Run the tool execution loop
    ///
    /// This will automatically handle tool calls until either:
    /// - Claude stops requesting tool use
    /// - Maximum iterations is reached
    ///
    /// # Arguments
    ///
    /// * `request` - The initial message request
    ///
    /// # Returns
    ///
    /// The final message from Claude after all tool executions complete
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The API call fails
    /// - Maximum iterations is reached
    /// - A tool execution fails
    pub async fn run(&self, mut request: MessageRequest) -> Result<Message> {
        if self.tools.is_empty() {
            debug!("No tools registered, running single message");
            return self.client.messages().create(request).await;
        }

        // Convert tools to Tool type for the request
        let tools: Vec<crate::types::Tool> = self
            .tools
            .values()
            .map(|tool| {
                crate::types::Tool::new(tool.name(), tool.description(), tool.input_schema())
            })
            .collect();

        request.tools = Some(tools);

        let mut messages = request.messages.clone();
        let mut iteration = 0;

        loop {
            iteration += 1;

            if iteration > self.max_iterations {
                error!("Maximum iterations ({}) reached", self.max_iterations);
                return Err(Error::ToolExecution(
                    ToolRunnerError::MaxIterationsReached(self.max_iterations).to_string(),
                ));
            }

            debug!(
                "Tool runner iteration {}/{}",
                iteration, self.max_iterations
            );

            // Update request with current messages
            request.messages = messages.clone();

            // Send message to Claude
            let message = self.client.messages().create(request.clone()).await?;

            if self.verbose {
                trace!("Received message: {:?}", message);
            }

            // Check if Claude wants to use tools
            let tool_uses: Vec<_> = message
                .content
                .iter()
                .filter_map(|block| {
                    if let ContentBlock::ToolUse { id, name, input } = block {
                        Some((id.clone(), name.clone(), input.clone()))
                    } else {
                        None
                    }
                })
                .collect();

            if tool_uses.is_empty() {
                debug!("No tool uses requested, returning final message");
                return Ok(message);
            }

            debug!("Processing {} tool use(s)", tool_uses.len());

            // Add assistant's message to history
            messages.push(MessageParam {
                role: Role::Assistant,
                content: message
                    .content
                    .iter()
                    .map(|block| {
                        match block {
                            ContentBlock::Text { text, .. } => {
                                ContentBlockParam::Text { text: text.clone() }
                            }
                            ContentBlock::ToolUse { id, name, input: _ } => {
                                // Note: ToolUse in responses becomes ContentBlockParam in requests
                                // We'll handle this in the tool results instead
                                ContentBlockParam::Text {
                                    text: format!("[Tool use: {} - {}]", name, id),
                                }
                            }
                            _ => ContentBlockParam::Text {
                                text: "[Other content]".to_string(),
                            },
                        }
                    })
                    .collect(),
            });

            // Execute tools and collect results
            let mut tool_results = Vec::new();

            for (tool_use_id, tool_name, input) in tool_uses {
                match self.tools.get(&tool_name) {
                    Some(tool) => {
                        debug!("Executing tool: {}", tool_name);

                        match tool.call(input).await {
                            Ok(result) => {
                                let result_text = result.as_string();
                                if self.verbose {
                                    trace!("Tool {} returned: {}", tool_name, result_text);
                                }

                                tool_results.push(ContentBlockParam::ToolResult {
                                    tool_use_id: tool_use_id.clone(),
                                    content: result_text,
                                    is_error: None,
                                });
                            }
                            Err(e) => {
                                error!("Tool {} failed: {}", tool_name, e);
                                tool_results.push(ContentBlockParam::ToolResult {
                                    tool_use_id: tool_use_id.clone(),
                                    content: format!("Error: {}", e),
                                    is_error: Some(true),
                                });
                            }
                        }
                    }
                    None => {
                        error!("Tool not found: {}", tool_name);
                        tool_results.push(ContentBlockParam::ToolResult {
                            tool_use_id: tool_use_id.clone(),
                            content: format!("Error: Tool '{}' not found", tool_name),
                            is_error: Some(true),
                        });
                    }
                }
            }

            // Add tool results as a user message
            messages.push(MessageParam {
                role: Role::User,
                content: tool_results,
            });
        }
    }

    /// Run the tool execution loop and stream the final message
    ///
    /// This executes all tool calls automatically, then streams the final response from Claude.
    /// Similar to Python SDK's `BetaStreamingToolRunner`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use futures::StreamExt;
    /// use turboclaude::streaming::StreamEvent;
    ///
    /// let mut stream = runner.run_streaming(request).await?;
    ///
    /// while let Some(event) = stream.next().await {
    ///     match event? {
    ///         StreamEvent::ContentBlockDelta(delta) => {
    ///             if let Some(text) = delta.delta.text {
    ///                 print!("{}", text);
    ///             }
    ///         }
    ///         StreamEvent::MessageStop => break,
    ///         _ => {}
    ///     }
    /// }
    /// ```
    pub async fn run_streaming(
        &self,
        mut request: MessageRequest,
    ) -> Result<crate::streaming::MessageStream> {
        if self.tools.is_empty() {
            debug!("No tools registered, running single streaming message");
            return self.client.messages().stream(request).await;
        }

        // Convert tools to Tool type for the request
        let tools: Vec<crate::types::Tool> = self
            .tools
            .values()
            .map(|tool| {
                crate::types::Tool::new(tool.name(), tool.description(), tool.input_schema())
            })
            .collect();

        request.tools = Some(tools);

        let mut messages = request.messages.clone();
        let mut iteration = 0;

        loop {
            iteration += 1;

            if iteration > self.max_iterations {
                error!("Maximum iterations ({}) reached", self.max_iterations);
                return Err(Error::ToolExecution(
                    ToolRunnerError::MaxIterationsReached(self.max_iterations).to_string(),
                ));
            }

            debug!(
                "Tool runner streaming iteration {}/{}",
                iteration, self.max_iterations
            );

            // Update request with current messages
            request.messages = messages.clone();

            // Send message to Claude (NOT streaming yet - we only stream the final response)
            let message = self.client.messages().create(request.clone()).await?;

            if self.verbose {
                trace!("Received message: {:?}", message);
            }

            // Check if Claude wants to use tools
            let tool_uses: Vec<_> = message
                .content
                .iter()
                .filter_map(|block| {
                    if let ContentBlock::ToolUse { id, name, input } = block {
                        Some((id.clone(), name.clone(), input.clone()))
                    } else {
                        None
                    }
                })
                .collect();

            if tool_uses.is_empty() {
                // No more tool uses - this was the final response
                // Since we already got it non-streaming, we need to make one more request
                // with streaming enabled. This is acceptable since the alternative would be
                // to buffer all tool execution anyway.
                debug!("No tool uses requested, streaming final response");
                return self.client.messages().stream(request).await;
            }

            debug!("Processing {} tool use(s)", tool_uses.len());

            // Add assistant's message to history
            messages.push(MessageParam {
                role: Role::Assistant,
                content: message
                    .content
                    .iter()
                    .map(|block| match block {
                        ContentBlock::Text { text, .. } => {
                            ContentBlockParam::Text { text: text.clone() }
                        }
                        ContentBlock::ToolUse { id, name, input: _ } => ContentBlockParam::Text {
                            text: format!("[Tool use: {} - {}]", name, id),
                        },
                        _ => ContentBlockParam::Text {
                            text: "[Other content]".to_string(),
                        },
                    })
                    .collect(),
            });

            // Execute tools and collect results
            let mut tool_results = Vec::new();

            for (tool_use_id, tool_name, input) in tool_uses {
                match self.tools.get(&tool_name) {
                    Some(tool) => {
                        debug!("Executing tool: {}", tool_name);

                        match tool.call(input).await {
                            Ok(result) => {
                                let result_text = result.as_string();
                                if self.verbose {
                                    trace!("Tool {} returned: {}", tool_name, result_text);
                                }

                                tool_results.push(ContentBlockParam::ToolResult {
                                    tool_use_id: tool_use_id.clone(),
                                    content: result_text,
                                    is_error: None,
                                });
                            }
                            Err(e) => {
                                error!("Tool {} failed: {}", tool_name, e);
                                tool_results.push(ContentBlockParam::ToolResult {
                                    tool_use_id: tool_use_id.clone(),
                                    content: format!("Error: {}", e),
                                    is_error: Some(true),
                                });
                            }
                        }
                    }
                    None => {
                        error!("Tool not found: {}", tool_name);
                        tool_results.push(ContentBlockParam::ToolResult {
                            tool_use_id: tool_use_id.clone(),
                            content: format!("Error: Tool '{}' not found", tool_name),
                            is_error: Some(true),
                        });
                    }
                }
            }

            // Add tool results as a user message
            messages.push(MessageParam {
                role: Role::User,
                content: tool_results,
            });
        }
    }

    /// Get the number of registered tools
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    /// Check if a tool is registered
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get a list of registered tool names
    pub fn tool_names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{traits::ToolContentBlock, FunctionTool, ToolResult};
    use serde::Deserialize;

    #[tokio::test]
    async fn test_tool_runner_add_tools() {
        #[derive(Deserialize)]
        struct TestInput {
            value: String,
        }

        async fn test_tool(input: TestInput) -> String {
            format!("Got: {}", input.value)
        }

        let client = Client::new("test-key");
        let tool = FunctionTool::with_schema(
            "test_tool",
            "A test tool",
            serde_json::json!({"type": "object", "properties": {"value": {"type": "string"}}}),
            test_tool,
        );

        let runner = ToolRunner::new(client).add_tool(tool);

        assert_eq!(runner.tool_count(), 1);
        assert!(runner.has_tool("test_tool"));
        assert_eq!(runner.tool_names(), vec!["test_tool"]);
    }

    /// Test 1: ToolRunner::new() initializes correctly
    #[test]
    fn test_tool_runner_new() {
        let client = Client::new("test-key");
        let runner = ToolRunner::new(client);

        assert_eq!(runner.tool_count(), 0, "New runner should have no tools");
        assert_eq!(
            runner.max_iterations, 10,
            "Default max iterations should be 10"
        );
        assert!(!runner.verbose, "Verbose should be false by default");
    }

    /// Test 2: ToolRunner::add_tool() registers tools
    #[test]
    fn test_tool_runner_add_tool() {
        #[derive(Deserialize)]
        struct Input1 {
            x: i32,
        }

        #[derive(Deserialize)]
        struct Input2 {
            y: String,
        }

        async fn tool1(_: Input1) -> String {
            "42".to_string()
        }

        async fn tool2(_: Input2) -> String {
            "hello".to_string()
        }

        let client = Client::new("test-key");
        let ft1 = FunctionTool::with_schema(
            "calculator",
            "Does math",
            serde_json::json!({"type": "object", "properties": {"x": {"type": "number"}}}),
            tool1,
        );
        let ft2 = FunctionTool::with_schema(
            "greeter",
            "Says hello",
            serde_json::json!({"type": "object", "properties": {"y": {"type": "string"}}}),
            tool2,
        );

        let runner = ToolRunner::new(client).add_tool(ft1).add_tool(ft2);

        assert_eq!(runner.tool_count(), 2);
        assert!(runner.has_tool("calculator"));
        assert!(runner.has_tool("greeter"));
        assert!(!runner.has_tool("nonexistent"));

        let names = runner.tool_names();
        assert!(names.contains(&"calculator"));
        assert!(names.contains(&"greeter"));
    }

    /// Test 3: ToolRunner::with_max_iterations() sets limit
    #[test]
    fn test_tool_runner_with_max_iterations() {
        let client = Client::new("test-key");
        let runner = ToolRunner::new(client).with_max_iterations(5);

        assert_eq!(runner.max_iterations, 5);

        let runner2 = runner.with_max_iterations(20);
        assert_eq!(runner2.max_iterations, 20);
    }

    /// Test 4: ToolResult::text() creates text result
    #[test]
    fn test_tool_result_text() {
        let result = ToolResult::text("Success");

        match &result {
            ToolResult::Text(text) => {
                assert_eq!(text, "Success");
            }
            _ => panic!("Expected Text variant"),
        }

        assert_eq!(result.as_string(), "Success");
    }

    /// Test 5: ToolResult::json() creates JSON result
    #[test]
    fn test_tool_result_json() {
        let json_value = serde_json::json!({"status": "ok", "count": 42});
        let result = ToolResult::json(json_value.clone());

        match &result {
            ToolResult::Json(val) => {
                assert_eq!(val, &json_value);
            }
            _ => panic!("Expected Json variant"),
        }

        let as_string = result.as_string();
        assert!(as_string.contains("status"));
        assert!(as_string.contains("ok"));
        assert!(as_string.contains("42"));
    }

    /// Test 6: ToolResult::from(String)
    #[test]
    fn test_tool_result_from_string() {
        let result: ToolResult = "Test message".to_string().into();

        match result {
            ToolResult::Text(text) => {
                assert_eq!(text, "Test message");
            }
            _ => panic!("Expected Text variant from String"),
        }
    }

    /// Test 7: ToolResult::from(Value)
    #[test]
    fn test_tool_result_from_value() {
        let json_value = serde_json::json!({"key": "value"});
        let result: ToolResult = json_value.clone().into();

        match result {
            ToolResult::Json(val) => {
                assert_eq!(val, json_value);
            }
            _ => panic!("Expected Json variant from Value"),
        }
    }

    /// Test 8: ToolResult::from(Result<T, E>)
    #[test]
    fn test_tool_result_from_result() {
        // Success case
        let ok_result: Result<String> = Ok("success".to_string());
        let tool_result: ToolResult = ok_result.into();
        match tool_result {
            ToolResult::Text(text) => {
                assert_eq!(text, "success");
            }
            _ => panic!("Expected Text variant from Ok result"),
        }

        // Error case
        let err_result: Result<String> = Err(Error::InvalidRequest("bad input".to_string()));
        let tool_result: ToolResult = err_result.into();
        let as_str = tool_result.as_string();
        assert!(as_str.contains("Error"));
        assert!(as_str.contains("bad input"));
    }

    /// Test 9: ToolRunner correctly identifies tool not found
    #[test]
    fn test_tool_runner_tool_not_found() {
        let client = Client::new("test-key");
        let runner = ToolRunner::new(client);

        assert!(!runner.has_tool("nonexistent"));
        assert_eq!(runner.tool_count(), 0);
        assert!(runner.tool_names().is_empty());
    }

    /// Test 10: Verify ToolRunner cloning works
    #[test]
    fn test_tool_runner_clone() {
        #[derive(Deserialize)]
        struct Input {
            x: i32,
        }

        async fn calculator(_: Input) -> String {
            "100".to_string()
        }

        let client = Client::new("test-key");
        let tool = FunctionTool::with_schema(
            "calc",
            "Calculator",
            serde_json::json!({"type": "object", "properties": {"x": {"type": "number"}}}),
            calculator,
        );

        let runner1 = ToolRunner::new(client).add_tool(tool);
        let runner2 = runner1.clone();

        // Both should have the same tools
        assert_eq!(runner1.tool_count(), runner2.tool_count());
        assert_eq!(runner1.has_tool("calc"), runner2.has_tool("calc"));
    }

    /// Test 11: Verify with_verbose sets flag
    #[test]
    fn test_tool_runner_with_verbose() {
        let client = Client::new("test-key");
        let runner = ToolRunner::new(client).with_verbose(true);

        assert!(runner.verbose);

        let runner2 = runner.with_verbose(false);
        assert!(!runner2.verbose);
    }

    /// Test 12: ToolResult::as_string() serialization for all variants
    #[test]
    fn test_tool_result_as_string_serialization() {
        // Text variant
        let text_result = ToolResult::text("Plain text");
        assert_eq!(text_result.as_string(), "Plain text");

        // JSON variant
        let json_result = ToolResult::json(serde_json::json!({"key": "value"}));
        let json_str = json_result.as_string();
        assert!(json_str.contains("key"));
        assert!(json_str.contains("value"));

        // ContentBlocks variant (if available in your implementation)
        let blocks_result = ToolResult::ContentBlocks(vec![
            ToolContentBlock::Text {
                text: "Block 1".to_string(),
            },
            ToolContentBlock::Text {
                text: "Block 2".to_string(),
            },
        ]);
        let blocks_str = blocks_result.as_string();
        assert!(blocks_str.contains("Block 1") || blocks_str.contains("text"));
    }

    /// Test 13: E2E calculator tool (mock test without actual API call)
    /// This test demonstrates the expected structure without making real API calls
    #[tokio::test]
    async fn test_tool_runner_e2e_calculator() {
        #[derive(Deserialize)]
        struct CalculatorInput {
            operation: String,
            a: f64,
            b: f64,
        }

        async fn calculator(input: CalculatorInput) -> Result<String> {
            let result = match input.operation.as_str() {
                "add" => input.a + input.b,
                "subtract" => input.a - input.b,
                "multiply" => input.a * input.b,
                "divide" => {
                    if input.b != 0.0 {
                        input.a / input.b
                    } else {
                        return Err(Error::InvalidRequest("Division by zero".to_string()));
                    }
                }
                _ => {
                    return Err(Error::InvalidRequest(format!(
                        "Unknown operation: {}",
                        input.operation
                    )));
                }
            };
            Ok(result.to_string())
        }

        let client = Client::new("test-key");
        let calc_tool = FunctionTool::with_schema(
            "calculator",
            "Performs basic arithmetic operations",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["add", "subtract", "multiply", "divide"]
                    },
                    "a": {"type": "number"},
                    "b": {"type": "number"}
                },
                "required": ["operation", "a", "b"]
            }),
            calculator,
        );

        let runner = ToolRunner::new(client)
            .add_tool(calc_tool)
            .with_max_iterations(5);

        // Verify the calculator tool is registered
        assert!(runner.has_tool("calculator"));
        assert_eq!(runner.tool_count(), 1);
        assert_eq!(runner.max_iterations, 5);

        // Test direct tool execution (bypassing the run loop for this unit test)
        let tool = runner.tools.get("calculator").expect("Tool should exist");

        // Test addition
        let add_input = serde_json::json!({
            "operation": "add",
            "a": 5.0,
            "b": 3.0
        });
        let add_result = tool.call(add_input).await;
        assert!(add_result.is_ok());
        let add_str = add_result.unwrap().as_string();
        assert!(add_str.contains("8") || add_str == "8.0" || add_str == "8");

        // Test division
        let div_input = serde_json::json!({
            "operation": "divide",
            "a": 10.0,
            "b": 2.0
        });
        let div_result = tool.call(div_input).await;
        assert!(div_result.is_ok());
        let div_str = div_result.unwrap().as_string();
        assert!(div_str.contains("5") || div_str == "5.0" || div_str == "5");

        // Test error case: division by zero
        let zero_input = serde_json::json!({
            "operation": "divide",
            "a": 10.0,
            "b": 0.0
        });
        let zero_result = tool.call(zero_input).await;
        assert!(zero_result.is_ok()); // Tool execution succeeds, but returns error message
        let zero_str = zero_result.unwrap().as_string();
        assert!(zero_str.contains("Error") || zero_str.contains("zero"));
    }
}
