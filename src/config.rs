//! Configuration for the Anthropic client

use std::time::Duration;
use http::HeaderMap;
use secrecy::SecretString;

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

    /// HTTP proxy URL for routing requests through a proxy server.
    ///
    /// Supports HTTP, HTTPS, and SOCKS5 proxies. Examples:
    /// - `http://proxy.example.com:8080` - HTTP proxy
    /// - `https://proxy.example.com:8443` - HTTPS proxy with TLS
    /// - `socks5://proxy.example.com:1080` - SOCKS5 proxy
    ///
    /// You can also set this via the `ANTHROPIC_PROXY` environment variable.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use anthropic::ClientConfig;
    /// let config = ClientConfig::default()
    ///     .with_proxy("http://proxy.example.com:8080");
    /// ```
    pub proxy: Option<String>,

    /// Connection pool configuration for optimizing HTTP/2 connections.
    ///
    /// Controls connection reuse, timeouts, and keep-alive settings for
    /// improved performance when making many requests.
    pub connection_pool: ConnectionPoolConfig,

    /// Rate limiting configuration for controlling request throughput.
    ///
    /// When enabled, limits the number of requests per second, preventing
    /// rate limit errors and reducing server load. Uses token bucket algorithm.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use anthropic::{ClientConfig, config::RateLimitConfig};
    /// # use std::time::Duration;
    /// let rate_limit = RateLimitConfig {
    ///     requests_per_second: 50.0,  // 50 requests per second
    ///     burst_size: 100,             // Allow burst of 100
    ///     auto_retry: true,            // Retry on rate limit
    ///     max_retry_wait: Duration::from_secs(60),
    /// };
    /// let config = ClientConfig::default()
    ///     .with_rate_limiting(rate_limit);
    /// ```
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
            max_retries: 2, // Default to 2 retries like Python SDK
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

    /// Set an HTTP proxy for routing requests.
    ///
    /// Supports HTTP, HTTPS, and SOCKS5 proxies.
    ///
    /// # Arguments
    ///
    /// * `proxy` - Proxy URL (e.g., `http://proxy.example.com:8080`)
    pub fn with_proxy(mut self, proxy: impl Into<String>) -> Self {
        self.proxy = Some(proxy.into());
        self
    }

    /// Set rate limiting configuration.
    ///
    /// Enables client-side rate limiting to prevent overwhelming the API and
    /// receiving rate limit responses.
    pub fn with_rate_limiting(mut self, config: RateLimitConfig) -> Self {
        self.rate_limit = Some(config);
        self
    }

    /// Set connection pool configuration for HTTP/2 optimization.
    pub fn with_connection_pool(mut self, config: ConnectionPoolConfig) -> Self {
        self.connection_pool = config;
        self
    }

    /// Load configuration from environment variables.
    ///
    /// This will look for:
    /// - `ANTHROPIC_API_KEY` or `ANTHROPIC_AUTH_TOKEN` for authentication
    /// - `ANTHROPIC_BASE_URL` for the API base URL
    /// - `ANTHROPIC_API_VERSION` for the API version
    /// - `ANTHROPIC_TIMEOUT` for request timeout (in seconds, must be a valid u64)
    /// - `ANTHROPIC_MAX_RETRIES` for maximum retry attempts (must be a valid u32)
    /// - `ANTHROPIC_PROXY` for HTTP proxy
    ///
    /// # Errors
    ///
    /// Returns an error if `ANTHROPIC_TIMEOUT` or `ANTHROPIC_MAX_RETRIES` environment
    /// variables are set but contain invalid values that cannot be parsed as numbers.
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

        // Timeout - return error if invalid
        if let Ok(timeout_str) = env::var("ANTHROPIC_TIMEOUT") {
            let timeout_secs = timeout_str.parse::<u64>()
                .map_err(|_| crate::error::Error::InvalidRequest(
                    format!("ANTHROPIC_TIMEOUT must be a valid number of seconds, got: '{}'", timeout_str)
                ))?;
            config.timeout = Duration::from_secs(timeout_secs);
        }

        // Max retries - return error if invalid
        if let Ok(max_retries_str) = env::var("ANTHROPIC_MAX_RETRIES") {
            let max_retries = max_retries_str.parse::<u32>()
                .map_err(|_| crate::error::Error::InvalidRequest(
                    format!("ANTHROPIC_MAX_RETRIES must be a valid number, got: '{}'", max_retries_str)
                ))?;
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

/// Configuration for HTTP connection pooling and optimization.
///
/// Controls how the underlying HTTP client manages connections to the API server,
/// including connection reuse, timeouts, and protocol settings for optimal performance.
///
/// # Default Values
///
/// - `max_idle_per_host`: 10 connections
/// - `idle_timeout`: 90 seconds
/// - `tcp_keepalive`: 60 seconds
/// - `http2`: true (uses HTTP/2 for multiplexing)
///
/// # Examples
///
/// ```rust
/// # use anthropic::config::ConnectionPoolConfig;
/// # use std::time::Duration;
/// // High-throughput scenario: more connections and longer idle timeout
/// let pool_config = ConnectionPoolConfig {
///     max_idle_per_host: 50,
///     idle_timeout: Duration::from_secs(300),
///     tcp_keepalive: Some(Duration::from_secs(120)),
///     http2: true,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ConnectionPoolConfig {
    /// Maximum number of idle connections to keep per host.
    ///
    /// Default: 10. Higher values (50-100) are good for high-throughput scenarios,
    /// while lower values (1-5) are better for resource-constrained environments.
    pub max_idle_per_host: usize,

    /// How long an idle connection remains in the pool before being closed.
    ///
    /// Default: 90 seconds. Longer timeouts (300-600s) reduce reconnection overhead
    /// but use more memory. Shorter timeouts (30-60s) free resources faster.
    pub idle_timeout: Duration,

    /// TCP keep-alive interval for detecting stale connections.
    ///
    /// Default: 60 seconds. Helps detect broken connections and prevents firewall
    /// timeouts on idle connections. Set to None to disable keep-alive.
    pub tcp_keepalive: Option<Duration>,

    /// Enable HTTP/2 protocol for connection multiplexing.
    ///
    /// Default: true. HTTP/2 allows multiple concurrent requests over a single
    /// connection, significantly improving throughput. Disable if experiencing
    /// compatibility issues with proxies or load balancers.
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

/// Configuration for client-side rate limiting.
///
/// Implements token bucket algorithm to control request throughput and prevent
/// hitting API rate limits. When enabled, the client will throttle requests
/// if necessary to stay within the configured limit.
///
/// # Default Values
///
/// - `requests_per_second`: 10.0 (conservative, safe for most use cases)
/// - `burst_size`: 20 (allows short bursts of up to 20 requests)
/// - `auto_retry`: true (automatically retry rate-limited requests)
/// - `max_retry_wait`: 60 seconds
///
/// # Examples
///
/// ```rust
/// # use anthropic::config::RateLimitConfig;
/// # use std::time::Duration;
/// // API allows 100 requests per minute with burst capacity
/// let rate_limit = RateLimitConfig {
///     requests_per_second: 100.0 / 60.0,  // ~1.67 req/s
///     burst_size: 50,                      // Allow bursts of 50
///     auto_retry: true,
///     max_retry_wait: Duration::from_secs(120),
/// };
/// ```
///
/// # Token Bucket Algorithm
///
/// The token bucket algorithm works as follows:
/// - Tokens accumulate at `requests_per_second` rate
/// - Each request consumes 1 token
/// - Up to `burst_size` tokens can be stored
/// - Requests are delayed if insufficient tokens are available
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per second allowed.
    ///
    /// Examples:
    /// - 10.0: 10 requests per second (conservative default)
    /// - 50.0: 50 requests per second (suitable for high-volume)
    /// - 1.666: ~100 requests per minute
    pub requests_per_second: f64,

    /// Burst size for token bucket algorithm.
    ///
    /// Allows temporary exceed of `requests_per_second` for short bursts.
    /// Default: 20. Higher values (50-100) allow larger bursts,
    /// lower values (5-10) enforce stricter limits.
    pub burst_size: u32,

    /// Whether to automatically retry on rate limit errors (429).
    ///
    /// When true, requests that hit rate limits will be automatically retried
    /// after a delay. The delay respects the `Retry-After` header from the API.
    /// Default: true (recommended for most applications).
    pub auto_retry: bool,

    /// Maximum time to wait when retrying rate-limited requests.
    ///
    /// Prevents indefinite waiting if the API is under heavy load.
    /// Default: 60 seconds. Increase for long-running batch operations,
    /// decrease for interactive applications.
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
    pub fn default_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let key: http::HeaderName = key.into().parse().expect("Invalid header name");
        let value: http::HeaderValue = value.into().parse().expect("Invalid header value");
        self.config.default_headers.insert(key, value);
        self
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