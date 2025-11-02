//! Messages API endpoint

use super::Resource;
use crate::{
    client::Client,
    error::Result,
    http::RawResponse,
    streaming::MessageStream,
    types::{Message, MessageRequest},
};
use std::sync::OnceLock;
use tracing::{debug, info, warn};

/// Messages API resource.
///
/// This is the main API for creating messages with Claude models.
#[derive(Clone)]
pub struct Messages {
    client: Client,
    batches: OnceLock<Batches>,
}

impl Messages {
    /// Create a new Messages resource.
    pub(crate) fn new(client: Client) -> Self {
        Self {
            client,
            batches: OnceLock::new(),
        }
    }

    /// Create a new message.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::{Client, MessageRequest, Message};
    /// # async fn example(client: Client) -> Result<(), Box<dyn std::error::Error>> {
    /// let request = MessageRequest::builder()
    ///     .model("claude-3-5-sonnet-20241022")
    ///     .max_tokens(1024u32)
    ///     .messages(vec![
    ///         Message::user("Hello, Claude!")
    ///     ])
    ///     .build()?;
    ///
    /// let message = client.messages().create(request).await?;
    /// println!("{}", message.text());
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(skip(self, request), fields(model = %request.model, max_tokens = request.max_tokens, message_count = request.messages.len()))]
    pub async fn create(&self, request: MessageRequest) -> Result<Message> {
        debug!("Creating message with {} messages", request.messages.len());

        // Validate request before sending
        if let Err(e) = crate::validation::validate_message_request(&request) {
            warn!("Request validation failed: {}", e);
            return Err(e);
        }

        debug!("Sending message request to API");
        let start = std::time::Instant::now();

        let result: Result<Message> = self
            .client
            .request(http::Method::POST, "/v1/messages")?
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?
            .parse_result();

        let elapsed = start.elapsed();
        match &result {
            Ok(message) => {
                info!(
                    elapsed_ms = elapsed.as_millis(),
                    stop_reason = ?message.stop_reason,
                    input_tokens = message.usage.input_tokens,
                    output_tokens = message.usage.output_tokens,
                    "Message created successfully"
                );
            }
            Err(e) => {
                warn!(elapsed_ms = elapsed.as_millis(), error = %e, "Message creation failed");
            }
        }

        result
    }

    /// Create a streaming message.
    ///
    /// Returns a stream of events as the message is generated.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::{Client, MessageRequest, Message};
    /// # use turboclaude::streaming::StreamEvent;
    /// # use futures::StreamExt;
    /// # async fn example(client: Client) -> Result<(), Box<dyn std::error::Error>> {
    /// let request = MessageRequest::builder()
    ///     .model("claude-3-5-sonnet-20241022")
    ///     .max_tokens(1024u32)
    ///     .messages(vec![
    ///         Message::user("Tell me a story")
    ///     ])
    ///     .stream(true)
    ///     .build()?;
    ///
    /// let mut stream = client.messages().stream(request).await?;
    ///
    /// while let Some(event) = stream.next().await {
    ///     match event? {
    ///         StreamEvent::ContentBlockDelta(event) => {
    ///             if let Some(text) = event.delta.text {
    ///                 print!("{}", text);
    ///             }
    ///         }
    ///         StreamEvent::MessageStop => break,
    ///         _ => {}
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(skip(self, request), fields(model = %request.model, max_tokens = request.max_tokens, message_count = request.messages.len()))]
    pub async fn stream(&self, mut request: MessageRequest) -> Result<MessageStream> {
        debug!(
            "Creating streaming message with {} messages",
            request.messages.len()
        );

        // Validate request before sending
        if let Err(e) = crate::validation::validate_message_request(&request) {
            warn!("Stream request validation failed: {}", e);
            return Err(e);
        }

        // Ensure streaming is enabled
        request.stream = Some(true);
        debug!("Opening stream for message");

        let result = self
            .client
            .request(http::Method::POST, "/v1/messages")?
            .body(serde_json::to_vec(&request)?)
            .send_streaming()
            .await
            .map(MessageStream::new);

        match &result {
            Ok(_) => {
                info!("Streaming message started successfully");
            }
            Err(e) => {
                warn!(error = %e, "Failed to start streaming message");
            }
        }

        result
    }

    /// Count tokens in a message request.
    ///
    /// This endpoint allows you to count tokens before sending a request,
    /// including tools, images, and documents.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::{Client, MessageRequest, Message};
    /// # async fn example(client: Client) -> Result<(), Box<dyn std::error::Error>> {
    /// let request = MessageRequest::builder()
    ///     .model("claude-3-5-sonnet-20241022")
    ///     .max_tokens(1024u32)
    ///     .messages(vec![Message::user("Hello, Claude!")])
    ///     .build()?;
    ///
    /// let token_count = client.messages().count_tokens(request).await?;
    /// println!("Input tokens: {}", token_count.input_tokens);
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(skip(self, request), fields(model = %request.model))]
    pub async fn count_tokens(&self, request: MessageRequest) -> Result<TokenCount> {
        debug!("Counting tokens for request");

        let result: Result<TokenCount> = self
            .client
            .request(http::Method::POST, "/v1/messages/count_tokens")?
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?
            .parse_result();

        match &result {
            Ok(count) => {
                debug!(input_tokens = count.input_tokens, "Token count retrieved");
            }
            Err(e) => {
                warn!(error = %e, "Failed to count tokens");
            }
        }

        result
    }

    /// Get the batches sub-resource for batch processing.
    ///
    /// This method uses lazy initialization with `OnceLock` for zero-allocation
    /// access after the first call. All subsequent calls return a reference to
    /// the same `Batches` instance.
    pub fn batches(&self) -> &Batches {
        self.batches
            .get_or_init(|| Batches::new(self.client.clone()))
    }

    /// Enable raw response mode for the next request.
    ///
    /// Returns a wrapper that provides access to response headers,
    /// status codes, and other HTTP metadata along with the parsed body.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::{Client, MessageRequest, Message};
    /// # async fn example(client: Client) -> Result<(), Box<dyn std::error::Error>> {
    /// let request = MessageRequest::builder()
    ///     .model("claude-3-5-sonnet-20241022")
    ///     .max_tokens(1024u32)
    ///     .messages(vec![Message::user("Hello")])
    ///     .build()?;
    ///
    /// // Get raw response with headers
    /// let raw = client.messages()
    ///     .with_raw_response()
    ///     .create(request)
    ///     .await?;
    ///
    /// // Access headers
    /// if let Some(request_id) = raw.request_id() {
    ///     println!("Request ID: {}", request_id);
    /// }
    ///
    /// // Access rate limit info
    /// if let Some((limit, remaining, reset)) = raw.rate_limit_info() {
    ///     println!("Rate limit: {}/{}, resets at {}", remaining, limit, reset);
    /// }
    ///
    /// // Access parsed response
    /// println!("Response: {}", raw.parsed().text());
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_raw_response(&self) -> MessagesRaw {
        MessagesRaw {
            client: self.client.clone(),
            batches: OnceLock::new(),
        }
    }
}

impl Resource for Messages {
    fn client(&self) -> &Client {
        &self.client
    }
}

/// Messages resource in raw response mode.
///
/// This wrapper provides the same methods as `Messages`, but returns
/// `RawResponse<T>` instead of `T`, giving access to HTTP headers and metadata.
#[derive(Clone)]
pub struct MessagesRaw {
    client: Client,
    batches: OnceLock<BatchesRaw>,
}

impl MessagesRaw {
    /// Create a new message and return the raw response with headers.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::{Client, MessageRequest, Message};
    /// # async fn example(client: Client) -> Result<(), Box<dyn std::error::Error>> {
    /// let request = MessageRequest::builder()
    ///     .model("claude-3-5-sonnet-20241022")
    ///     .max_tokens(1024u32)
    ///     .messages(vec![Message::user("Hello")])
    ///     .build()?;
    ///
    /// let raw = client.messages().with_raw_response().create(request).await?;
    ///
    /// // Check rate limits
    /// if let Some((limit, remaining, _)) = raw.rate_limit_info() {
    ///     if remaining < 10 {
    ///         println!("Warning: Only {} requests remaining!", remaining);
    ///     }
    /// }
    ///
    /// let message = raw.into_parsed();
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(&self, request: MessageRequest) -> Result<RawResponse<Message>> {
        let response = self
            .client
            .request(http::Method::POST, "/v1/messages")?
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?;

        response.into_parsed_raw()
    }

    /// Count tokens and return the raw response with headers.
    pub async fn count_tokens(&self, request: MessageRequest) -> Result<RawResponse<TokenCount>> {
        let response = self
            .client
            .request(http::Method::POST, "/v1/messages/count_tokens")?
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?;

        response.into_parsed_raw()
    }

    /// Get the batches sub-resource in raw response mode.
    ///
    /// This method uses lazy initialization with `OnceLock` for zero-allocation
    /// access after the first call.
    pub fn batches(&self) -> &BatchesRaw {
        self.batches
            .get_or_init(|| BatchesRaw::new(self.client.clone()))
    }
}

/// Token count response.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TokenCount {
    /// Number of input tokens
    pub input_tokens: u32,
}

/// Batch processing for messages.
#[derive(Clone)]
pub struct Batches {
    client: Client,
}

impl Batches {
    /// Create a new Batches resource.
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Create a new batch of message requests.
    ///
    /// Send a batch of Message creation requests. Once created, the batch begins
    /// processing immediately.
    pub async fn create(&self, requests: Vec<BatchRequest>) -> Result<MessageBatch> {
        #[derive(serde::Serialize)]
        struct BatchCreateBody {
            requests: Vec<BatchRequest>,
        }

        let response = self
            .client
            .request(http::Method::POST, "/v1/messages/batches")?
            .body(serde_json::to_vec(&BatchCreateBody { requests })?)
            .send()
            .await?;

        response.parse_result()
    }

    /// List all batches.
    ///
    /// Returns the most recently created batches first.
    pub async fn list(&self) -> Result<Vec<MessageBatch>> {
        #[derive(serde::Deserialize)]
        struct BatchList {
            data: Vec<MessageBatch>,
        }

        let list: BatchList = self
            .client
            .request(http::Method::GET, "/v1/messages/batches")?
            .send()
            .await?
            .parse_result()?;

        Ok(list.data)
    }

    /// Get a specific batch by ID.
    ///
    /// This endpoint is idempotent and can be used to poll for batch completion.
    pub async fn get(&self, batch_id: &str) -> Result<MessageBatch> {
        let response = self
            .client
            .request(
                http::Method::GET,
                &format!("/v1/messages/batches/{}", batch_id),
            )?
            .send()
            .await?;

        response.parse_result()
    }

    /// Cancel a batch.
    ///
    /// Batches may be canceled any time before processing ends. Once cancellation
    /// is initiated, the batch enters a `canceling` state.
    pub async fn cancel(&self, batch_id: &str) -> Result<MessageBatch> {
        let response = self
            .client
            .request(
                http::Method::POST,
                &format!("/v1/messages/batches/{}/cancel", batch_id),
            )?
            .send()
            .await?;

        response.parse_result()
    }

    /// Get results for a completed batch.
    ///
    /// Streams the results of a Message Batch as JSONL. Each line is a JSON object
    /// containing the result of a single request in the batch.
    pub async fn results(&self, batch_id: &str) -> Result<Vec<BatchResult>> {
        // First get the batch to find the results_url
        let batch = self.get(batch_id).await?;

        let results_url = batch.results_url.ok_or_else(|| {
            crate::error::Error::InvalidRequest("Batch does not have results_url yet".to_string())
        })?;

        // Fetch the results from the URL
        let response = reqwest::get(&results_url)
            .await
            .map_err(|e| crate::error::Error::Connection(e.to_string()))?;

        if !response.status().is_success() {
            return Err(crate::error::Error::ApiError {
                status: response.status().as_u16(),
                message: "Failed to fetch batch results".to_string(),
                error_type: None,
                request_id: None,
            });
        }

        let text = response
            .text()
            .await
            .map_err(|e| crate::error::Error::Connection(e.to_string()))?;

        // Parse JSONL (one JSON object per line)
        let results: Result<Vec<BatchResult>> = text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                serde_json::from_str(line).map_err(|e| {
                    crate::error::Error::ResponseValidation(format!(
                        "Failed to parse batch result: {}",
                        e
                    ))
                })
            })
            .collect();

        results
    }
}

/// Request for batch processing.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BatchRequest {
    /// Custom ID for this request
    pub custom_id: String,

    /// The message request parameters
    pub params: MessageRequest,
}

/// Result from batch processing.
#[derive(Debug, serde::Deserialize)]
pub struct BatchResult {
    /// Custom ID from the request
    pub custom_id: String,

    /// Result of the request
    pub result: BatchResultType,
}

/// Type of batch result.
#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type")]
pub enum BatchResultType {
    /// Successful message generation
    #[serde(rename = "succeeded")]
    Success {
        /// The generated message
        message: Message,
    },

    /// Error during processing
    #[serde(rename = "errored")]
    Error {
        /// Error details
        error: BatchError,
    },
}

/// Error in batch processing.
#[derive(Debug, serde::Deserialize)]
pub struct BatchError {
    /// Error type
    #[serde(rename = "type")]
    pub error_type: String,

    /// Error message
    pub message: String,
}

use crate::types::batch::MessageBatch;

/// Batches resource in raw response mode.
#[derive(Clone)]
pub struct BatchesRaw {
    client: Client,
}

impl BatchesRaw {
    /// Create a new BatchesRaw resource.
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Create a new batch and return raw response with headers.
    pub async fn create(&self, requests: Vec<BatchRequest>) -> Result<RawResponse<MessageBatch>> {
        #[derive(serde::Serialize)]
        struct BatchCreateBody {
            requests: Vec<BatchRequest>,
        }

        let response = self
            .client
            .request(http::Method::POST, "/v1/messages/batches")?
            .body(serde_json::to_vec(&BatchCreateBody { requests })?)
            .send()
            .await?;

        response.into_parsed_raw()
    }

    /// Get a specific batch by ID and return raw response with headers.
    pub async fn get(&self, batch_id: &str) -> Result<RawResponse<MessageBatch>> {
        let response = self
            .client
            .request(
                http::Method::GET,
                &format!("/v1/messages/batches/{}", batch_id),
            )?
            .send()
            .await?;

        response.into_parsed_raw()
    }

    /// Cancel a batch and return raw response with headers.
    pub async fn cancel(&self, batch_id: &str) -> Result<RawResponse<MessageBatch>> {
        let response = self
            .client
            .request(
                http::Method::POST,
                &format!("/v1/messages/batches/{}/cancel", batch_id),
            )?
            .send()
            .await?;

        response.into_parsed_raw()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Models, Role};

    #[test]
    fn test_messages_resource_creation() {
        let client = Client::new("test-api-key");
        let messages = client.messages();

        // Verify resource is created (client is cloned, so we can't use ptr::eq)
        // Just verify the resource has a client reference
        let _ = messages.client();
    }

    #[test]
    fn test_messages_create_request() {
        let request = MessageRequest::builder()
            .model(Models::CLAUDE_3_5_SONNET)
            .max_tokens(1024u32)
            .messages(vec![Message::user("Hello")])
            .build()
            .unwrap();

        assert_eq!(request.model, Models::CLAUDE_3_5_SONNET);
        assert_eq!(request.max_tokens, 1024);
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.messages[0].role, Role::User);
    }

    #[test]
    fn test_messages_create_with_system_prompt() {
        let request = MessageRequest::builder()
            .model(Models::CLAUDE_3_5_SONNET)
            .max_tokens(1024u32)
            .messages(vec![Message::user("Hello")])
            .system("You are a helpful assistant")
            .build()
            .unwrap();

        assert!(request.system.is_some());

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["system"], "You are a helpful assistant");
    }

    #[test]
    fn test_messages_create_with_tools() {
        use crate::types::Tool;
        use serde_json::json;

        let tool = Tool {
            name: "get_weather".to_string(),
            description: "Get weather for a location".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "location": {"type": "string"}
                }
            }),
        };

        let request = MessageRequest::builder()
            .model(Models::CLAUDE_3_5_SONNET)
            .max_tokens(1024u32)
            .messages(vec![Message::user("What's the weather?")])
            .tools(vec![tool.clone()])
            .build()
            .unwrap();

        assert!(request.tools.is_some());
        assert_eq!(request.tools.as_ref().unwrap().len(), 1);
        assert_eq!(request.tools.as_ref().unwrap()[0].name, "get_weather");

        let json = serde_json::to_value(&request).unwrap();
        assert!(json["tools"].is_array());
        assert_eq!(json["tools"][0]["name"], "get_weather");
    }

    #[test]
    fn test_messages_stream_request() {
        let mut request = MessageRequest::builder()
            .model(Models::CLAUDE_3_5_SONNET)
            .max_tokens(1024u32)
            .messages(vec![Message::user("Tell me a story")])
            .build()
            .unwrap();

        // Initially no stream flag
        assert!(request.stream.is_none());

        // Set stream flag
        request.stream = Some(true);
        assert_eq!(request.stream, Some(true));

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["stream"], true);
    }

    #[test]
    fn test_batch_request_creation() {
        let batch_request = BatchRequest {
            custom_id: "req-001".to_string(),
            params: MessageRequest::builder()
                .model(Models::CLAUDE_3_5_SONNET)
                .max_tokens(512u32)
                .messages(vec![Message::user("Hello")])
                .build()
                .unwrap(),
        };

        assert_eq!(batch_request.custom_id, "req-001");
        assert_eq!(batch_request.params.max_tokens, 512);

        let json = serde_json::to_value(&batch_request).unwrap();
        assert_eq!(json["custom_id"], "req-001");
        assert_eq!(json["params"]["max_tokens"], 512);
    }

    #[test]
    fn test_token_count_serialization() {
        let token_count = TokenCount { input_tokens: 150 };

        assert_eq!(token_count.input_tokens, 150);

        let json = serde_json::to_value(&token_count).unwrap();
        assert_eq!(json["input_tokens"], 150);
    }

    #[test]
    fn test_batch_error_deserialization() {
        let json = r#"{
            "type": "rate_limit_error",
            "message": "Rate limit exceeded"
        }"#;

        let error: BatchError = serde_json::from_str(json).unwrap();
        assert_eq!(error.error_type, "rate_limit_error");
        assert_eq!(error.message, "Rate limit exceeded");
    }

    #[test]
    fn test_batch_result_success() {
        use crate::types::{ContentBlock, Usage};

        let message = Message {
            id: "msg_123".to_string(),
            message_type: "message".to_string(),
            role: Role::Assistant,
            content: vec![ContentBlock::Text {
                text: "Hello!".to_string(),
                citations: None,
            }],
            model: Models::CLAUDE_3_5_SONNET.to_string(),
            stop_reason: Some(crate::types::StopReason::EndTurn),
            stop_sequence: None,
            usage: Usage {
                input_tokens: 10,
                output_tokens: 5,
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
            },
        };

        let result_json = serde_json::json!({
            "custom_id": "req-001",
            "result": {
                "type": "succeeded",
                "message": message
            }
        });

        let batch_result: BatchResult = serde_json::from_value(result_json).unwrap();
        assert_eq!(batch_result.custom_id, "req-001");

        match batch_result.result {
            BatchResultType::Success { message } => {
                assert_eq!(message.id, "msg_123");
            }
            _ => panic!("Expected success result"),
        }
    }

    #[test]
    fn test_messages_with_raw_response() {
        let client = Client::new("test-api-key");
        let _messages_raw = client.messages().with_raw_response();

        // Verify we can create raw response wrapper
        // (actual HTTP calls tested in integration tests)
    }

    #[test]
    fn test_batches_lazy_initialization() {
        let client = Client::new("test-api-key");
        let messages = client.messages();

        // Get batches twice - should return same instance
        let batches1 = messages.batches();
        let batches2 = messages.batches();

        // Verify pointer equality - both references point to same instance
        assert!(
            std::ptr::eq(batches1, batches2),
            "Multiple calls to batches() should return the same instance"
        );
    }

    #[test]
    fn test_messages_raw_batches_lazy_initialization() {
        let client = Client::new("test-api-key");
        let messages_raw = client.messages().with_raw_response();

        // Get batches twice - should return same instance
        let batches1 = messages_raw.batches();
        let batches2 = messages_raw.batches();

        // Verify pointer equality
        assert!(
            std::ptr::eq(batches1, batches2),
            "Multiple calls to batches() should return the same BatchesRaw instance"
        );
    }
}
