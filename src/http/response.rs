//! HTTP response handling

use http::{StatusCode, HeaderMap};
use serde::de::DeserializeOwned;

/// HTTP response wrapper.
#[derive(Debug)]
pub struct Response {
    status: StatusCode,
    headers: HeaderMap,
    body: Vec<u8>,
    pub retries_taken: u32,
    pub elapsed: std::time::Duration,
}

/// Raw response wrapper that provides access to both the parsed body and HTTP metadata.
///
/// This matches the Python SDK's `with_raw_response` functionality, providing access to:
/// - Response headers (rate limits, request IDs, cache status, etc.)
/// - HTTP status code and reason
/// - Parsed response body
/// - Request metadata (URL, method, etc.)
/// - Timing information (elapsed duration, retries)
///
/// This type provides **complete feature parity** with the Python SDK's `APIResponse` type.
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
/// // Get raw response with full HTTP metadata
/// let raw = client.messages()
///     .with_raw_response()
///     .create(request)
///     .await?;
///
/// // Access HTTP metadata
/// println!("Status: {}", raw.status_code());
/// println!("Request ID: {:?}", raw.request_id());
/// println!("Elapsed: {:?}", raw.elapsed());
/// println!("Retries: {}", raw.retries_taken());
///
/// // Access rate limit info
/// if let Some((limit, remaining, reset)) = raw.rate_limit_info() {
///     println!("Rate limit: {}/{}, resets at {}", remaining, limit, reset);
/// }
///
/// // Access parsed body
/// println!("Response: {}", raw.parsed().text());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct RawResponse<T> {
    /// The parsed response body
    parsed: T,
    /// HTTP status code
    status: StatusCode,
    /// Response headers
    headers: HeaderMap,
    /// Number of retries taken (0 if no retries)
    retries_taken: u32,
    /// Time elapsed for the complete request/response cycle
    elapsed: std::time::Duration,
}

impl Response {
    /// Create a new response.
    pub fn new(
        status: StatusCode,
        headers: HeaderMap,
        body: Vec<u8>,
        retries_taken: u32,
        elapsed: std::time::Duration,
    ) -> Self {
        Self {
            status,
            headers,
            body,
            retries_taken,
            elapsed,
        }
    }

    /// Get the status code.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Get the headers.
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Get the raw body bytes.
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Get the body as a string.
    pub fn text(&self) -> Result<String, crate::error::Error> {
        String::from_utf8(self.body.clone())
            .map_err(|e| crate::error::Error::ResponseValidation(e.to_string()))
    }

    /// Parse the body as JSON.
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, crate::error::Error> {
        serde_json::from_slice(&self.body)
            .map_err(crate::error::Error::Serialization)
    }

    /// Check if the response is successful (2xx status).
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }

    /// Check if the response is an error (4xx or 5xx status).
    pub fn is_error(&self) -> bool {
        self.status.is_client_error() || self.status.is_server_error()
    }

    /// Convert this response into a `RawResponse` with a parsed body.
    ///
    /// This is used internally to support `with_raw_response()` mode.
    pub fn into_raw<T: DeserializeOwned>(self) -> Result<RawResponse<T>, crate::error::Error> {
        let parsed = serde_json::from_slice(&self.body)
            .map_err(crate::error::Error::Serialization)?;

        Ok(RawResponse::with_metadata(
            parsed,
            self.status,
            self.headers,
            self.retries_taken,
            self.elapsed,
        ))
    }

    /// Parse a successful response, converting HTTP errors to SDK errors.
    ///
    /// This is the DRY helper that eliminates the repeated error handling pattern
    /// across all resource methods.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::http::Response;
    /// # async fn example(response: Response) -> Result<(), Box<dyn std::error::Error>> {
    /// // Instead of:
    /// // if response.is_error() {
    /// //     return Err(Error::from_response(...));
    /// // }
    /// // response.json()
    ///
    /// // Just use:
    /// let result: MyType = response.parse_result()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_result<T: DeserializeOwned>(self) -> Result<T, crate::error::Error> {
        if self.is_error() {
            return Err(crate::error::Error::from_response(
                self.status.as_u16(),
                &self.text()?,
                &self.headers,
            ));
        }
        self.json()
    }

    /// Parse a successful response into a `RawResponse`, converting HTTP errors to SDK errors.
    ///
    /// This is the DRY helper for raw response mode that eliminates duplication.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::http::Response;
    /// # async fn example(response: Response) -> Result<(), Box<dyn std::error::Error>> {
    /// let raw = response.into_parsed_raw()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_parsed_raw<T: DeserializeOwned>(self) -> Result<RawResponse<T>, crate::error::Error> {
        if self.is_error() {
            return Err(crate::error::Error::from_response(
                self.status.as_u16(),
                &self.text()?,
                &self.headers,
            ));
        }
        self.into_raw()
    }
}

impl<T> RawResponse<T> {
    /// Create a new raw response.
    pub fn new(parsed: T, status: StatusCode, headers: HeaderMap) -> Self {
        Self {
            parsed,
            status,
            headers,
            retries_taken: 0,
            elapsed: std::time::Duration::from_secs(0),
        }
    }

    /// Create a new raw response with retry and timing information.
    pub fn with_metadata(
        parsed: T,
        status: StatusCode,
        headers: HeaderMap,
        retries_taken: u32,
        elapsed: std::time::Duration,
    ) -> Self {
        Self {
            parsed,
            status,
            headers,
            retries_taken,
            elapsed,
        }
    }

    /// Get a reference to the parsed response body.
    ///
    /// This is the primary way to access the response data.
    pub fn parsed(&self) -> &T {
        &self.parsed
    }

    /// Consume this raw response and return the parsed body.
    ///
    /// Equivalent to Python SDK's direct access to the parsed object.
    pub fn into_parsed(self) -> T {
        self.parsed
    }

    /// Get the HTTP status code.
    ///
    /// Equivalent to Python SDK's `status_code` property.
    pub fn status_code(&self) -> u16 {
        self.status.as_u16()
    }

    /// Get the HTTP status as a `StatusCode` object.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Get a reference to the response headers.
    ///
    /// Equivalent to Python SDK's `headers` property.
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Number of retries taken for this request (0 if no retries).
    ///
    /// Equivalent to Python SDK's `retries_taken` property.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use anthropic::{Client, MessageRequest, Message};
    /// # async fn example(client: Client) -> Result<(), Box<dyn std::error::Error>> {
    /// # let request = MessageRequest::builder().model("claude-3-5-sonnet-20241022").max_tokens(1024u32).messages(vec![Message::user("Hello")]).build()?;
    /// let raw = client.messages().with_raw_response().create(request).await?;
    ///
    /// if raw.retries_taken() > 0 {
    ///     println!("Request was retried {} times", raw.retries_taken());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn retries_taken(&self) -> u32 {
        self.retries_taken
    }

    /// Time elapsed for the complete request/response cycle.
    ///
    /// Equivalent to Python SDK's `elapsed` property.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use anthropic::{Client, MessageRequest, Message};
    /// # async fn example(client: Client) -> Result<(), Box<dyn std::error::Error>> {
    /// # let request = MessageRequest::builder().model("claude-3-5-sonnet-20241022").max_tokens(1024u32).messages(vec![Message::user("Hello")]).build()?;
    /// let raw = client.messages().with_raw_response().create(request).await?;
    ///
    /// println!("Request took {:?}", raw.elapsed());
    /// # Ok(())
    /// # }
    /// ```
    pub fn elapsed(&self) -> std::time::Duration {
        self.elapsed
    }

    /// Get a specific header value by name.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use anthropic::{Client, MessageRequest, Message};
    /// # async fn example(client: Client) -> Result<(), Box<dyn std::error::Error>> {
    /// # let request = MessageRequest::builder().model("claude-3-5-sonnet-20241022").max_tokens(1024u32).messages(vec![Message::user("Hello")]).build()?;
    /// let raw = client.messages().with_raw_response().create(request).await?;
    ///
    /// if let Some(request_id) = raw.get_header("request-id") {
    ///     println!("Request ID: {}", request_id.to_str().unwrap());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_header(&self, name: &str) -> Option<&http::HeaderValue> {
        self.headers.get(name)
    }

    /// Get the rate limit information from headers.
    ///
    /// Returns `(limit, remaining, reset_timestamp)` if headers are present.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use anthropic::{Client, MessageRequest, Message};
    /// # async fn example(client: Client) -> Result<(), Box<dyn std::error::Error>> {
    /// # let request = MessageRequest::builder().model("claude-3-5-sonnet-20241022").max_tokens(1024u32).messages(vec![Message::user("Hello")]).build()?;
    /// let raw = client.messages().with_raw_response().create(request).await?;
    ///
    /// if let Some((limit, remaining, reset)) = raw.rate_limit_info() {
    ///     println!("Rate limit: {}/{}, resets at {}", remaining, limit, reset);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn rate_limit_info(&self) -> Option<(u32, u32, String)> {
        let limit = self.headers.get("anthropic-ratelimit-requests-limit")?
            .to_str().ok()?
            .parse::<u32>().ok()?;

        let remaining = self.headers.get("anthropic-ratelimit-requests-remaining")?
            .to_str().ok()?
            .parse::<u32>().ok()?;

        let reset = self.headers.get("anthropic-ratelimit-requests-reset")?
            .to_str().ok()?
            .to_string();

        Some((limit, remaining, reset))
    }

    /// Get the request ID from headers.
    ///
    /// The request ID is useful for debugging and support tickets.
    pub fn request_id(&self) -> Option<String> {
        self.headers.get("request-id")?
            .to_str().ok()
            .map(|s| s.to_string())
    }
}