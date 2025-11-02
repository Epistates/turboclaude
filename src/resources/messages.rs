//! Messages API endpoint

use crate::{
    client::Client,
    error::Result,
    http::RawResponse,
    types::{Message, MessageRequest},
    streaming::MessageStream,
};

/// Messages API resource.
///
/// This is the main API for creating messages with Claude models.
#[derive(Clone)]
pub struct Messages {
    client: Client,
}

impl Messages {
    /// Create a new Messages resource.
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Create a new message.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use anthropic::{Client, MessageRequest, Message};
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
    pub async fn create(&self, request: MessageRequest) -> Result<Message> {
        // Apply rate limiting if configured
        self.client.apply_rate_limit().await;

        self.client
            .request(http::Method::POST, "/v1/messages")
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?
            .parse_result()
    }

    /// Create a streaming message.
    ///
    /// Returns a stream of events as the message is generated.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use anthropic::{Client, MessageRequest, Message};
    /// # use anthropic::streaming::StreamEvent;
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
    pub async fn stream(&self, mut request: MessageRequest) -> Result<MessageStream> {
        // Apply rate limiting if configured
        self.client.apply_rate_limit().await;

        // Ensure streaming is enabled
        request.stream = Some(true);

        let byte_stream = self.client
            .request(http::Method::POST, "/v1/messages")
            .body(serde_json::to_vec(&request)?)
            .send_streaming()
            .await?;

        Ok(MessageStream::new(byte_stream))
    }

    /// Count tokens in a message request.
    ///
    /// This endpoint allows you to count tokens before sending a request,
    /// including tools, images, and documents.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use anthropic::{Client, MessageRequest, Message};
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
    pub async fn count_tokens(&self, request: MessageRequest) -> Result<TokenCount> {
        // Apply rate limiting if configured
        self.client.apply_rate_limit().await;

        let response = self.client
            .request(http::Method::POST, "/v1/messages/count_tokens")
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?;

        response.parse_result()
    }

    /// Get the batches sub-resource for batch processing.
    pub fn batches(&self) -> Batches {
        Batches::new(self.client.clone())
    }

    /// Enable raw response mode for the next request.
    ///
    /// Returns a wrapper that provides access to response headers,
    /// status codes, and other HTTP metadata along with the parsed body.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use anthropic::{Client, MessageRequest, Message};
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
        }
    }
}

/// Messages resource in raw response mode.
///
/// This wrapper provides the same methods as `Messages`, but returns
/// `RawResponse<T>` instead of `T`, giving access to HTTP headers and metadata.
#[derive(Clone)]
pub struct MessagesRaw {
    client: Client,
}

impl MessagesRaw {
    /// Create a new message and return the raw response with headers.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use anthropic::{Client, MessageRequest, Message};
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
        let response = self.client
            .request(http::Method::POST, "/v1/messages")
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?;

        response.into_parsed_raw()
    }

    /// Count tokens and return the raw response with headers.
    pub async fn count_tokens(&self, request: MessageRequest) -> Result<RawResponse<TokenCount>> {
        let response = self.client
            .request(http::Method::POST, "/v1/messages/count_tokens")
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?;

        response.into_parsed_raw()
    }

    /// Get the batches sub-resource in raw response mode.
    pub fn batches(&self) -> BatchesRaw {
        BatchesRaw::new(self.client.clone())
    }
}

/// Token count response.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TokenCount {
    /// Number of input tokens
    pub input_tokens: u32,
}

/// Batch processing for messages.
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

        let response = self.client
            .request(http::Method::POST, "/v1/messages/batches")
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

        let list: BatchList = self.client
            .request(http::Method::GET, "/v1/messages/batches")
            .send()
            .await?
            .parse_result()?;

        Ok(list.data)
    }

    /// Get a specific batch by ID.
    ///
    /// This endpoint is idempotent and can be used to poll for batch completion.
    pub async fn get(&self, batch_id: &str) -> Result<MessageBatch> {
        let response = self.client
            .request(http::Method::GET, &format!("/v1/messages/batches/{}", batch_id))
            .send()
            .await?;

        response.parse_result()
    }

    /// Cancel a batch.
    ///
    /// Batches may be canceled any time before processing ends. Once cancellation
    /// is initiated, the batch enters a `canceling` state.
    pub async fn cancel(&self, batch_id: &str) -> Result<MessageBatch> {
        let response = self.client
            .request(http::Method::POST, &format!("/v1/messages/batches/{}/cancel", batch_id))
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

        let results_url = batch.results_url
            .ok_or_else(|| crate::error::Error::InvalidRequest(
                "Batch does not have results_url yet".to_string()
            ))?;

        // Fetch the results from the URL
        let response = reqwest::get(&results_url).await
            .map_err(|e| crate::error::Error::Connection(e.to_string()))?;

        if !response.status().is_success() {
            return Err(crate::error::Error::ApiError {
                status: response.status().as_u16(),
                message: "Failed to fetch batch results".to_string(),
                error_type: None,
                request_id: None,
            });
        }

        let text = response.text().await
            .map_err(|e| crate::error::Error::Connection(e.to_string()))?;

        // Parse JSONL (one JSON object per line)
        let results: Result<Vec<BatchResult>> = text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                serde_json::from_str(line)
                    .map_err(|e| crate::error::Error::ResponseValidation(
                        format!("Failed to parse batch result: {}", e)
                    ))
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
        message: Message
    },

    /// Error during processing
    #[serde(rename = "errored")]
    Error {
        /// Error details
        error: BatchError
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

        let response = self.client
            .request(http::Method::POST, "/v1/messages/batches")
            .body(serde_json::to_vec(&BatchCreateBody { requests })?)
            .send()
            .await?;

        response.into_parsed_raw()
    }

    /// Get a specific batch by ID and return raw response with headers.
    pub async fn get(&self, batch_id: &str) -> Result<RawResponse<MessageBatch>> {
        let response = self.client
            .request(http::Method::GET, &format!("/v1/messages/batches/{}", batch_id))
            .send()
            .await?;

        response.into_parsed_raw()
    }

    /// Cancel a batch and return raw response with headers.
    pub async fn cancel(&self, batch_id: &str) -> Result<RawResponse<MessageBatch>> {
        let response = self.client
            .request(http::Method::POST, &format!("/v1/messages/batches/{}/cancel", batch_id))
            .send()
            .await?;

        response.into_parsed_raw()
    }
}