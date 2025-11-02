//! HTTP request builder

use super::Response;
use crate::error::Result;
use futures::StreamExt;
use http::{HeaderMap, HeaderName, HeaderValue, Method};
use std::time::Duration;
use url::Url;

/// Builder for HTTP requests.
#[derive(Debug, Clone)]
pub struct RequestBuilder {
    method: Method,
    url: Url,
    headers: HeaderMap,
    body: Option<Vec<u8>>,
    timeout: Duration,
    pub(crate) max_retries: u32,
    pub(crate) http_client: Option<reqwest::Client>,
}

impl RequestBuilder {
    /// Create a new request builder.
    pub fn new(method: Method, url: Url) -> Self {
        Self {
            method,
            url,
            headers: HeaderMap::new(),
            body: None,
            timeout: Duration::from_secs(600),
            max_retries: 2,
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
    /// Panics if the header name or value contains invalid characters.
    /// For fallible header setting, use [`try_header`](Self::try_header) instead.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let key_str = key.into();
        let value_str = value.into();

        // Try to parse, but provide better error context
        let key = key_str
            .parse::<HeaderName>()
            .unwrap_or_else(|e| panic!("Invalid header name '{}': {}", key_str, e));
        let value = value_str
            .parse::<HeaderValue>()
            .unwrap_or_else(|e| panic!("Invalid header value '{}': {}", value_str, e));

        self.headers.insert(key, value);
        self
    }

    /// Try to set a header, returning an error if the name or value is invalid.
    ///
    /// This is the fallible version of [`header`](Self::header).
    ///
    /// # Errors
    /// Returns an error if the header name or value contains invalid characters.
    pub fn try_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Result<Self> {
        let key_str = key.into();
        let value_str = value.into();

        let key = key_str.parse::<HeaderName>().map_err(|e| {
            crate::error::Error::HttpClient(format!("Invalid header name '{}': {}", key_str, e))
        })?;
        let value = value_str.parse::<HeaderValue>().map_err(|e| {
            crate::error::Error::HttpClient(format!("Invalid header value '{}': {}", value_str, e))
        })?;

        self.headers.insert(key, value);
        Ok(self)
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
    pub async fn send(self) -> Result<Response> {
        let client = self.http_client.ok_or_else(|| {
            crate::error::Error::HttpClient("No HTTP client configured".to_string())
        })?;

        // Build reqwest request
        let mut req = client
            .request(self.method.clone(), self.url.as_str())
            .timeout(self.timeout);

        // Add headers
        for (key, value) in &self.headers {
            req = req.header(key, value);
        }

        // Add body if present
        if let Some(body) = self.body {
            req = req.body(body);
        }

        // Send request with retry logic
        let mut attempt = 0;
        loop {
            match req
                .try_clone()
                .ok_or_else(|| {
                    crate::error::Error::HttpClient("Could not clone request".to_string())
                })?
                .send()
                .await
            {
                Ok(resp) => {
                    let status = resp.status();
                    let headers = resp.headers().clone();
                    let body = resp
                        .bytes()
                        .await
                        .map_err(|e| crate::error::Error::Connection(e.to_string()))?
                        .to_vec();

                    let response = Response::new(status, headers, body);

                    // Check if we should retry
                    if response.is_error() && attempt < self.max_retries {
                        let error = crate::error::Error::from_response(
                            status.as_u16(),
                            &String::from_utf8_lossy(response.body()),
                            response.headers(),
                        );

                        if error.is_retryable() {
                            attempt += 1;
                            if let Some(delay) = error.retry_after() {
                                tokio::time::sleep(delay).await;
                            } else {
                                // Exponential backoff: 1s, 2s
                                let delay = Duration::from_secs(2u64.pow(attempt - 1));
                                tokio::time::sleep(delay).await;
                            }
                            continue;
                        }
                    }

                    return Ok(response);
                }
                Err(e) if e.is_timeout() => {
                    if attempt >= self.max_retries {
                        return Err(crate::error::Error::Timeout(self.timeout));
                    }
                    attempt += 1;
                    tokio::time::sleep(Duration::from_secs(2u64.pow(attempt - 1))).await;
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

        let mut req = client
            .request(self.method.clone(), self.url.as_str())
            .timeout(self.timeout);

        for (key, value) in &self.headers {
            req = req.header(key, value);
        }

        if let Some(body) = self.body {
            req = req.body(body);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| crate::error::Error::Connection(e.to_string()))?;

        Ok(resp
            .bytes_stream()
            .map(|result| result.map_err(|e| crate::error::Error::Streaming(e.to_string()))))
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
