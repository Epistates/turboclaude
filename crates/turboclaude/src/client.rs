//! Main client implementation for the Anthropic API

use std::sync::Arc;
use std::time::Duration;

use secrecy::{ExposeSecret, SecretString};
use std::sync::OnceLock;

use crate::{
    config::ClientConfig,
    error::{Error, Result},
    http::{AnthropicHttpProvider, HttpProvider, RequestBuilder},
    resources::{Beta, Completions, Messages, Models},
};

/// Main client for interacting with the Anthropic API.
///
/// This client provides access to all Anthropic API endpoints and handles
/// authentication, retries, rate limiting, and other common concerns.
///
/// # Example
///
/// ```rust,no_run
/// use turboclaude::Client;
///
/// let client = Client::new("sk-ant-...");
/// ```
#[derive(Clone)]
pub struct Client {
    inner: Arc<ClientInner>,
}

struct ClientInner {
    /// HTTP provider for making requests (handles auth, retries, etc.)
    provider: Arc<dyn HttpProvider>,

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
    ///
    /// # Panics
    ///
    /// This convenience method panics if the client cannot be built with the default
    /// configuration. For fallible construction with explicit error handling, use
    /// [`Client::try_new()`] instead.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use turboclaude::Client;
    ///
    /// let client = Client::new("sk-ant-...");
    /// ```
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::builder()
            .api_key(api_key)
            .build()
            .expect("Failed to build client with provided API key")
    }

    /// Create a new client with an API key (fallible version).
    ///
    /// This is the fallible version of [`Client::new()`] that returns a `Result`
    /// instead of panicking on error.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The base URL is invalid (e.g., from environment variable)
    /// - HTTP client configuration fails
    /// - Required configuration is missing
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use turboclaude::Client;
    ///
    /// let client = Client::try_new("sk-ant-...").expect("Failed to create client");
    /// ```
    pub fn try_new(api_key: impl Into<String>) -> Result<Self> {
        Self::builder().api_key(api_key).build()
    }

    /// Create a new client builder for advanced configuration.
    pub fn builder() -> AnthropicClientBuilder {
        AnthropicClientBuilder::default()
    }

    /// Create a client with a custom HTTP provider.
    ///
    /// This allows using alternative providers like AWS Bedrock or Google Vertex AI
    /// instead of the default Anthropic API.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "bedrock")]
    /// # {
    /// use turboclaude::Client;
    /// use turboclaude::providers::bedrock::BedrockHttpProvider;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let provider = Arc::new(
    ///     BedrockHttpProvider::builder()
    ///         .region("us-east-1")
    ///         .build()
    ///         .await?
    /// );
    ///
    /// let client = Client::from_provider(provider);
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    pub fn from_provider(provider: Arc<dyn HttpProvider>) -> Self {
        Self {
            inner: Arc::new(ClientInner {
                provider,
                messages: OnceLock::new(),
                completions: OnceLock::new(),
                models: OnceLock::new(),
                beta: OnceLock::new(),
            }),
        }
    }

    /// Create a client from a configuration object.
    pub fn from_config(config: ClientConfig) -> Result<Self> {
        // Build the Anthropic HTTP provider from config
        let mut provider_builder = AnthropicHttpProvider::builder();

        // Set credentials
        if let Some(api_key) = config.api_key {
            provider_builder = provider_builder.api_key(api_key.expose_secret());
        }
        if let Some(auth_token) = config.auth_token {
            provider_builder = provider_builder.auth_token(auth_token.expose_secret());
        }

        // Set optional configuration
        if let Some(base_url) = config.base_url {
            provider_builder = provider_builder.base_url(base_url);
        }
        if let Some(api_version) = config.api_version {
            provider_builder = provider_builder.api_version(api_version);
        }
        provider_builder = provider_builder
            .timeout(config.timeout)
            .max_retries(config.max_retries);

        // Add custom headers
        for (key, value) in config.default_headers {
            if let (Some(key), Ok(value_str)) = (key, value.to_str()) {
                provider_builder = provider_builder.header(key.as_str(), value_str)?;
            }
        }

        // Build the provider (this will handle env var loading if needed)
        let provider = Arc::new(provider_builder.build()?);

        let inner = Arc::new(ClientInner {
            provider,
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
    /// # use turboclaude::{Client, Message, MessageRequest};
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
        self.inner
            .messages
            .get_or_init(|| Messages::new(self.clone()))
    }

    /// Access the Completions API endpoint (legacy).
    pub fn completions(&self) -> &Completions {
        self.inner
            .completions
            .get_or_init(|| Completions::new(self.clone()))
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
    /// # use turboclaude::Client;
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

    /// Create a request builder for custom requests.
    ///
    /// # Errors
    ///
    /// Returns an error if the URL cannot be constructed from the base URL and path.
    pub(crate) fn request(&self, method: http::Method, path: &str) -> Result<RequestBuilder> {
        self.inner.provider.create_request(method, path)
    }

    /// Create a request builder for beta API requests with beta header injection.
    ///
    /// This is similar to `request()` but adds the `anthropic-beta` header
    /// required for beta features.
    pub(crate) fn beta_request(
        &self,
        method: http::Method,
        path: &str,
        beta_version: &str,
    ) -> Result<RequestBuilder> {
        // Try to downcast to AnthropicHttpProvider for beta support
        if let Some(anthropic_provider) = self
            .inner
            .provider
            .as_any()
            .downcast_ref::<AnthropicHttpProvider>()
        {
            anthropic_provider.build_beta_request(method, path, beta_version)
        } else {
            // Fallback: add header manually
            Ok(self
                .request(method, path)?
                .header("anthropic-beta", beta_version))
        }
    }

    /// Get the base URL for the API
    pub(crate) fn base_url(&self) -> &str {
        self.inner.provider.base_url()
    }

    /// Get the provider name (for debugging)
    #[allow(dead_code)]
    pub(crate) fn provider_name(&self) -> &'static str {
        self.inner.provider.provider_name()
    }

    /// Get HTTP client for special cases (multipart uploads, etc.)
    ///
    /// This is only available when using AnthropicHttpProvider.
    ///
    /// # Panics
    ///
    /// Panics if the provider is not AnthropicHttpProvider.
    #[allow(dead_code)]
    pub(crate) fn http_client(&self) -> &reqwest::Client {
        self.inner
            .provider
            .as_any()
            .downcast_ref::<AnthropicHttpProvider>()
            .map(|p| &p.inner.http_client)
            .expect("http_client() is only available with AnthropicHttpProvider")
    }

    /// Get API key for special cases that need direct access
    ///
    /// This is only available when using AnthropicHttpProvider with API key auth.
    /// For other providers or auth methods, this will return an empty string.
    pub(crate) fn api_key(&self) -> String {
        self.inner
            .provider
            .as_any()
            .downcast_ref::<AnthropicHttpProvider>()
            .and_then(|p| p.inner.api_key.as_ref())
            .map(|k| k.expose_secret().to_string())
            .unwrap_or_default()
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
    ///
    /// # Errors
    ///
    /// Returns an error if the header name or value is invalid according to HTTP specifications.
    pub fn default_header(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self> {
        let key_str = key.into();
        let value_str = value.into();

        let key: http::HeaderName = key_str
            .parse()
            .map_err(|_| Error::InvalidHeaderName(key_str.clone()))?;
        let value: http::HeaderValue = value_str
            .parse()
            .map_err(|_| Error::InvalidHeaderValue(value_str.clone()))?;

        self.config.default_headers.insert(key, value);
        Ok(self)
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

    /// Test 1: Client from config with valid URL
    #[test]
    fn test_client_from_config_valid_url() {
        let config = ClientConfig {
            api_key: Some(SecretString::new("test-api-key".into())),
            auth_token: None,
            base_url: Some("https://api.example.com".to_string()),
            api_version: Some("2024-01-01".to_string()),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            default_headers: http::HeaderMap::new(),
            proxy: None,
            connection_pool: crate::config::ConnectionPoolConfig::default(),
            rate_limit: None,
        };

        let client = Client::from_config(config);
        assert!(
            client.is_ok(),
            "Client creation should succeed with valid config"
        );

        let client = client.unwrap();
        assert_eq!(client.base_url(), "https://api.example.com/");
    }

    /// Test 2: Client from config with invalid URL scheme
    #[test]
    fn test_client_from_config_invalid_scheme() {
        let config = ClientConfig {
            api_key: Some(SecretString::new("test-api-key".into())),
            auth_token: None,
            base_url: Some("ftp://invalid.example.com".to_string()),
            api_version: None,
            timeout: Duration::from_secs(600),
            max_retries: 2,
            default_headers: http::HeaderMap::new(),
            proxy: None,
            connection_pool: crate::config::ConnectionPoolConfig::default(),
            rate_limit: None,
        };

        let result = Client::from_config(config);
        assert!(result.is_err(), "Should reject non-HTTP/HTTPS schemes");

        match result {
            Err(Error::InvalidUrl(msg)) => {
                assert!(msg.contains("ftp"), "Error should mention invalid scheme");
                assert!(
                    msg.contains("http") || msg.contains("https"),
                    "Error should mention valid schemes"
                );
            }
            _ => panic!("Expected InvalidUrl error"),
        }
    }

    /// Test 3: Client from config with empty URL
    #[test]
    fn test_client_from_config_empty_url() {
        let config = ClientConfig {
            api_key: Some(SecretString::new("test-api-key".into())),
            auth_token: None,
            base_url: Some("   ".to_string()), // Empty/whitespace URL
            api_version: None,
            timeout: Duration::from_secs(600),
            max_retries: 2,
            default_headers: http::HeaderMap::new(),
            proxy: None,
            connection_pool: crate::config::ConnectionPoolConfig::default(),
            rate_limit: None,
        };

        let result = Client::from_config(config);
        assert!(result.is_err(), "Should reject empty URLs");

        match result {
            Err(Error::InvalidUrl(msg)) => {
                assert!(msg.contains("empty"), "Error should mention empty URL");
            }
            _ => panic!("Expected InvalidUrl error for empty URL"),
        }
    }

    /// Test 4: Verify lazy initialization of resources
    #[test]
    fn test_resource_lazy_initialization() {
        let client = Client::new("test-key");

        // Resources should be initialized on first access via OnceLock
        let messages1 = client.messages();
        let messages2 = client.messages();

        // Should return the same instance (pointer equality)
        assert!(
            std::ptr::eq(messages1, messages2),
            "Multiple calls should return same Messages instance"
        );

        // Same for other resources
        let completions1 = client.completions();
        let completions2 = client.completions();
        assert!(
            std::ptr::eq(completions1, completions2),
            "Multiple calls should return same Completions instance"
        );

        let models1 = client.models();
        let models2 = client.models();
        assert!(
            std::ptr::eq(models1, models2),
            "Multiple calls should return same Models instance"
        );

        let beta1 = client.beta();
        let beta2 = client.beta();
        assert!(
            std::ptr::eq(beta1, beta2),
            "Multiple calls should return same Beta instance"
        );
    }

    /// Test 5: Client clone shares Arc
    #[test]
    fn test_client_clone_shares_arc() {
        let client1 = Client::new("test-key");
        let client2 = client1.clone();

        // Both should share the same Arc<ClientInner>
        // We can verify this by checking that Arc::strong_count increases
        // (though we can't directly access the Arc, we can test behavior)

        // Access resources on both clients
        let _msg1 = client1.messages();
        let _msg2 = client2.messages();

        // If Arc is shared properly, modifying state in one shouldn't affect the other
        // (they share the same state, which is what we want)

        // Verify both can access the same base URL
        assert_eq!(client1.base_url(), client2.base_url());
        assert_eq!(client1.api_key(), client2.api_key());
    }

    /// Test 6: Config merge precedence
    #[test]
    fn test_config_merge_precedence() {
        let config1 = ClientConfig {
            api_key: Some(SecretString::new("key1".into())),
            auth_token: None,
            base_url: Some("https://base1.com".to_string()),
            api_version: Some("2024-01-01".to_string()),
            timeout: Duration::from_secs(30),
            max_retries: 2,
            default_headers: {
                let mut headers = http::HeaderMap::new();
                headers.insert(
                    "x-custom".parse::<http::HeaderName>().unwrap(),
                    "value1".parse::<http::HeaderValue>().unwrap(),
                );
                headers
            },
            proxy: Some("http://proxy1.com".to_string()),
            connection_pool: crate::config::ConnectionPoolConfig::default(),
            rate_limit: None,
        };

        let config2 = ClientConfig {
            api_key: Some(SecretString::new("key2".into())),
            auth_token: None,
            base_url: Some("https://base2.com".to_string()),
            api_version: None,
            timeout: Duration::from_secs(60),
            max_retries: 5,
            default_headers: {
                let mut headers = http::HeaderMap::new();
                headers.insert(
                    "x-other".parse::<http::HeaderName>().unwrap(),
                    "value2".parse::<http::HeaderValue>().unwrap(),
                );
                headers
            },
            proxy: None,
            connection_pool: crate::config::ConnectionPoolConfig::default(),
            rate_limit: Some(crate::config::RateLimitConfig::default()),
        };

        let merged = config1.merge(config2);

        // config2 values should take precedence
        assert_eq!(merged.base_url, Some("https://base2.com".to_string()));
        assert_eq!(merged.timeout, Duration::from_secs(60));
        assert_eq!(merged.max_retries, 5);

        // Headers should be merged (both present)
        assert!(merged.default_headers.contains_key("x-custom"));
        assert!(merged.default_headers.contains_key("x-other"));

        // config2's None should not override config1's Some
        // Actually, based on the merge logic, api_version should remain from config1
        // since config2.api_version is None
        assert_eq!(merged.api_version, Some("2024-01-01".to_string()));

        // proxy: config2 is None, but the merge logic keeps config1's value
        // Wait, looking at the code: if other.proxy.is_some() { self.proxy = other.proxy; }
        // So None in config2 won't override. proxy should stay as config1's
        assert_eq!(merged.proxy, Some("http://proxy1.com".to_string()));

        // rate_limit: config2 has Some, should override
        assert!(merged.rate_limit.is_some());
    }

    /// Test 7: Config from environment variables
    #[cfg(feature = "env")]
    #[test]
    fn test_config_from_env_variables() {
        // Use temp-env for safe, thread-safe environment variable management (Rust 2024 compliant)
        temp_env::with_vars(
            [
                ("ANTHROPIC_API_KEY", Some("test-env-key".to_string())),
                (
                    "ANTHROPIC_BASE_URL",
                    Some("https://env-base.com".to_string()),
                ),
                ("ANTHROPIC_API_VERSION", Some("2024-02-01".to_string())),
                ("ANTHROPIC_TIMEOUT", Some("120".to_string())),
                ("ANTHROPIC_MAX_RETRIES", Some("5".to_string())),
                ("ANTHROPIC_PROXY", Some("http://proxy-env.com".to_string())),
            ],
            || {
                let config = ClientConfig::from_env();
                assert!(config.is_ok(), "Should load config from environment");

                let config = config.unwrap();
                assert!(config.api_key.is_some());
                assert_eq!(config.base_url, Some("https://env-base.com".to_string()));
                assert_eq!(config.api_version, Some("2024-02-01".to_string()));
                assert_eq!(config.timeout, Duration::from_secs(120));
                assert_eq!(config.max_retries, 5);
                assert_eq!(config.proxy, Some("http://proxy-env.com".to_string()));
            },
        );
    }
}
