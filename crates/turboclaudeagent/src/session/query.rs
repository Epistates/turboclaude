//! Query handling and message streaming
//!
//! Provides methods for executing queries, receiving messages, and managing
//! query builders for the agent session.

use crate::error::{AgentError, Result as AgentResult};
use crate::session::core::AgentSession;
use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::sync::atomic::Ordering;
use turboclaude_protocol::{Message, QueryRequest, QueryResponse, RequestId, ToolDefinition};

impl AgentSession {
    /// Execute a query with the agent
    ///
    /// This is the primary entry point for running queries. The session will:
    /// 1. Send the query to Claude
    /// 2. Handle any hook events from Claude
    /// 3. Evaluate permission checks
    /// 4. Return the final response
    pub async fn query(&self, request: QueryRequest) -> AgentResult<QueryResponse> {
        // Validate request
        if request.query.is_empty() {
            return Err(AgentError::Config("Query cannot be empty".into()));
        }
        if request.max_tokens == 0 {
            return Err(AgentError::Config("max_tokens must be > 0".into()));
        }

        // Ensure connected (auto-reconnect if needed)
        self.ensure_connected().await?;

        // Generate request ID
        let request_id = RequestId::new();

        // Increment active queries
        let count = self.active_queries.fetch_add(1, Ordering::Relaxed);

        // Check if we've exceeded max concurrent queries
        if count as usize >= self.config.max_concurrent_queries {
            self.active_queries.fetch_sub(1, Ordering::Relaxed);
            return Err(AgentError::Protocol(format!(
                "Too many concurrent queries (max: {})",
                self.config.max_concurrent_queries
            )));
        }

        // Get router
        let router_lock = self.router.lock().await;
        let router = match router_lock.as_ref() {
            Some(r) => r,
            None => {
                self.active_queries.fetch_sub(1, Ordering::Relaxed);
                return Err(AgentError::Transport("Router not initialized".into()));
            }
        };

        // Send query via router
        let response = router.send_query(request_id, request).await;

        // Decrement active queries
        self.active_queries.fetch_sub(1, Ordering::Relaxed);

        // Return response
        response
    }

    /// Execute a simple query with just a string (convenience method)
    ///
    /// Returns a `QueryBuilder` that can be awaited directly or chained with
    /// additional configuration methods.
    ///
    /// # Examples
    ///
    /// Simple usage (defaults):
    /// ```no_run
    /// # use turboclaudeagent::ClaudeAgentClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = ClaudeAgentClient::builder().api_key("key").build()?;
    /// # let client = ClaudeAgentClient::new(config);
    /// let session = client.create_session().await?;
    /// let response = session.query_str("What is 2+2?").await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// With chained configuration:
    /// ```no_run
    /// # use turboclaudeagent::ClaudeAgentClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = ClaudeAgentClient::builder().api_key("key").build()?;
    /// # let client = ClaudeAgentClient::new(config);
    /// let session = client.create_session().await?;
    /// let response = session.query_str("Analyze this data")
    ///     .max_tokens(8000)
    ///     .system_prompt("Be concise and analytical")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn query_str(&self, query: impl Into<String>) -> QueryBuilder<'_> {
        QueryBuilder::new(self, query.into())
    }

    /// Receive all messages from the session as a stream
    ///
    /// Returns a stream of parsed messages as they arrive from the CLI.
    /// This is useful for implementing streaming UIs or processing partial results.
    ///
    /// # Returns
    ///
    /// An async stream that yields `ParsedMessage` items or errors
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use turboclaudeagent::ClaudeAgentClient;
    /// # use futures::StreamExt;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = ClaudeAgentClient::builder().api_key("key").build()?;
    /// # let client = ClaudeAgentClient::new(config);
    /// let session = client.create_session().await?;
    ///
    /// // Start receiving messages in a separate task
    /// let stream = session.receive_messages().await;
    /// tokio::pin!(stream);
    /// while let Some(result) = stream.next().await {
    ///     match result {
    ///         Ok(msg) => println!("Received: {:?}", msg),
    ///         Err(e) => eprintln!("Error: {}", e),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn receive_messages(
        &self,
    ) -> impl futures::Stream<Item = Result<crate::message_parser::ParsedMessage, AgentError>> + '_
    {
        use crate::message_parser::parse_message;
        use futures::stream;
        use std::sync::Arc;

        let transport = Arc::clone(&self.transport);

        stream::unfold(transport, |transport| async move {
            match transport.recv_message().await {
                Ok(Some(json_value)) => {
                    // Parse the message using the message parser
                    match parse_message(json_value) {
                        Ok(parsed) => Some((Ok(parsed), transport)),
                        Err(e) => Some((
                            Err(AgentError::Protocol(format!("Message parse error: {}", e))),
                            transport,
                        )),
                    }
                }
                Ok(None) => {
                    // Transport closed
                    None
                }
                Err(e) => Some((
                    Err(AgentError::Transport(format!("Transport error: {}", e))),
                    transport,
                )),
            }
        })
    }
}

/// Builder for constructing and executing queries with chainable configuration
///
/// Created by [`AgentSession::query_str()`] and can be awaited directly or
/// configured with chainable methods before execution.
///
/// # Examples
///
/// ```no_run
/// # use turboclaudeagent::ClaudeAgentClient;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let config = ClaudeAgentClient::builder().api_key("key").build()?;
/// # let client = ClaudeAgentClient::new(config);
/// let session = client.create_session().await?;
///
/// // Simple: await directly
/// let response = session.query_str("Hello").await?;
///
/// // Advanced: chain configuration
/// let response = session.query_str("Analyze data")
///     .max_tokens(8000)
///     .system_prompt("You are a data analyst")
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct QueryBuilder<'a> {
    session: &'a AgentSession,
    query: String,
    system_prompt: Option<String>,
    model: Option<String>,
    max_tokens: Option<u32>,
    tools: Option<Vec<ToolDefinition>>,
    messages: Option<Vec<Message>>,
}

impl<'a> QueryBuilder<'a> {
    /// Create a new query builder
    pub(crate) fn new(session: &'a AgentSession, query: String) -> Self {
        Self {
            session,
            query,
            system_prompt: None,
            model: None,
            max_tokens: None,
            tools: None,
            messages: None,
        }
    }

    /// Set the maximum number of tokens in the response
    ///
    /// Default: 4096
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set a system prompt override
    ///
    /// This overrides the default system prompt for this query only.
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Set the model for this query
    ///
    /// Overrides the session's default model for this query only.
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the available tools for this query
    ///
    /// Default: empty (no tools)
    pub fn tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Set the message history for this query
    ///
    /// Default: empty (no history)
    pub fn messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = Some(messages);
        self
    }

    /// Execute the query (called automatically when awaited)
    ///
    /// You typically don't need to call this directly - just `.await` the builder.
    pub async fn send(self) -> AgentResult<QueryResponse> {
        // Get session state for defaults
        let state = self.session.state.lock().await;
        let default_model = state.current_model.clone();
        drop(state);

        // Inject skill context if skills feature is enabled
        #[cfg(feature = "skills")]
        let system_prompt = {
            let manager = self.session.skill_manager.read().await;
            if let Some(m) = manager.as_ref() {
                let skill_context = m.build_context().await;
                if !skill_context.is_empty() {
                    // Append skill context to system prompt
                    let current_prompt = self.system_prompt.unwrap_or_default();
                    Some(format!("{}{}", current_prompt, skill_context))
                } else {
                    self.system_prompt
                }
            } else {
                self.system_prompt
            }
        };

        #[cfg(not(feature = "skills"))]
        let system_prompt = self.system_prompt;

        // Increment usage counters for active skills
        #[cfg(feature = "skills")]
        {
            let manager = self.session.skill_manager.read().await;
            if let Some(m) = manager.as_ref() {
                m.increment_usage().await;
            }
        }

        // Build request with configured or default values
        let request = QueryRequest {
            query: self.query,
            system_prompt,
            model: self.model.unwrap_or(default_model),
            max_tokens: self.max_tokens.unwrap_or(4096),
            tools: self.tools.unwrap_or_default(),
            messages: self.messages.unwrap_or_default(),
        };

        // Execute via session
        self.session.query(request).await
    }
}

impl<'a> IntoFuture for QueryBuilder<'a> {
    type Output = AgentResult<QueryResponse>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.send())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::AtomicU32;

    #[test]
    fn test_concurrent_query_tracking() {
        let counter = Arc::new(AtomicU32::new(0));

        // Simulate concurrent queries
        let c1 = Arc::clone(&counter);
        let c2 = Arc::clone(&counter);
        let c3 = Arc::clone(&counter);

        let v1 = c1.fetch_add(1, Ordering::Relaxed);
        let v2 = c2.fetch_add(1, Ordering::Relaxed);
        let v3 = c3.fetch_add(1, Ordering::Relaxed);

        assert_eq!(v1, 0);
        assert_eq!(v2, 1);
        assert_eq!(v3, 2);

        // Cleanup
        c1.fetch_sub(1, Ordering::Relaxed);
        c2.fetch_sub(1, Ordering::Relaxed);
        c3.fetch_sub(1, Ordering::Relaxed);

        assert_eq!(counter.load(Ordering::Relaxed), 0);
    }
}
