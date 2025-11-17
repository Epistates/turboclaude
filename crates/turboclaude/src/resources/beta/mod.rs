//! Beta API features
//!
//! This namespace contains experimental features that may change or be removed
//! in future versions. All beta features require the `anthropic-beta` header.

use super::Resource;
use crate::client::Client;
use std::sync::OnceLock;
use tracing::{debug, info, warn};

pub use files::Files;
pub use models::Models;
pub use skills::Skills;

// Beta submodules
mod files;
mod models;
mod skills;

// Beta API version constants
/// Beta version for Extended Thinking API
pub const BETA_EXTENDED_THINKING: &str = "extended-thinking-2025-02-15";

/// Beta version for Files API
pub const BETA_FILES_API: &str = "files-api-2025-04-14";

/// Beta version for Tool Runners
pub const BETA_TOOL_RUNNERS: &str = "tool-runners-2025-03-01";

/// Beta version for Computer Use tools
pub const BETA_COMPUTER_USE: &str = "computer-use-2025-01-24";

/// Beta version for Skills API
pub const BETA_SKILLS_API: &str = "skills-2025-10-02";

/// Beta API features container.
///
/// Access beta/experimental features through `client.beta()`.
///
/// # Example
///
/// ```rust,no_run
/// # use turboclaude::Client;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("sk-ant-...");
///
/// // Access beta messages (when implemented)
/// let beta_messages = client.beta().messages();
///
/// // Access beta tools (when implemented)
/// let beta_tools = client.beta().tools();
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Beta {
    client: Client,
    messages: OnceLock<BetaMessages>,
    tools: OnceLock<BetaTools>,
    files: OnceLock<Files>,
    models: OnceLock<Models>,
    skills: OnceLock<Skills>,
}

impl Beta {
    /// Create a new Beta resource.
    pub(crate) fn new(client: Client) -> Self {
        Self {
            client,
            messages: OnceLock::new(),
            tools: OnceLock::new(),
            files: OnceLock::new(),
            models: OnceLock::new(),
            skills: OnceLock::new(),
        }
    }

    /// Access beta messages features.
    ///
    /// Provides access to extended thinking and other beta message features.
    /// Uses lazy initialization with `OnceLock` for zero-allocation access
    /// after the first call.
    pub fn messages(&self) -> &BetaMessages {
        self.messages
            .get_or_init(|| BetaMessages::new(self.client.clone()))
    }

    /// Access beta tools features.
    ///
    /// Provides access to tool runners and other beta tool features.
    /// Uses lazy initialization with `OnceLock` for zero-allocation access
    /// after the first call.
    pub fn tools(&self) -> &BetaTools {
        self.tools
            .get_or_init(|| BetaTools::new(self.client.clone()))
    }

    /// Access files API.
    ///
    /// Upload, download, and manage files for document analysis and other features.
    /// Uses lazy initialization with `OnceLock` for zero-allocation access
    /// after the first call.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    ///
    /// // Upload a file
    /// let file = client.beta().files().upload("data.csv").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn files(&self) -> &Files {
        self.files.get_or_init(|| Files::new(self.client.clone()))
    }

    /// Access skills API.
    ///
    /// Create and manage agent skills with low-latency tool integration.
    /// Uses lazy initialization with `OnceLock` for zero-allocation access
    /// after the first call.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    ///
    /// // Create a skill
    /// let skill = client.beta().skills()
    ///     .create()
    ///     .file("SKILL.md", b"---\nname: my-skill\n...".to_vec())
    ///     .display_title("My Skill")
    ///     .send()
    ///     .await?;
    ///
    /// // List skills
    /// let page = client.beta().skills().list().send().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn skills(&self) -> &Skills {
        self.skills.get_or_init(|| Skills::new(self.client.clone()))
    }

    /// Access models API.
    ///
    /// List and retrieve model information including model IDs and display names.
    /// Uses lazy initialization with `OnceLock` for zero-allocation access
    /// after the first call.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    ///
    /// // List models
    /// let page = client.beta().models().list().send().await?;
    /// for model in &page.data {
    ///     println!("{}: {}", model.id, model.display_name);
    /// }
    ///
    /// // Retrieve a specific model
    /// let model = client.beta().models()
    ///     .retrieve("claude-3-5-sonnet-20241022")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn models(&self) -> &Models {
        self.models.get_or_init(|| Models::new(self.client.clone()))
    }
}

impl Resource for Beta {
    fn client(&self) -> &Client {
        &self.client
    }
}

/// Beta messages features.
///
/// Provides access to extended thinking, advanced prompt caching,
/// and other experimental message features.
#[derive(Clone)]
pub struct BetaMessages {
    client: Client,
}

impl BetaMessages {
    /// Create a new BetaMessages resource.
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Create a message with extended thinking (beta feature).
    ///
    /// Extended thinking enables the model to spend more compute on difficult reasoning tasks.
    /// The model will return thinking blocks along with its response.
    ///
    /// # Arguments
    ///
    /// * `request` - The message request with thinking configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the extended thinking configuration is invalid.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use turboclaude::{Client, MessageRequest, Message};
    /// use turboclaude::types::beta::ThinkingConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    ///
    /// let mut request = MessageRequest::builder()
    ///     .model("claude-3-7-sonnet-20250219")
    ///     .max_tokens(16000u32)
    ///     .messages(vec![
    ///         Message::user("Solve this complex math problem: 2^100 mod 13")
    ///     ])
    ///     .thinking(ThinkingConfig::new(5000))  // 5000 tokens for thinking
    ///     .build()?;
    ///
    /// let response = client.beta().messages().create_with_thinking(request).await?;
    /// // Response will include thinking blocks and final answer
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(skip(self, request), fields(
        model = %request.model,
        max_tokens = request.max_tokens,
        thinking_budget = request.thinking.as_ref().map(|t| t.budget_tokens)
    ))]
    pub async fn create_with_thinking(
        &self,
        request: crate::types::MessageRequest,
    ) -> crate::error::Result<crate::types::Message> {
        debug!("Creating message with extended thinking");

        // Validate the complete request
        if let Err(e) = crate::validation::validate_message_request(&request) {
            warn!("Extended thinking request validation failed: {}", e);
            return Err(e);
        }

        debug!("Sending extended thinking request to API");
        let start = std::time::Instant::now();

        let result: crate::error::Result<crate::types::Message> = self
            .client
            .beta_request(
                crate::http::Method::POST,
                "/v1/messages",
                BETA_EXTENDED_THINKING,
            )?
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?
            .parse_result();

        let elapsed = start.elapsed();
        match &result {
            Ok(message) => {
                info!(
                    elapsed_ms = elapsed.as_millis(),
                    input_tokens = message.usage.input_tokens,
                    output_tokens = message.usage.output_tokens,
                    "Extended thinking message created successfully"
                );
            }
            Err(e) => {
                warn!(
                    elapsed_ms = elapsed.as_millis(),
                    error = %e,
                    "Extended thinking message creation failed"
                );
            }
        }

        result
    }

    /// Create a streaming message with extended thinking (beta feature).
    ///
    /// Similar to `create_with_thinking` but returns a stream of events as the message
    /// is generated, allowing for real-time display of thinking and response content.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use turboclaude::{Client, MessageRequest, Message};
    /// use turboclaude::types::beta::ThinkingConfig;
    /// use turboclaude::streaming::StreamEvent;
    /// use futures::StreamExt;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    ///
    /// let mut request = MessageRequest::builder()
    ///     .model("claude-3-7-sonnet-20250219")
    ///     .max_tokens(16000u32)
    ///     .messages(vec![Message::user("Complex problem...")])
    ///     .thinking(ThinkingConfig::new(5000))
    ///     .stream(true)
    ///     .build()?;
    ///
    /// let mut stream = client.beta().messages().stream_with_thinking(request).await?;
    ///
    /// while let Some(event) = stream.next().await {
    ///     match event? {
    ///         StreamEvent::ContentBlockDelta(event) => {
    ///             print!("{}", event.delta.text.unwrap_or_default());
    ///         }
    ///         StreamEvent::MessageStop => break,
    ///         _ => {}
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(skip(self, request), fields(
        model = %request.model,
        max_tokens = request.max_tokens,
        thinking_budget = request.thinking.as_ref().map(|t| t.budget_tokens)
    ))]
    pub async fn stream_with_thinking(
        &self,
        mut request: crate::types::MessageRequest,
    ) -> crate::error::Result<crate::streaming::MessageStream> {
        debug!("Creating streaming message with extended thinking");

        // Validate the complete request
        if let Err(e) = crate::validation::validate_message_request(&request) {
            warn!("Stream with thinking request validation failed: {}", e);
            return Err(e);
        }

        // Ensure streaming is enabled
        request.stream = Some(true);
        debug!("Opening extended thinking stream");

        let result = self
            .client
            .beta_request(
                crate::http::Method::POST,
                "/v1/messages",
                BETA_EXTENDED_THINKING,
            )?
            .body(serde_json::to_vec(&request)?)
            .send_streaming()
            .await
            .map(crate::streaming::MessageStream::new);

        match &result {
            Ok(_) => {
                info!("Extended thinking stream started successfully");
            }
            Err(e) => {
                warn!(error = %e, "Failed to start extended thinking stream");
            }
        }

        result
    }

    /// Create a tool runner for automatic tool execution with beta features.
    ///
    /// This provides access to the tool runner system that automatically handles
    /// tool execution loops, compatible with beta API features like extended thinking.
    ///
    /// Note: Unlike other resources, this returns an owned `ToolRunner` because
    /// tool runners are stateful builders that are consumed during execution.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "schema")]
    /// # use turboclaude::{Client, Message, MessageRequest};
    /// # #[cfg(feature = "schema")]
    /// # use turboclaude::tools::{FunctionTool, ToolRunner};
    /// # #[cfg(feature = "schema")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    ///
    /// // Create a tool runner through beta API
    /// let runner = client.beta().messages().tool_runner();
    ///
    /// // Use it the same way as the regular tool runner
    /// let final_message = runner
    ///     .add_tool(my_tool)
    ///     .with_max_iterations(5)
    ///     .run(request)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "schema")]
    pub fn tool_runner(&self) -> crate::tools::ToolRunner {
        crate::tools::ToolRunner::new(self.client.clone())
    }

    /// Create a message with structured output parsing (beta feature).
    ///
    /// This method enables type-safe structured outputs using JSON schema.
    /// The model will return JSON that conforms to the schema of type `T`,
    /// which is automatically parsed and validated.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type to parse the output into. Must implement `Serialize`,
    ///   `Deserialize`, and `JsonSchema`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use serde::{Deserialize, Serialize};
    /// use schemars::JsonSchema;
    /// use turboclaude_protocol::types::models;
    ///
    /// #[derive(Debug, Serialize, Deserialize, JsonSchema)]
    /// struct Order {
    ///     product_name: String,
    ///     price: f64,
    ///     quantity: u32,
    /// }
    ///
    /// let parsed = client.beta().messages()
    ///     .parse::<Order>()
    ///     .model(models::CLAUDE_SONNET_4_5_20250929_STRUCTURED_OUTPUTS)
    ///     .messages(vec![
    ///         Message::user("Extract order from: '2 Green Tea for $5.50 each'")
    ///     ])
    ///     .max_tokens(1024)
    ///     .send()
    ///     .await?;
    ///
    /// let order = parsed.parsed_output()?;
    /// println!("Product: {}, Quantity: {}", order.product_name, order.quantity);
    /// ```
    ///
    /// # Important
    ///
    /// - Requires `schema` feature flag
    /// - Use the structured outputs model: `CLAUDE_SONNET_4_5_20250929_STRUCTURED_OUTPUTS`
    /// - Requires beta header `structured-outputs-2025-09-17`
    /// - Output must be valid JSON matching the schema
    #[cfg(feature = "schema")]
    pub fn parse<T>(&self) -> ParseBuilder<T>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + schemars::JsonSchema,
    {
        ParseBuilder::new(self.client.clone())
    }
}

impl Resource for BetaMessages {
    fn client(&self) -> &Client {
        &self.client
    }
}

/// Beta tools features.
///
/// Provides access to tool runners, computer use tools,
/// and other experimental tool features.
#[derive(Clone)]
#[allow(dead_code)]
pub struct BetaTools {
    client: Client,
}

impl BetaTools {
    /// Create a new BetaTools resource.
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    // Note: Beta tools (computer use, bash, code execution, text editor, web search)
    // are defined as types in `types::beta::BetaToolParam` and passed as parameters
    // to message creation. See examples/computer_use.rs and types/beta/tools.rs
}

impl Resource for BetaTools {
    fn client(&self) -> &Client {
        &self.client
    }
}

/// Builder for creating structured output requests.
///
/// This builder allows you to configure a message request that will parse
/// the output into a strongly-typed structure.
///
/// # Type Parameters
///
/// * `T` - The type to parse the output into
#[cfg(feature = "schema")]
pub struct ParseBuilder<T> {
    client: Client,
    model: Option<String>,
    messages: Vec<crate::types::MessageParam>,
    max_tokens: u32,
    temperature: Option<f64>,
    top_p: Option<f64>,
    top_k: Option<u32>,
    stop_sequences: Vec<String>,
    system: Option<crate::types::SystemPrompt>,
    _phantom: std::marker::PhantomData<T>,
}

#[cfg(feature = "schema")]
impl<T> ParseBuilder<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned + schemars::JsonSchema,
{
    /// Create a new parse builder.
    fn new(client: Client) -> Self {
        Self {
            client,
            model: None,
            messages: Vec::new(),
            max_tokens: 1024,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: Vec::new(),
            system: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set the model to use.
    ///
    /// For structured outputs, you should use:
    /// `models::CLAUDE_SONNET_4_5_20250929_STRUCTURED_OUTPUTS`
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the messages for the conversation.
    pub fn messages(mut self, messages: Vec<crate::types::MessageParam>) -> Self {
        self.messages = messages;
        self
    }

    /// Set the maximum tokens to generate.
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set the temperature for sampling.
    pub fn temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set top_p for nucleus sampling.
    pub fn top_p(mut self, top_p: f64) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Set top_k for top-k sampling.
    pub fn top_k(mut self, top_k: u32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    /// Add a stop sequence.
    pub fn stop_sequence(mut self, sequence: impl Into<String>) -> Self {
        self.stop_sequences.push(sequence.into());
        self
    }

    /// Set the system prompt.
    pub fn system(mut self, system: impl Into<crate::types::SystemPrompt>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Send the request and get a parsed response.
    ///
    /// This method:
    /// 1. Generates a JSON schema from type `T`
    /// 2. Sends the request with the schema as `output_format`
    /// 3. Parses the response into `ParsedBetaMessage<T>`
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No model is specified
    /// - No messages are provided
    /// - The API request fails
    /// - The response cannot be parsed as JSON
    /// - The JSON doesn't match the schema for type `T`
    pub async fn send(self) -> crate::error::Result<crate::types::beta::ParsedBetaMessage<T>> {
        let model = self.model.ok_or_else(|| crate::Error::InvalidRequest(
            "Model is required for structured output requests".to_string()
        ))?;

        if self.messages.is_empty() {
            return Err(crate::Error::InvalidRequest(
                "At least one message is required".to_string()
            ));
        }

        // Generate JSON schema from type T
        let schema = crate::schema::generate_schema::<T>();

        // Build the request with output_format
        let mut request_body = serde_json::json!({
            "model": model,
            "max_tokens": self.max_tokens,
            "messages": self.messages,
            "output_format": {
                "type": "json_schema",
                "schema": schema
            }
        });

        // Add optional fields
        if let Some(temp) = self.temperature {
            request_body["temperature"] = serde_json::json!(temp);
        }
        if let Some(top_p) = self.top_p {
            request_body["top_p"] = serde_json::json!(top_p);
        }
        if let Some(top_k) = self.top_k {
            request_body["top_k"] = serde_json::json!(top_k);
        }
        if !self.stop_sequences.is_empty() {
            request_body["stop_sequences"] = serde_json::json!(self.stop_sequences);
        }
        if let Some(system) = self.system {
            request_body["system"] = serde_json::to_value(system)?;
        }

        // Send request with structured-outputs beta header
        debug!("Sending structured output request");
        let message: crate::types::beta::BetaMessage = self
            .client
            .beta_request(
                crate::http::Method::POST,
                "/v1/messages",
                "structured-outputs-2025-09-17",
            )?
            .body(serde_json::to_vec(&request_body)?)
            .send()
            .await?
            .parse_result()?;

        info!("Structured output message received successfully");

        Ok(crate::types::beta::ParsedBetaMessage::new(message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beta_resources_creation() {
        let client = Client::new("test-key");
        let beta = Beta::new(client.clone());

        // Test that we can create beta resources
        let _messages = beta.messages();
        let _tools = beta.tools();
        let _files = beta.files();

        // Verify beta resource can be accessed through client
        let beta_from_client = client.beta();
        let _messages2 = beta_from_client.messages();
    }

    #[cfg(feature = "schema")]
    #[test]
    fn test_beta_messages_tool_runner() {
        let client = Client::new("test-key");
        let beta_messages = client.beta().messages();

        // Test that we can create a tool runner through beta API
        let runner = beta_messages.tool_runner();

        // Verify it's a proper ToolRunner instance
        assert_eq!(runner.tool_count(), 0);
        assert_eq!(runner.tool_names(), Vec::<&str>::new());
    }

    #[cfg(feature = "schema")]
    #[tokio::test]
    async fn test_beta_tool_runner_with_tools() {
        use crate::tools::FunctionTool;
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct TestInput {
            value: String,
        }

        async fn test_tool(input: TestInput) -> String {
            format!("Beta: {}", input.value)
        }

        let client = Client::new("test-key");
        let tool = FunctionTool::with_schema(
            "beta_test_tool",
            "A beta test tool",
            serde_json::json!({"type": "object", "properties": {"value": {"type": "string"}}}),
            test_tool,
        );

        // Create runner through beta API
        let runner = client
            .beta()
            .messages()
            .tool_runner()
            .add_tool(tool)
            .with_max_iterations(3);

        assert_eq!(runner.tool_count(), 1);
        assert!(runner.has_tool("beta_test_tool"));
        assert_eq!(runner.tool_names(), vec!["beta_test_tool"]);
    }

    #[test]
    fn test_beta_sub_resources_lazy_initialization() {
        let client = Client::new("test-key");
        let beta = client.beta();

        // Test messages
        let messages1 = beta.messages();
        let messages2 = beta.messages();
        assert!(
            std::ptr::eq(messages1, messages2),
            "Multiple calls to messages() should return the same instance"
        );

        // Test tools
        let tools1 = beta.tools();
        let tools2 = beta.tools();
        assert!(
            std::ptr::eq(tools1, tools2),
            "Multiple calls to tools() should return the same instance"
        );

        // Test files
        let files1 = beta.files();
        let files2 = beta.files();
        assert!(
            std::ptr::eq(files1, files2),
            "Multiple calls to files() should return the same instance"
        );

        // Test skills
        let skills1 = beta.skills();
        let skills2 = beta.skills();
        assert!(
            std::ptr::eq(skills1, skills2),
            "Multiple calls to skills() should return the same instance"
        );

        // Test models
        let models1 = beta.models();
        let models2 = beta.models();
        assert!(
            std::ptr::eq(models1, models2),
            "Multiple calls to models() should return the same instance"
        );
    }
}
