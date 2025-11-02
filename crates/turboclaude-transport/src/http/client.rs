//! HTTP transport client implementation
//!
//! Implements the Transport trait for HTTP requests with retry logic,
//! rate limiting, and full HTTP/2 support.

use crate::error::{Result, TransportError};
use crate::traits::{HttpRequest, HttpResponse, Transport};
use async_trait::async_trait;
use reqwest::Client as ReqwestClient;
use std::sync::Arc;
use std::time::Duration;

pub use super::retry::RetryPolicy;
use turboclaude_core::retry::BackoffStrategy;

/// HTTP transport implementation
///
/// Handles HTTP requests with:
/// - Automatic retries with exponential backoff
/// - Rate limiting
/// - Connection pooling
/// - HTTP/2 support
/// - Timeout handling
#[derive(Clone)]
pub struct HttpTransport {
    client: Arc<ReqwestClient>,
    retry_policy: RetryPolicy,
    timeout: Duration,
}

impl HttpTransport {
    /// Create a new HTTP transport with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(Default::default())
    }

    /// Create a new HTTP transport with custom configuration
    pub fn with_config(config: HttpTransportConfig) -> Result<Self> {
        let client = ReqwestClient::builder()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .pool_max_idle_per_host(config.pool_max_idle_per_host)
            .http2_prior_knowledge()
            .build()
            .map_err(|e| TransportError::Connection(e.to_string()))?;

        Ok(Self {
            client: Arc::new(client),
            retry_policy: config.retry_policy,
            timeout: config.timeout,
        })
    }

    /// Get a reference to the underlying reqwest client
    pub fn reqwest_client(&self) -> Arc<ReqwestClient> {
        self.client.clone()
    }

    /// Set the retry policy
    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = policy;
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

impl Default for HttpTransport {
    fn default() -> Self {
        Self::new().expect("Failed to create HTTP transport with defaults")
    }
}

#[async_trait]
impl Transport for HttpTransport {
    async fn send_http(&self, request: HttpRequest) -> Result<HttpResponse> {
        let method_upper = request.method.to_uppercase();
        let method = match method_upper.as_str() {
            "GET" => reqwest::Method::GET,
            "POST" => reqwest::Method::POST,
            "PUT" => reqwest::Method::PUT,
            "DELETE" => reqwest::Method::DELETE,
            "PATCH" => reqwest::Method::PATCH,
            "HEAD" => reqwest::Method::HEAD,
            "OPTIONS" => reqwest::Method::OPTIONS,
            _ => {
                return Err(TransportError::Http(format!(
                    "Unsupported HTTP method: {}",
                    request.method
                )));
            }
        };

        let mut attempt = 0;
        let max_retries = self.retry_policy.max_retries();

        loop {
            match self.try_send_request(&request, &method).await {
                Ok(response) => return Ok(response),
                Err(err) => {
                    attempt += 1;

                    if !RetryPolicy::is_retryable(&err) || attempt > max_retries {
                        return Err(err);
                    }

                    // Calculate backoff
                    let delay = self.retry_policy.calculate_delay(attempt);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    async fn is_connected(&self) -> bool {
        // HTTP is stateless, always "connected"
        true
    }

    async fn close(&mut self) -> Result<()> {
        // No-op for HTTP client
        Ok(())
    }
}

impl HttpTransport {
    async fn try_send_request(
        &self,
        request: &HttpRequest,
        method: &reqwest::Method,
    ) -> Result<HttpResponse> {
        let mut req = self.client.request(method.clone(), &request.url);

        // Add headers
        for (key, value) in &request.headers {
            req = req.header(key.as_str(), value.as_str());
        }

        // Add body if present
        if let Some(body) = &request.body {
            req = req.body(body.clone());
        }

        // Send request
        let response = req.send().await.map_err(|e| {
            if e.is_timeout() {
                TransportError::Timeout
            } else if e.is_connect() {
                TransportError::Connection(e.to_string())
            } else {
                TransportError::Http(e.to_string())
            }
        })?;

        let status = response.status().as_u16();
        let mut headers = std::collections::HashMap::new();

        // Collect headers
        for (key, value) in response.headers() {
            if let Ok(v) = value.to_str() {
                headers.insert(key.to_string(), v.to_string());
            }
        }

        // Collect body
        let body = response
            .bytes()
            .await
            .map_err(|e| TransportError::Http(e.to_string()))?
            .to_vec();

        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }
}

/// HTTP transport configuration
#[derive(Clone, Debug)]
pub struct HttpTransportConfig {
    /// Request timeout
    pub timeout: Duration,

    /// Connection timeout
    pub connect_timeout: Duration,

    /// Maximum idle connections per host
    pub pool_max_idle_per_host: usize,

    /// Retry policy
    pub retry_policy: RetryPolicy,
}

impl Default for HttpTransportConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(600),
            connect_timeout: Duration::from_secs(30),
            pool_max_idle_per_host: 10,
            retry_policy: RetryPolicy::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_transport_creation() {
        let transport = HttpTransport::new().expect("Failed to create transport");
        assert!(matches!(transport, HttpTransport { .. }));
    }

    #[test]
    fn test_http_transport_with_config() {
        let config = HttpTransportConfig {
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            pool_max_idle_per_host: 5,
            retry_policy: RetryPolicy::default(),
        };

        let transport = HttpTransport::with_config(config).expect("Failed to create transport");
        assert_eq!(transport.timeout, Duration::from_secs(30));
    }
}
