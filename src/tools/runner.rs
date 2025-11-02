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
/// use anthropic::{Client, MessageRequest, tools::ToolRunner};
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
                crate::types::Tool::new(
                    tool.name(),
                    tool.description(),
                    tool.input_schema(),
                )
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

            debug!("Tool runner iteration {}/{}", iteration, self.max_iterations);

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
                content: message.content.iter().map(|block| {
                    match block {
                        ContentBlock::Text { text } => ContentBlockParam::Text {
                            text: text.clone(),
                        },
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
                }).collect(),
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
    /// use anthropic::streaming::StreamEvent;
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
                crate::types::Tool::new(
                    tool.name(),
                    tool.description(),
                    tool.input_schema(),
                )
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

            debug!("Tool runner streaming iteration {}/{}", iteration, self.max_iterations);

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
                content: message.content.iter().map(|block| {
                    match block {
                        ContentBlock::Text { text } => ContentBlockParam::Text {
                            text: text.clone(),
                        },
                        ContentBlock::ToolUse { id, name, input: _ } => {
                            ContentBlockParam::Text {
                                text: format!("[Tool use: {} - {}]", name, id),
                            }
                        }
                        _ => ContentBlockParam::Text {
                            text: "[Other content]".to_string(),
                        },
                    }
                }).collect(),
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
    use crate::tools::FunctionTool;

    #[tokio::test]
    async fn test_tool_runner_add_tools() {
        use serde::Deserialize;

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
}
