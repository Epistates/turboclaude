//! Configuration for the Anthropic client

use http::HeaderMap;
use secrecy::SecretString;
use std::time::Duration;

/// Configuration for the Anthropic client.
///
/// This struct holds all the configuration options for creating a client,
/// similar to the Python SDK's configuration but adapted for Rust patterns.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// API key for authentication (preferred method)
    pub api_key: Option<SecretString>,

    /// Auth token for authentication (alternative to API key)
    pub auth_token: Option<SecretString>,

    /// Base URL for the API
    pub base_url: Option<String>,

    /// API version header value
    pub api_version: Option<String>,

    /// Default timeout for requests
    pub timeout: Duration,

    /// Maximum number of retries for failed requests
    pub max_retries: u32,

    /// Custom headers to include with every request
    pub default_headers: HeaderMap,

    /// HTTP proxy URL
    pub proxy: Option<String>,

    /// Connection pool configuration
    pub connection_pool: ConnectionPoolConfig,

    /// Rate limiting configuration
    pub rate_limit: Option<RateLimitConfig>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            auth_token: None,
            base_url: None,
            api_version: None,
            timeout: Duration::from_secs(600), // 10 minutes, matching Python SDK
            max_retries: 2,                    // Default to 2 retries like Python SDK
            default_headers: HeaderMap::new(),
            proxy: None,
            connection_pool: ConnectionPoolConfig::default(),
            rate_limit: None,
        }
    }
}

impl ClientConfig {
    /// Create a new configuration with an API key.
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(SecretString::new(api_key.into().into_boxed_str())),
            ..Default::default()
        }
    }

    /// Create a new configuration with an auth token.
    pub fn with_auth_token(auth_token: impl Into<String>) -> Self {
        Self {
            auth_token: Some(SecretString::new(auth_token.into().into_boxed_str())),
            ..Default::default()
        }
    }

    /// Load configuration from environment variables.
    ///
    /// This will look for:
    /// - `ANTHROPIC_API_KEY` or `ANTHROPIC_AUTH_TOKEN` for authentication
    /// - `ANTHROPIC_BASE_URL` for the API base URL
    /// - `ANTHROPIC_API_VERSION` for the API version
    /// - `ANTHROPIC_TIMEOUT` for request timeout (in seconds)
    /// - `ANTHROPIC_MAX_RETRIES` for maximum retry attempts
    /// - `ANTHROPIC_PROXY` for HTTP proxy
    #[cfg(feature = "env")]
    pub fn from_env() -> Result<Self, crate::error::Error> {
        use std::env;

        let mut config = Self::default();

        // Authentication
        if let Ok(api_key) = env::var("ANTHROPIC_API_KEY") {
            config.api_key = Some(SecretString::new(api_key.into_boxed_str()));
        } else if let Ok(auth_token) = env::var("ANTHROPIC_AUTH_TOKEN") {
            config.auth_token = Some(SecretString::new(auth_token.into_boxed_str()));
        }

        // Base URL
        if let Ok(base_url) = env::var("ANTHROPIC_BASE_URL") {
            config.base_url = Some(base_url);
        }

        // API version
        if let Ok(api_version) = env::var("ANTHROPIC_API_VERSION") {
            config.api_version = Some(api_version);
        }

        // Timeout
        if let Ok(timeout_str) = env::var("ANTHROPIC_TIMEOUT")
            && let Ok(timeout_secs) = timeout_str.parse::<u64>()
        {
            config.timeout = Duration::from_secs(timeout_secs);
        }

        // Max retries
        if let Ok(max_retries_str) = env::var("ANTHROPIC_MAX_RETRIES")
            && let Ok(max_retries) = max_retries_str.parse::<u32>()
        {
            config.max_retries = max_retries;
        }

        // Proxy
        if let Ok(proxy) = env::var("ANTHROPIC_PROXY") {
            config.proxy = Some(proxy);
        }

        Ok(config)
    }

    /// Merge this configuration with another, with the other taking precedence.
    pub fn merge(mut self, other: ClientConfig) -> Self {
        if other.api_key.is_some() {
            self.api_key = other.api_key;
        }
        if other.auth_token.is_some() {
            self.auth_token = other.auth_token;
        }
        if other.base_url.is_some() {
            self.base_url = other.base_url;
        }
        if other.api_version.is_some() {
            self.api_version = other.api_version;
        }
        if other.timeout != Duration::from_secs(600) {
            self.timeout = other.timeout;
        }
        if other.max_retries != 2 {
            self.max_retries = other.max_retries;
        }
        if !other.default_headers.is_empty() {
            for (key, value) in other.default_headers.iter() {
                self.default_headers.insert(key.clone(), value.clone());
            }
        }
        if other.proxy.is_some() {
            self.proxy = other.proxy;
        }
        if other.rate_limit.is_some() {
            self.rate_limit = other.rate_limit;
        }

        self
    }
}

/// Configuration for HTTP connection pooling.
#[derive(Debug, Clone)]
pub struct ConnectionPoolConfig {
    /// Maximum number of idle connections per host
    pub max_idle_per_host: usize,

    /// Idle connection timeout
    pub idle_timeout: Duration,

    /// TCP keep-alive interval
    pub tcp_keepalive: Option<Duration>,

    /// Enable HTTP/2
    pub http2: bool,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_idle_per_host: 10,
            idle_timeout: Duration::from_secs(90),
            tcp_keepalive: Some(Duration::from_secs(60)),
            http2: true,
        }
    }
}

/// Configuration for rate limiting.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per second
    pub requests_per_second: f64,

    /// Burst size for token bucket
    pub burst_size: u32,

    /// Whether to automatically retry on rate limit errors
    pub auto_retry: bool,

    /// Maximum wait time for rate limit retry
    pub max_retry_wait: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10.0,
            burst_size: 20,
            auto_retry: true,
            max_retry_wait: Duration::from_secs(60),
        }
    }
}

/// Builder for creating ClientConfig with a fluent API.
#[derive(Debug, Default)]
pub struct ClientConfigBuilder {
    config: ClientConfig,
}

impl ClientConfigBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the API key.
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.config.api_key = Some(SecretString::new(api_key.into().into_boxed_str()));
        self
    }

    /// Set the auth token.
    pub fn auth_token(mut self, auth_token: impl Into<String>) -> Self {
        self.config.auth_token = Some(SecretString::new(auth_token.into().into_boxed_str()));
        self
    }

    /// Set the base URL.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.config.base_url = Some(base_url.into());
        self
    }

    /// Set the API version.
    pub fn api_version(mut self, api_version: impl Into<String>) -> Self {
        self.config.api_version = Some(api_version.into());
        self
    }

    /// Set the request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Set the maximum number of retries.
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.config.max_retries = max_retries;
        self
    }

    /// Add a default header.
    ///
    /// # Errors
    ///
    /// Returns an error if the header name or value is invalid according to HTTP specifications.
    pub fn default_header(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> crate::Result<Self> {
        let key_str = key.into();
        let value_str = value.into();

        let key: http::HeaderName = key_str
            .parse()
            .map_err(|_| crate::Error::InvalidHeaderName(key_str.clone()))?;
        let value: http::HeaderValue = value_str
            .parse()
            .map_err(|_| crate::Error::InvalidHeaderValue(value_str.clone()))?;

        self.config.default_headers.insert(key, value);
        Ok(self)
    }

    /// Set the HTTP proxy.
    pub fn proxy(mut self, proxy: impl Into<String>) -> Self {
        self.config.proxy = Some(proxy.into());
        self
    }

    /// Enable rate limiting with default configuration.
    pub fn with_rate_limiting(mut self) -> Self {
        self.config.rate_limit = Some(RateLimitConfig::default());
        self
    }

    /// Set custom rate limiting configuration.
    pub fn rate_limit(mut self, config: RateLimitConfig) -> Self {
        self.config.rate_limit = Some(config);
        self
    }

    /// Set connection pool configuration.
    pub fn connection_pool(mut self, config: ConnectionPoolConfig) -> Self {
        self.config.connection_pool = config;
        self
    }

    /// Build the configuration.
    pub fn build(self) -> ClientConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ClientConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(600));
        assert_eq!(config.max_retries, 2);
        assert!(config.api_key.is_none());
        assert!(config.auth_token.is_none());
    }

    #[test]
    fn test_config_with_api_key() {
        let config = ClientConfig::with_api_key("test-key");
        assert!(config.api_key.is_some());
        assert!(config.auth_token.is_none());
    }

    #[test]
    fn test_config_builder() {
        let config = ClientConfigBuilder::new()
            .api_key("test-key")
            .base_url("https://example.com")
            .timeout(Duration::from_secs(30))
            .max_retries(5)
            .with_rate_limiting()
            .build();

        assert!(config.api_key.is_some());
        assert_eq!(config.base_url, Some("https://example.com".to_string()));
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_retries, 5);
        assert!(config.rate_limit.is_some());
    }

    #[test]
    fn test_config_merge() {
        let config1 = ClientConfig::with_api_key("key1");
        let config2 = ClientConfigBuilder::new()
            .base_url("https://example.com")
            .timeout(Duration::from_secs(30))
            .build();

        let merged = config1.merge(config2);
        assert!(merged.api_key.is_some());
        assert_eq!(merged.base_url, Some("https://example.com".to_string()));
        assert_eq!(merged.timeout, Duration::from_secs(30));
    }
}
