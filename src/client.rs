//! Main client implementation for the Anthropic API

use std::sync::Arc;
use std::time::Duration;

use secrecy::{ExposeSecret, SecretString};
use std::sync::OnceLock;
use url::Url;

use crate::{
    config::ClientConfig,
    error::{Error, Result},
    http::RequestBuilder,
    resources::{Messages, Completions, Models, Beta},
    DEFAULT_BASE_URL, DEFAULT_API_VERSION,
};

/// Main client for interacting with the Anthropic API.
///
/// This client provides access to all Anthropic API endpoints and handles
/// authentication, retries, rate limiting, and other common concerns.
///
/// # Example
///
/// ```rust,no_run
/// use anthropic::Client;
///
/// let client = Client::new("sk-ant-...");
/// ```
#[derive(Clone)]
pub struct Client {
    inner: Arc<ClientInner>,
}

struct ClientInner {
    /// HTTP client for making requests
    http_client: reqwest::Client,
    /// Base URL for the API
    base_url: Url,
    /// API key for authentication
    api_key: Option<SecretString>,
    /// Auth token (alternative to API key)
    auth_token: Option<SecretString>,
    /// API version header value
    api_version: String,
    /// Default timeout for requests
    timeout: Duration,
    /// Maximum number of retries
    max_retries: u32,
    /// Custom headers to include with every request
    default_headers: http::HeaderMap,
    /// Rate limiter for controlling request throughput
    rate_limiter: Option<std::sync::Arc<governor::RateLimiter<governor::state::Direct, governor::clock::DefaultClock>>>,

    // Lazy-initialized resources (like Python's @cached_property)
    messages: OnceLock<Messages>,
    completions: OnceLock<Completions>,
    models: OnceLock<Models>,
    beta: OnceLock<Beta>,
}

impl Client {
    /// Create a new client with an API key.
    ///
    /// The API key can also be loaded from the `ANTHROPIC_API_KEY` environment
    /// variable if the `env` feature is enabled.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::builder()
            .api_key(api_key)
            .build()
            .expect("Failed to build client with provided API key")
    }

    /// Create a new client builder for advanced configuration.
    pub fn builder() -> AnthropicClientBuilder {
        AnthropicClientBuilder::default()
    }

    /// Create a client from a configuration object.
    ///
    /// Applies all configuration settings including proxy, connection pooling,
    /// and rate limiting from the provided `ClientConfig`.
    pub fn from_config(config: ClientConfig) -> Result<Self> {
        let mut client_builder = reqwest::Client::builder()
            .timeout(config.timeout)
            .user_agent(format!("anthropic-rust/{}", crate::VERSION))
            .http2_prior_knowledge();  // Assume HTTP/2 for connection reuse

        // Apply connection pool configuration
        client_builder = client_builder
            .pool_max_idle_per_host(config.connection_pool.max_idle_per_host)
            .pool_idle_timeout(config.connection_pool.idle_timeout);

        // Apply TCP keep-alive if configured
        if let Some(keepalive) = config.connection_pool.tcp_keepalive {
            client_builder = client_builder.tcp_keepalive(Some(keepalive));
        }

        // Apply proxy configuration if provided
        if let Some(proxy_url) = &config.proxy {
            let proxy = reqwest::Proxy::all(proxy_url)
                .map_err(|e| Error::HttpClient(format!("Invalid proxy URL: {}", e)))?;
            client_builder = client_builder.proxy(proxy);
        }

        let http_client = client_builder
            .build()
            .map_err(|e| Error::HttpClient(e.to_string()))?;

        let base_url_string = config.base_url
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        // Validate base URL is not empty
        if base_url_string.trim().is_empty() {
            return Err(Error::InvalidUrl("Base URL cannot be empty".to_string()));
        }

        let base_url: Url = base_url_string
            .parse()
            .map_err(|e| Error::InvalidUrl(format!("{}", e)))?;

        // Validate URL scheme is HTTP or HTTPS
        match base_url.scheme() {
            "http" | "https" => {},
            scheme => return Err(Error::InvalidUrl(format!(
                "Invalid URL scheme '{}'. Only 'http' and 'https' are supported.",
                scheme
            ))),
        }

        // Get API key/auth token from config or environment
        let mut api_key = config.api_key;
        let mut auth_token = config.auth_token;

        if api_key.is_none() && auth_token.is_none() {
            // Try to load from environment if `env` feature is enabled
            #[cfg(feature = "env")]
            {
                use std::env;
                api_key = env::var("ANTHROPIC_API_KEY").ok().map(|s| SecretString::new(s.into_boxed_str()));
                auth_token = env::var("ANTHROPIC_AUTH_TOKEN").ok().map(|s| SecretString::new(s.into_boxed_str()));

                if api_key.is_none() && auth_token.is_none() {
                    return Err(Error::Authentication(
                        "No API key or auth token provided. Set ANTHROPIC_API_KEY environment variable or provide credentials explicitly.".to_string()
                    ));
                }
            }

            #[cfg(not(feature = "env"))]
            return Err(Error::Authentication(
                "No API key or auth token provided".to_string()
            ));
        }

        // Initialize rate limiter if configured
        let rate_limiter = config.rate_limit.map(|rate_config| {
            use governor::Quota;
            use std::num::NonZeroU32;

            let quota = Quota::per_second(
                NonZeroU32::new((rate_config.requests_per_second.max(1.0)) as u32)
                    .unwrap_or(NonZeroU32::new(1).unwrap())
            );

            let limiter = governor::RateLimiter::direct(quota);
            std::sync::Arc::new(limiter)
        });

        let inner = Arc::new(ClientInner {
            http_client,
            base_url,
            api_key,
            auth_token,
            api_version: config.api_version
                .unwrap_or_else(|| DEFAULT_API_VERSION.to_string()),
            timeout: config.timeout,
            max_retries: config.max_retries,
            default_headers: config.default_headers,
            rate_limiter,
            messages: OnceLock::new(),
            completions: OnceLock::new(),
            models: OnceLock::new(),
            beta: OnceLock::new(),
        });

        Ok(Self { inner })
    }

    /// Access the Messages API endpoint.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use anthropic::{Client, Message, MessageRequest};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("api-key");
    /// # let request = MessageRequest::builder()
    /// #     .model("claude-3-5-sonnet-20241022")
    /// #     .max_tokens(1024u32)
    /// #     .messages(vec![Message::user("Hello")])
    /// #     .build()?;
    /// let message = client.messages()
    ///     .create(request)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn messages(&self) -> &Messages {
        self.inner.messages.get_or_init(|| Messages::new(self.clone()))
    }

    /// Access the Completions API endpoint (legacy).
    pub fn completions(&self) -> &Completions {
        self.inner.completions.get_or_init(|| Completions::new(self.clone()))
    }

    /// Access the Models API endpoint.
    pub fn models(&self) -> &Models {
        self.inner.models.get_or_init(|| Models::new(self.clone()))
    }

    /// Access beta API features.
    ///
    /// # Example
    ///
    /// ```rust,no_run,ignore
    /// # use anthropic::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("api-key");
    /// // Beta features not yet implemented
    /// let tools = client.beta()
    ///     .tools()
    ///     .list()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn beta(&self) -> &Beta {
        self.inner.beta.get_or_init(|| Beta::new(self.clone()))
    }

    /// Apply rate limiting if configured.
    ///
    /// This method will block (asynchronously) until a token is available
    /// if rate limiting is enabled on the client.
    pub(crate) async fn apply_rate_limit(&self) {
        if let Some(ref limiter) = self.inner.rate_limiter {
            // For synchronous rate limiting, we use the direct method
            // which blocks the current thread
            while limiter.check().is_err() {
                // Sleep for a short duration to avoid busy-waiting
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        }
    }

    /// Create a request builder for custom requests.
    pub(crate) fn request(&self, method: http::Method, path: &str) -> RequestBuilder {
        let url = self.inner.base_url.join(path)
            .expect("Failed to construct URL");

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
            builder = builder.header("authorization", format!("Bearer {}", auth_token.expose_secret()));
        }

        // Add custom default headers
        for (key, value) in &self.inner.default_headers {
            builder = builder.header(key.as_str(), value.to_str().unwrap_or(""));
        }

        builder
    }

    /// Create a request builder for beta API requests with beta header injection.
    ///
    /// This is similar to `request()` but adds the `anthropic-beta` header
    /// required for beta features.
    pub(crate) fn beta_request(&self, method: http::Method, path: &str, beta_version: &str) -> RequestBuilder {
        self.request(method, path)
            .header("anthropic-beta", beta_version)
    }

    /// Get a copy of the HTTP client for internal use
    #[allow(dead_code)]
    pub(crate) fn http_client(&self) -> &reqwest::Client {
        &self.inner.http_client
    }

    /// Get the base URL for the API
    pub(crate) fn base_url(&self) -> &str {
        self.inner.base_url.as_str()
    }

    /// Get the API key (for beta features that need direct access)
    pub(crate) fn api_key(&self) -> &str {
        self.inner.api_key.as_ref()
            .map(|k| k.expose_secret().as_str())
            .unwrap_or("")
    }

    /// Create a multipart request builder for file uploads and other multipart operations.
    ///
    /// This builder applies the same configuration (timeouts, retries, headers) as regular
    /// requests but supports multipart form data.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use anthropic::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("api-key");
    /// let form = reqwest::multipart::Form::new()
    ///     .text("field_name", "field_value");
    ///
    /// let response = client.multipart_request(http::Method::POST, "/v1/files")?
    ///     .multipart(form)
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub(crate) fn multipart_request(&self, method: http::Method, path: &str) -> Result<reqwest::RequestBuilder> {
        let url = self.inner.base_url.join(path)
            .map_err(|e| Error::InvalidUrl(format!("Failed to construct URL: {}", e)))?;

        let mut req = self.inner.http_client
            .request(method, url)
            .timeout(self.inner.timeout)
            .header("anthropic-version", &self.inner.api_version);

        // Add authentication
        if let Some(api_key) = &self.inner.api_key {
            req = req.header("x-api-key", api_key.expose_secret());
        } else if let Some(auth_token) = &self.inner.auth_token {
            req = req.header("authorization", format!("Bearer {}", auth_token.expose_secret()));
        }

        // Add custom default headers
        for (key, value) in &self.inner.default_headers {
            req = req.header(key.as_str(), value.to_str().unwrap_or(""));
        }

        Ok(req)
    }
}

/// Builder for creating a configured Client.
#[derive(Default)]
pub struct AnthropicClientBuilder {
    config: ClientConfig,
}

impl AnthropicClientBuilder {
    /// Set the API key for authentication.
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.config.api_key = Some(SecretString::new(api_key.into().into_boxed_str()));
        self
    }

    /// Set the auth token for authentication (alternative to API key).
    pub fn auth_token(mut self, auth_token: impl Into<String>) -> Self {
        self.config.auth_token = Some(SecretString::new(auth_token.into().into_boxed_str()));
        self
    }

    /// Set the base URL for the API.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.config.base_url = Some(base_url.into());
        self
    }

    /// Set the API version header value.
    pub fn api_version(mut self, api_version: impl Into<String>) -> Self {
        self.config.api_version = Some(api_version.into());
        self
    }

    /// Set the default timeout for requests.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Set the maximum number of retries.
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.config.max_retries = max_retries;
        self
    }

    /// Add a custom default header.
    pub fn default_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let key: http::HeaderName = key.into().parse().expect("Invalid header name");
        let value: http::HeaderValue = value.into().parse().expect("Invalid header value");
        self.config.default_headers.insert(key, value);
        self
    }

    /// Build the client with the configured options.
    pub fn build(self) -> Result<Client> {
        Client::from_config(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_builder() {
        let client = Client::builder()
            .api_key("test-key")
            .base_url("https://example.com")
            .timeout(Duration::from_secs(30))
            .max_retries(3)
            .build();

        assert!(client.is_ok());
    }

    #[test]
    fn test_client_new() {
        let client = Client::new("test-key");
        // Should not panic
        let _ = client.messages();
        let _ = client.completions();
        let _ = client.models();
        let _ = client.beta();
    }

    #[test]
    fn test_client_clone() {
        let client1 = Client::new("test-key");
        let client2 = client1.clone();

        // Both clients should work
        let _ = client1.messages();
        let _ = client2.messages();
    }
}