//! Beta API features
//!
//! This namespace contains experimental features that may change or be removed
//! in future versions. All beta features require the `anthropic-beta` header.

use crate::client::Client;

pub use files::Files;

// Beta submodules
mod files;

// Beta API version constants
/// Beta version for Extended Thinking API
pub const BETA_EXTENDED_THINKING: &str = "extended-thinking-2025-02-15";

/// Beta version for Files API
pub const BETA_FILES_API: &str = "files-api-2025-04-14";

/// Beta version for Tool Runners
pub const BETA_TOOL_RUNNERS: &str = "tool-runners-2025-03-01";

/// Beta version for Computer Use tools
pub const BETA_COMPUTER_USE: &str = "computer-use-2025-01-24";

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
}

impl Beta {
    /// Create a new Beta resource.
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Access beta messages features.
    ///
    /// Provides access to extended thinking and other beta message features.
    pub fn messages(&self) -> BetaMessages {
        BetaMessages::new(self.client.clone())
    }

    /// Access beta tools features.
    ///
    /// Provides access to tool runners and other beta tool features.
    pub fn tools(&self) -> BetaTools {
        BetaTools::new(self.client.clone())
    }

    /// Access files API.
    ///
    /// Upload, download, and manage files for document analysis and other features.
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
    pub fn files(&self) -> Files {
        Files::new(self.client.clone())
    }
}

/// Beta messages features.
///
/// Provides access to extended thinking, advanced prompt caching,
/// and other experimental message features.
#[derive(Clone)]
pub struct BetaMessages {
    _client: Client,
}

impl BetaMessages {
    /// Create a new BetaMessages resource.
    pub(crate) fn new(client: Client) -> Self {
        Self { _client: client }
    }

    /// Create a tool runner for automatic tool execution with beta features.
    ///
    /// This provides access to the tool runner system that automatically handles
    /// tool execution loops, compatible with beta API features like extended thinking.
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

    // TODO: Implement extended thinking support
}

/// Beta tools features.
///
/// Provides access to tool runners, computer use tools,
/// and other experimental tool features.
#[allow(dead_code)]
pub struct BetaTools {
    client: Client,
}

impl BetaTools {
    /// Create a new BetaTools resource.
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    // TODO: Implement tool runners (4 variants)
    // TODO: Implement computer use tools
    // TODO: Implement bash tools
    // TODO: Implement code execution tools
}

impl Resource for BetaTools {
    fn client(&self) -> &Client {
        &self.client
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
}
