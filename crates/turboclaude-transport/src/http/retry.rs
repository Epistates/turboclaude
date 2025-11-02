//! Retry policy for HTTP transport
//!
//! This module re-exports the universal retry abstraction from `turboclaude-core`
//! with HTTP-specific defaults and utilities.

use crate::error::TransportError;
use std::time::Duration;
pub use turboclaude_core::retry::{BackoffStrategy, ExponentialBackoff, ExponentialBackoffBuilder};

/// HTTP-specific retry policy with sensible defaults for network operations.
///
/// This is a wrapper around `ExponentialBackoff` configured for HTTP transport.
/// For custom retry logic, use `ExponentialBackoff::builder()` directly.
///
/// # Default Configuration
///
/// - `max_retries`: 3
/// - `initial_delay`: 500ms (optimized for network latency)
/// - `max_delay`: 60s
/// - `multiplier`: 2.0 (exponential backoff)
/// - `jitter`: 0.1 (10% randomization to prevent thundering herd)
///
/// # Examples
///
/// ```rust
/// use turboclaude_transport::http::RetryPolicy;
/// use std::time::Duration;
///
/// // Use default HTTP retry settings
/// let policy = RetryPolicy::default();
///
/// // Customize retry behavior
/// let custom = RetryPolicy::builder()
///     .max_retries(5)
///     .initial_delay(Duration::from_millis(100))
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    inner: ExponentialBackoff,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            inner: ExponentialBackoff::builder()
                .max_retries(3)
                .initial_delay(Duration::from_millis(500))
                .max_delay(Duration::from_secs(60))
                .multiplier(2.0)
                .jitter(0.1)
                .build(),
        }
    }
}

impl RetryPolicy {
    /// Create a new builder for configuring HTTP retry policy.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use turboclaude_transport::http::RetryPolicy;
    /// use std::time::Duration;
    ///
    /// let policy = RetryPolicy::builder()
    ///     .max_retries(5)
    ///     .initial_delay(Duration::from_secs(1))
    ///     .build();
    /// ```
    pub fn builder() -> RetryPolicyBuilder {
        RetryPolicyBuilder {
            inner: ExponentialBackoff::builder(),
        }
    }

    /// Check if a transport error should be retried.
    ///
    /// # HTTP Retry Logic
    ///
    /// Retryable errors:
    /// - Timeout errors
    /// - Connection errors (network failures)
    ///
    /// Non-retryable errors:
    /// - HTTP errors (status codes should be handled at application layer)
    /// - Serialization errors (will fail again)
    /// - I/O errors (typically fatal)
    /// - Process errors (subprocess-specific)
    ///
    /// # Parameters
    ///
    /// - `error`: The transport error to evaluate
    ///
    /// # Returns
    ///
    /// `true` if the error should be retried, `false` otherwise
    pub fn is_retryable(error: &TransportError) -> bool {
        match error {
            // Timeout and connection errors are retryable
            TransportError::Timeout => true,
            TransportError::Connection(_) => true,

            // HTTP errors are generally not retryable unless specific status codes
            // (status code handling should be at application layer with proper error types)
            TransportError::Http(_) => false,

            // Don't retry serialization, I/O, or process errors
            TransportError::Serialization(_) => false,
            TransportError::Io(_) => false,
            TransportError::Process(_) => false,

            // Don't retry other errors
            TransportError::Other(_) => false,
        }
    }

    /// Get the underlying ExponentialBackoff instance.
    ///
    /// This allows access to the full BackoffStrategy API.
    pub fn inner(&self) -> &ExponentialBackoff {
        &self.inner
    }

    /// Calculate delay for a given attempt number.
    ///
    /// This delegates to the underlying ExponentialBackoff.
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        self.inner.next_delay(attempt).unwrap_or(Duration::ZERO)
    }
}

// Implement BackoffStrategy by delegating to inner
#[async_trait::async_trait]
impl BackoffStrategy for RetryPolicy {
    async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<T, E>> + Send,
        T: Send,
        E: std::error::Error + Send + Sync + 'static,
    {
        self.inner.execute(operation).await
    }

    fn should_retry(&self, error: &dyn std::error::Error, attempt: u32) -> bool {
        self.inner.should_retry(error, attempt)
    }

    fn next_delay(&self, attempt: u32) -> Option<Duration> {
        self.inner.next_delay(attempt)
    }

    fn max_retries(&self) -> u32 {
        self.inner.max_retries()
    }
}

/// Builder for HTTP retry policies.
pub struct RetryPolicyBuilder {
    inner: ExponentialBackoffBuilder,
}

impl RetryPolicyBuilder {
    /// Set the maximum number of retry attempts.
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.inner = self.inner.max_retries(max_retries);
        self
    }

    /// Set the initial delay before the first retry.
    pub fn initial_delay(mut self, delay: Duration) -> Self {
        self.inner = self.inner.initial_delay(delay);
        self
    }

    /// Set the maximum delay between retries.
    pub fn max_delay(mut self, delay: Duration) -> Self {
        self.inner = self.inner.max_delay(delay);
        self
    }

    /// Set the exponential multiplier.
    pub fn multiplier(mut self, multiplier: f64) -> Self {
        self.inner = self.inner.multiplier(multiplier);
        self
    }

    /// Set the jitter factor (0.0 to 1.0).
    pub fn jitter(mut self, jitter: f64) -> Self {
        self.inner = self.inner.jitter(jitter);
        self
    }

    /// Build the retry policy.
    pub fn build(self) -> RetryPolicy {
        RetryPolicy {
            inner: self.inner.build(),
        }
    }
}

impl Default for RetryPolicyBuilder {
    fn default() -> Self {
        Self {
            inner: ExponentialBackoff::builder()
                .max_retries(3)
                .initial_delay(Duration::from_millis(500))
                .max_delay(Duration::from_secs(60))
                .multiplier(2.0)
                .jitter(0.1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries(), 3);
        assert!(RetryPolicy::is_retryable(&TransportError::Timeout));
        assert!(!RetryPolicy::is_retryable(&TransportError::Http(
            "400".to_string()
        )));
    }

    #[test]
    fn test_retry_policy_builder() {
        let policy = RetryPolicy::builder()
            .max_retries(5)
            .initial_delay(Duration::from_secs(1))
            .max_delay(Duration::from_secs(120))
            .multiplier(1.5)
            .build();

        assert_eq!(policy.max_retries(), 5);
    }

    #[test]
    fn test_is_retryable() {
        // Retryable errors
        assert!(RetryPolicy::is_retryable(&TransportError::Timeout));
        assert!(RetryPolicy::is_retryable(&TransportError::Connection(
            "network error".to_string()
        )));

        // Non-retryable errors
        assert!(!RetryPolicy::is_retryable(&TransportError::Http(
            "500".to_string()
        )));
        assert!(!RetryPolicy::is_retryable(&TransportError::Serialization(
            "parse error".to_string()
        )));
        assert!(!RetryPolicy::is_retryable(&TransportError::Io(
            std::io::Error::other("io error")
        )));
    }

    #[test]
    fn test_default_matches_old_behavior() {
        // Verify defaults match the old RetryPolicy implementation
        let policy = RetryPolicy::default();

        assert_eq!(policy.max_retries(), 3);

        // Check that delay calculation works (first delay should be around 500ms)
        let first_delay = policy.next_delay(0).unwrap();
        // With 10% jitter, should be between 450ms and 550ms
        assert!(first_delay.as_millis() >= 450 && first_delay.as_millis() <= 550);
    }

    #[test]
    fn test_calculate_delay_exponential() {
        let policy = RetryPolicy::default();

        let delay_0 = policy.calculate_delay(0).as_millis();
        let delay_1 = policy.calculate_delay(1).as_millis();
        let delay_2 = policy.calculate_delay(2).as_millis();

        // Each should be roughly growing (with jitter)
        assert!(delay_0 > 0);
        assert!(delay_1 > delay_0);
        assert!(delay_2 > delay_1);
    }

    #[tokio::test]
    async fn test_execute_with_retry() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicU32, Ordering};

        let policy = RetryPolicy::builder()
            .max_retries(3)
            .initial_delay(Duration::from_millis(1)) // Fast for testing
            .jitter(0.0)
            .build();

        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = Arc::clone(&attempts);

        let result = policy
            .execute(|| {
                let attempts = Arc::clone(&attempts_clone);
                async move {
                    let current = attempts.fetch_add(1, Ordering::SeqCst);
                    if current < 2 {
                        Err(std::io::Error::other("retry me"))
                    } else {
                        Ok(42)
                    }
                }
            })
            .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }
}
