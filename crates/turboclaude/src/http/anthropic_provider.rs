//! Standard Anthropic API HTTP provider implementation
//!
//! This provider handles requests to the standard Anthropic API endpoints with
//! authentication, retries, rate limiting, and streaming support.

use super::{HttpProvider, Method, RequestBuilder, provider::serialize_body};
use crate::{DEFAULT_API_VERSION, error::Result};
use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;
use secrecy::{ExposeSecret, SecretString};
use std::{sync::Arc, time::Duration};
use url::Url;

/// HTTP provider for the standard Anthropic API.
///
/// This provider implements the `HttpProvider` trait for the standard Anthropic API,
/// handling authentication via API key or auth token, retries, rate limiting, and
/// server-sent events (SSE) streaming.
///
/// # Architecture
///
/// - Uses `reqwest` for HTTP client
/// - Supports both API key (`x-api-key`) and auth token (`Authorization: Bearer`) authentication
/// - Implements exponential backoff for retryable errors
/// - Manages custom headers and API versioning
///
/// # Example
///
/// ```rust,no_run
/// use turboclaude::http::AnthropicHttpProvider;
/// use std::sync::Arc;
///
/// let provider = Arc::new(AnthropicHttpProvider::builder()
///     .api_key("sk-ant-...")
///     .build()
///     .unwrap());
/// ```
#[derive(Debug, Clone)]
pub struct AnthropicHttpProvider {
    pub(crate) inner: Arc<ProviderInner>,
}

#[derive(Debug)]
pub(crate) struct ProviderInner {
    /// HTTP client for making requests
    pub(crate) http_client: reqwest::Client,
    /// Base URL for the API
    pub(crate) base_url: Url,
    /// API key for authentication (x-api-key header)
    pub(crate) api_key: Option<SecretString>,
    /// Auth token for authentication (Authorization: Bearer header)
    pub(crate) auth_token: Option<SecretString>,
    /// API version header value
    pub(crate) api_version: String,
    /// Default timeout for requests
    pub(crate) timeout: Duration,
    /// Maximum number of retries
    pub(crate) max_retries: u32,
    /// Custom headers to include with every request
    pub(crate) default_headers: http::HeaderMap,
}

impl AnthropicHttpProvider {
    /// Create a new builder for configuring the provider.
    pub fn builder() -> AnthropicHttpProviderBuilder {
        AnthropicHttpProviderBuilder::default()
    }

    /// Create a request builder with provider configuration.
    fn build_request(&self, method: Method, path: &str) -> Result<RequestBuilder> {
        let url = self.inner.base_url.join(path).map_err(|e| {
            crate::error::Error::InvalidUrl(format!(
                "Failed to construct URL from path '{}': {}",
                path, e
            ))
        })?;

        let mut builder = RequestBuilder::new(method, url)
            .with_client(self.inner.http_client.clone())
            .timeout(self.inner.timeout)
            .max_retries(self.inner.max_retries)
            .header("anthropic-version", &self.inner.api_version)
            .header("content-type", "application/json");

        // Add authentication
        if let Some(api_key) = &self.inner.api_key {
            builder = builder.header("x-api-key", api_key.expose_secret());
        } else if let Some(auth_token) = &self.inner.auth_token {
            builder = builder.header(
                "authorization",
                format!("Bearer {}", auth_token.expose_secret()),
            );
        }

        // Add custom default headers
        for (key, value) in &self.inner.default_headers {
            if let Ok(value_str) = value.to_str() {
                builder = builder.header(key.as_str(), value_str);
            }
        }

        Ok(builder)
    }

    /// Create a beta request builder with anthropic-beta header.
    pub fn build_beta_request(
        &self,
        method: Method,
        path: &str,
        beta_version: &str,
    ) -> Result<RequestBuilder> {
        Ok(self
            .build_request(method, path)?
            .header("anthropic-beta", beta_version))
    }
}

#[async_trait]
impl HttpProvider for AnthropicHttpProvider {
    async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<&(dyn erased_serde::Serialize + Send + Sync)>,
    ) -> Result<super::Response> {
        let mut builder = self.build_request(method, path)?;

        if let Some(body) = body {
            let body_bytes = serialize_body(body)?;
            builder = builder.body(body_bytes);
        }

        builder.send().await
    }

    async fn request_streaming(
        &self,
        method: Method,
        path: &str,
        body: Option<&(dyn erased_serde::Serialize + Send + Sync)>,
    ) -> Result<Box<dyn Stream<Item = Result<Bytes>> + Send + Unpin>> {
        let mut builder = self.build_request(method, path)?;

        if let Some(body) = body {
            let body_bytes = serialize_body(body)?;
            builder = builder.body(body_bytes);
        }

        let stream = builder.send_streaming().await?;
        Ok(Box::new(stream))
    }

    fn create_request(&self, method: Method, path: &str) -> Result<RequestBuilder> {
        self.build_request(method, path)
    }

    fn provider_name(&self) -> &'static str {
        "anthropic"
    }

    fn supports_beta(&self) -> bool {
        true
    }

    fn base_url(&self) -> &str {
        self.inner.base_url.as_str()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Builder for creating an `AnthropicHttpProvider` with custom configuration.
///
/// # Example
///
/// ```rust,no_run
/// use turboclaude::http::AnthropicHttpProvider;
///
/// let provider = AnthropicHttpProvider::builder()
///     .api_key("sk-ant-...")
///     .timeout(std::time::Duration::from_secs(120))
///     .max_retries(3)
///     .build()
///     .unwrap();
/// ```
#[derive(Default)]
pub struct AnthropicHttpProviderBuilder {
    api_key: Option<SecretString>,
    auth_token: Option<SecretString>,
    base_url: Option<String>,
    api_version: Option<String>,
    timeout: Option<Duration>,
    max_retries: Option<u32>,
    default_headers: http::HeaderMap,
}

impl AnthropicHttpProviderBuilder {
    /// Set the API key for authentication.
    ///
    /// This will use the `x-api-key` header for authentication.
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(SecretString::new(api_key.into().into_boxed_str()));
        self
    }

    /// Set the auth token for authentication (alternative to API key).
    ///
    /// This will use the `Authorization: Bearer` header for authentication.
    pub fn auth_token(mut self, auth_token: impl Into<String>) -> Self {
        self.auth_token = Some(SecretString::new(auth_token.into().into_boxed_str()));
        self
    }

    /// Set the base URL for the API.
    ///
    /// Defaults to `https://api.anthropic.com`.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Set the API version header value.
    ///
    /// Defaults to the current SDK version.
    pub fn api_version(mut self, version: impl Into<String>) -> Self {
        self.api_version = Some(version.into());
        self
    }

    /// Set the request timeout.
    ///
    /// Defaults to 600 seconds (10 minutes).
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the maximum number of retries for failed requests.
    ///
    /// Defaults to 2 retries.
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Add a custom header to include with every request.
    ///
    /// # Errors
    ///
    /// Returns an error if the header name or value is invalid.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Result<Self> {
        let key_str = key.into();
        let value_str = value.into();

        let key = key_str.parse::<http::HeaderName>().map_err(|e| {
            crate::error::Error::HttpClient(format!("Invalid header name '{}': {}", key_str, e))
        })?;
        let value = value_str.parse::<http::HeaderValue>().map_err(|e| {
            crate::error::Error::HttpClient(format!("Invalid header value '{}': {}", value_str, e))
        })?;

        self.default_headers.insert(key, value);
        Ok(self)
    }

    /// Build the provider with the configured settings.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Neither API key nor auth token is provided
    /// - The base URL is invalid
    /// - HTTP client creation fails
    pub fn build(mut self) -> Result<AnthropicHttpProvider> {
        // Check authentication
        if self.api_key.is_none() && self.auth_token.is_none() {
            #[cfg(feature = "env")]
            {
                use std::env;
                self.api_key = env::var("ANTHROPIC_API_KEY")
                    .ok()
                    .map(|s| SecretString::new(s.into_boxed_str()));
                self.auth_token = env::var("ANTHROPIC_AUTH_TOKEN")
                    .ok()
                    .map(|s| SecretString::new(s.into_boxed_str()));

                if self.api_key.is_none() && self.auth_token.is_none() {
                    return Err(crate::error::Error::Authentication(
                        "No API key or auth token provided. Set ANTHROPIC_API_KEY environment variable or provide credentials explicitly.".to_string()
                    ));
                }
            }

            #[cfg(not(feature = "env"))]
            return Err(crate::error::Error::Authentication(
                "No API key or auth token provided".to_string(),
            ));
        }

        // Destructure to avoid partial move
        let Self {
            api_key,
            auth_token,
            base_url,
            api_version,
            timeout,
            max_retries,
            default_headers,
        } = self;

        Self::build_with_credentials(
            api_key,
            auth_token,
            base_url,
            api_version,
            timeout,
            max_retries,
            default_headers,
        )
    }

    /// Internal helper to build with provided credentials and configuration.
    fn build_with_credentials(
        api_key: Option<SecretString>,
        auth_token: Option<SecretString>,
        base_url: Option<String>,
        api_version: Option<String>,
        timeout: Option<Duration>,
        max_retries: Option<u32>,
        default_headers: http::HeaderMap,
    ) -> Result<AnthropicHttpProvider> {
        let timeout = timeout.unwrap_or(Duration::from_secs(600));

        let http_client = reqwest::Client::builder()
            .timeout(timeout)
            .user_agent(format!("turboclaude-rust/{}", crate::VERSION))
            .build()
            .map_err(|e| crate::error::Error::HttpClient(e.to_string()))?;

        let base_url_string = base_url.unwrap_or_else(|| crate::DEFAULT_BASE_URL.to_string());

        if base_url_string.trim().is_empty() {
            return Err(crate::error::Error::InvalidUrl(
                "Base URL cannot be empty".to_string(),
            ));
        }

        let base_url: Url = base_url_string
            .parse()
            .map_err(|e| crate::error::Error::InvalidUrl(format!("{}", e)))?;

        // Validate URL scheme
        match base_url.scheme() {
            "http" | "https" => {}
            scheme => {
                return Err(crate::error::Error::InvalidUrl(format!(
                    "Invalid URL scheme '{}'. Only 'http' and 'https' are supported.",
                    scheme
                )));
            }
        }

        let inner = Arc::new(ProviderInner {
            http_client,
            base_url,
            api_key,
            auth_token,
            api_version: api_version.unwrap_or_else(|| DEFAULT_API_VERSION.to_string()),
            timeout,
            max_retries: max_retries.unwrap_or(2),
            default_headers,
        });

        Ok(AnthropicHttpProvider { inner })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_with_api_key() {
        let provider = AnthropicHttpProvider::builder()
            .api_key("test-key")
            .build()
            .unwrap();

        assert_eq!(provider.provider_name(), "anthropic");
        assert!(provider.supports_beta());
    }

    #[test]
    fn test_builder_with_auth_token() {
        let provider = AnthropicHttpProvider::builder()
            .auth_token("test-token")
            .build()
            .unwrap();

        assert_eq!(provider.provider_name(), "anthropic");
    }

    #[test]
    fn test_builder_without_credentials_fails() {
        let result = AnthropicHttpProvider::builder().build();
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_with_custom_config() {
        let provider = AnthropicHttpProvider::builder()
            .api_key("test-key")
            .base_url("https://custom.api.com")
            .timeout(Duration::from_secs(30))
            .max_retries(5)
            .api_version("2025-01-01")
            .build()
            .unwrap();

        assert_eq!(provider.base_url(), "https://custom.api.com/");
        assert_eq!(provider.inner.timeout, Duration::from_secs(30));
        assert_eq!(provider.inner.max_retries, 5);
        assert_eq!(provider.inner.api_version, "2025-01-01");
    }

    #[test]
    fn test_builder_with_custom_headers() {
        let provider = AnthropicHttpProvider::builder()
            .api_key("test-key")
            .header("X-Custom-Header", "custom-value")
            .unwrap()
            .build()
            .unwrap();

        assert!(
            provider
                .inner
                .default_headers
                .contains_key("x-custom-header")
        );
    }

    #[test]
    fn test_build_request() {
        let provider = AnthropicHttpProvider::builder()
            .api_key("test-key")
            .build()
            .unwrap();

        let request = provider.build_request(Method::GET, "/v1/messages").unwrap();
        assert_eq!(request.method(), &Method::GET);
        assert!(request.url().as_str().ends_with("/v1/messages"));
    }

    #[test]
    fn test_build_beta_request() {
        let provider = AnthropicHttpProvider::builder()
            .api_key("test-key")
            .build()
            .unwrap();

        let request = provider
            .build_beta_request(Method::POST, "/v1/messages", "tools-2025-01-01")
            .unwrap();

        let headers = request.headers();
        assert!(headers.contains_key("anthropic-beta"));
    }
}
