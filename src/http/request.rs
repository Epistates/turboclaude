//! HTTP request builder

use std::time::Duration;
use url::Url;
use http::{Method, HeaderMap, HeaderName, HeaderValue};
use futures::StreamExt;
use crate::error::Result;
use super::{Response, retry::{RetryConfig, calculate_retry_delay}};

/// Builder for HTTP requests.
///
/// Provides a fluent API for constructing and sending HTTP requests with automatic retry logic,
/// configurable timeouts, and custom headers. Integrates the retry module for exponential backoff
/// and intelligent retry decision-making.
#[derive(Debug, Clone)]
pub struct RequestBuilder {
    method: Method,
    url: Url,
    headers: HeaderMap,
    body: Option<Vec<u8>>,
    timeout: Duration,
    pub(crate) max_retries: u32,
    pub(crate) retry_config: RetryConfig,
    pub(crate) http_client: Option<reqwest::Client>,
}

impl RequestBuilder {
    /// Create a new request builder.
    ///
    /// Uses default retry configuration with exponential backoff (1s, 2s, 4s, etc.)
    /// and a 10-minute timeout.
    pub fn new(method: Method, url: Url) -> Self {
        Self {
            method,
            url,
            headers: HeaderMap::new(),
            body: None,
            timeout: Duration::from_secs(600),
            max_retries: 2,
            retry_config: RetryConfig::default(),
            http_client: None,
        }
    }

    /// Set the HTTP client to use
    pub(crate) fn with_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = Some(client);
        self
    }

    /// Set a header.
    ///
    /// # Panics
    ///
    /// Panics if the header name or value is invalid according to HTTP specifications.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let key = key.into().parse::<HeaderName>()
            .expect("invalid HTTP header name: header names must be valid HTTP identifiers");
        let value = value.into().parse::<HeaderValue>()
            .expect("invalid HTTP header value: header values must be valid ASCII strings");
        self.headers.insert(key, value);
        self
    }

    /// Set custom retry configuration.
    ///
    /// Override the default retry behavior with custom backoff parameters.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use anthropic::http::retry::RetryConfig;
    /// use std::time::Duration;
    ///
    /// let custom_retry = RetryConfig {
    ///     max_retries: 5,
    ///     initial_interval: Duration::from_millis(500),
    ///     max_interval: Duration::from_secs(30),
    ///     multiplier: 2.0,
    ///     randomization_factor: 0.1,
    /// };
    ///
    /// // builder.retry_config(custom_retry);
    /// ```
    pub fn retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Set the request body.
    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    /// Set the request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set max retries.
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Send the request and get a response.
    ///
    /// Sends the HTTP request with automatic retry logic based on the configured
    /// `RetryConfig`. Retryable errors (429, 5xx, timeouts, etc.) are automatically
    /// retried with exponential backoff, respecting the `Retry-After` header if present.
    pub async fn send(self) -> Result<Response> {
        let client = self.http_client.ok_or_else(|| {
            crate::error::Error::HttpClient("No HTTP client configured".to_string())
        })?;

        // Build reqwest request
        let mut req = client.request(self.method.clone(), self.url.as_str())
            .timeout(self.timeout);

        // Add headers
        for (key, value) in &self.headers {
            req = req.header(key, value);
        }

        // Add body if present
        if let Some(body) = self.body {
            req = req.body(body);
        }

        // Send request with retry logic using configured RetryConfig
        let mut attempt = 0;
        let start_time = std::time::Instant::now();
        loop {
            match req.try_clone()
                .ok_or_else(|| crate::error::Error::HttpClient("could not clone request for retry".to_string()))?
                .send()
                .await
            {
                Ok(resp) => {
                    let status = resp.status();
                    let headers = resp.headers().clone();
                    let body = resp.bytes().await
                        .map_err(|e| crate::error::Error::Connection(e.to_string()))?
                        .to_vec();

                    let response = Response::new(status, headers, body, attempt, start_time.elapsed());

                    // Check if we should retry using the retry module
                    if response.is_error() && attempt < self.max_retries {
                        let error = crate::error::Error::from_response(
                            status.as_u16(),
                            &String::from_utf8_lossy(response.body()),
                            response.headers(),
                        );

                        if error.is_retryable() {
                            if let Some(delay) = calculate_retry_delay(&error, attempt, &self.retry_config) {
                                tokio::time::sleep(delay).await;
                                attempt += 1;
                                continue;
                            }
                        }
                    }

                    return Ok(response);
                }
                Err(e) if e.is_timeout() => {
                    if attempt >= self.max_retries {
                        return Err(crate::error::Error::Timeout(self.timeout));
                    }
                    let timeout_error = crate::error::Error::Timeout(self.timeout);
                    if let Some(delay) = calculate_retry_delay(&timeout_error, attempt, &self.retry_config) {
                        tokio::time::sleep(delay).await;
                        attempt += 1;
                    } else {
                        return Err(timeout_error);
                    }
                }
                Err(e) => {
                    return Err(crate::error::Error::Connection(e.to_string()));
                }
            }
        }
    }

    /// Send a streaming request
    pub async fn send_streaming(self) -> Result<impl futures::Stream<Item = Result<bytes::Bytes>>> {
        let client = self.http_client.ok_or_else(|| {
            crate::error::Error::HttpClient("No HTTP client configured".to_string())
        })?;

        let mut req = client.request(self.method.clone(), self.url.as_str())
            .timeout(self.timeout);

        for (key, value) in &self.headers {
            req = req.header(key, value);
        }

        if let Some(body) = self.body {
            req = req.body(body);
        }

        let resp = req.send().await
            .map_err(|e| crate::error::Error::Connection(e.to_string()))?;

        Ok(resp.bytes_stream().map(|result| {
            result.map_err(|e| crate::error::Error::Streaming(e.to_string()))
        }))
    }

    /// Get the method.
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Get the URL.
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Get the headers.
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Get the timeout.
    pub fn timeout_duration(&self) -> Duration {
        self.timeout
    }
}