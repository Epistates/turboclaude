//! Exponential backoff with jitter.

use super::strategy::BackoffStrategy;
use async_trait::async_trait;
use std::error::Error;
use std::future::Future;
use std::time::Duration;

/// Exponential backoff strategy with configurable jitter.
///
/// Delays between retries increase exponentially: `initial_delay * multiplier^attempt`,
/// capped at `max_delay`. Jitter is added to prevent thundering herd problems.
///
/// # Mathematical Formula
///
/// For attempt `n` (0-indexed after first failure):
/// ```text
/// base_delay = initial_delay * (multiplier ^ n)
/// capped_delay = min(base_delay, max_delay)
/// jitter_range = capped_delay * jitter
/// final_delay = capped_delay + random(-jitter_range/2, +jitter_range/2)
/// ```
///
/// # Examples
///
/// ```rust
/// use turboclaude_core::retry::{BackoffStrategy, ExponentialBackoff};
/// use std::time::Duration;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Default configuration (max_retries=3, initial=100ms, max=60s, multiplier=2.0, jitter=0.1)
/// let backoff = ExponentialBackoff::default();
///
/// // Custom configuration
/// let backoff = ExponentialBackoff::builder()
///     .max_retries(5)
///     .initial_delay(Duration::from_millis(100))
///     .max_delay(Duration::from_secs(30))
///     .multiplier(2.0)
///     .jitter(0.1)
///     .build();
///
/// let result = backoff.execute(|| async {
///     // Your operation here
///     Ok::<_, std::io::Error>(42)
/// }).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Performance Characteristics
///
/// - **Memory**: O(1) - no allocations during retry loop
/// - **CPU**: O(1) per retry - simple arithmetic + one random number generation
/// - **I/O**: Sleeps between retries using `tokio::time::sleep`
#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    max_retries: u32,
    initial_delay: Duration,
    max_delay: Duration,
    multiplier: f64,
    jitter: f64,
}

impl ExponentialBackoff {
    /// Create a new builder for configuring exponential backoff.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use turboclaude_core::retry::ExponentialBackoff;
    /// use std::time::Duration;
    ///
    /// let backoff = ExponentialBackoff::builder()
    ///     .max_retries(5)
    ///     .initial_delay(Duration::from_millis(100))
    ///     .build();
    /// ```
    pub fn builder() -> ExponentialBackoffBuilder {
        ExponentialBackoffBuilder::default()
    }
}

impl Default for ExponentialBackoff {
    /// Create an exponential backoff with sensible defaults.
    ///
    /// Defaults:
    /// - `max_retries`: 3
    /// - `initial_delay`: 100ms
    /// - `max_delay`: 60s
    /// - `multiplier`: 2.0 (doubles each time)
    /// - `jitter`: 0.1 (10% randomization)
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(60),
            multiplier: 2.0,
            jitter: 0.1,
        }
    }
}

#[async_trait]
impl BackoffStrategy for ExponentialBackoff {
    async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: Future<Output = Result<T, E>> + Send,
        T: Send,
        E: Error + Send + Sync + 'static,
    {
        let mut attempt = 0;
        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(err) if !self.should_retry(&err, attempt) => return Err(err),
                Err(err) if attempt >= self.max_retries => return Err(err),
                Err(_) => {
                    if let Some(delay) = self.next_delay(attempt) {
                        tokio::time::sleep(delay).await;
                    }
                    attempt += 1;
                }
            }
        }
    }

    fn next_delay(&self, attempt: u32) -> Option<Duration> {
        // Calculate base delay with exponential growth
        // Note: attempt 0 represents the delay before the first RETRY (after initial attempt fails)
        let base_delay = self.initial_delay.as_secs_f64() * self.multiplier.powi(attempt as i32);

        // Apply jitter if configured
        let jittered = if self.jitter > 0.0 {
            // Jitter is applied as: base * jitter * random(-1.0, +1.0)
            // This gives a range of [base * (1 - jitter), base * (1 + jitter)]
            let jitter_amount = base_delay * self.jitter * (rand::random::<f64>() - 0.5) * 2.0;
            base_delay + jitter_amount
        } else {
            base_delay
        };

        // Cap at max_delay
        Some(Duration::from_secs_f64(
            jittered.min(self.max_delay.as_secs_f64()),
        ))
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

/// Builder for configuring `ExponentialBackoff`.
///
/// Provides a fluent API for setting retry parameters.
///
/// # Examples
///
/// ```rust
/// use turboclaude_core::retry::ExponentialBackoff;
/// use std::time::Duration;
///
/// let backoff = ExponentialBackoff::builder()
///     .max_retries(5)
///     .initial_delay(Duration::from_millis(100))
///     .max_delay(Duration::from_secs(30))
///     .multiplier(2.0)
///     .jitter(0.1)
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct ExponentialBackoffBuilder {
    max_retries: Option<u32>,
    initial_delay: Option<Duration>,
    max_delay: Option<Duration>,
    multiplier: Option<f64>,
    jitter: Option<f64>,
}

impl ExponentialBackoffBuilder {
    /// Set the maximum number of retry attempts.
    ///
    /// Default: 3
    ///
    /// # Examples
    ///
    /// ```rust
    /// use turboclaude_core::retry::ExponentialBackoff;
    ///
    /// let backoff = ExponentialBackoff::builder()
    ///     .max_retries(5)
    ///     .build();
    /// ```
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Set the initial delay before the first retry.
    ///
    /// Default: 100ms
    ///
    /// # Examples
    ///
    /// ```rust
    /// use turboclaude_core::retry::ExponentialBackoff;
    /// use std::time::Duration;
    ///
    /// let backoff = ExponentialBackoff::builder()
    ///     .initial_delay(Duration::from_millis(500))
    ///     .build();
    /// ```
    pub fn initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = Some(delay);
        self
    }

    /// Set the maximum delay between retries.
    ///
    /// Default: 60s
    ///
    /// # Examples
    ///
    /// ```rust
    /// use turboclaude_core::retry::ExponentialBackoff;
    /// use std::time::Duration;
    ///
    /// let backoff = ExponentialBackoff::builder()
    ///     .max_delay(Duration::from_secs(30))
    ///     .build();
    /// ```
    pub fn max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = Some(delay);
        self
    }

    /// Set the exponential multiplier.
    ///
    /// Each retry delay is multiplied by this factor.
    ///
    /// Default: 2.0 (doubles each time)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use turboclaude_core::retry::ExponentialBackoff;
    ///
    /// let backoff = ExponentialBackoff::builder()
    ///     .multiplier(1.5)  // More gradual backoff
    ///     .build();
    /// ```
    pub fn multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = Some(multiplier);
        self
    }

    /// Set the jitter factor (0.0 to 1.0).
    ///
    /// Jitter adds randomness to prevent thundering herd. A jitter of 0.1
    /// means the delay can vary by ±10%.
    ///
    /// Default: 0.1
    ///
    /// # Examples
    ///
    /// ```rust
    /// use turboclaude_core::retry::ExponentialBackoff;
    ///
    /// let backoff = ExponentialBackoff::builder()
    ///     .jitter(0.2)  // 20% randomization
    ///     .build();
    /// ```
    pub fn jitter(mut self, jitter: f64) -> Self {
        self.jitter = Some(jitter.clamp(0.0, 1.0));
        self
    }

    /// Build the `ExponentialBackoff` instance.
    ///
    /// Uses default values for any unset parameters.
    pub fn build(self) -> ExponentialBackoff {
        ExponentialBackoff {
            max_retries: self.max_retries.unwrap_or(3),
            initial_delay: self.initial_delay.unwrap_or(Duration::from_millis(100)),
            max_delay: self.max_delay.unwrap_or(Duration::from_secs(60)),
            multiplier: self.multiplier.unwrap_or(2.0),
            jitter: self.jitter.unwrap_or(0.1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn test_exponential_delay_calculation() {
        let backoff = ExponentialBackoff {
            max_retries: 5,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
            jitter: 0.0, // No jitter for predictable tests
        };

        // Attempt 0: 100ms * 2^0 = 100ms
        assert_eq!(backoff.next_delay(0).unwrap(), Duration::from_millis(100));

        // Attempt 1: 100ms * 2^1 = 200ms
        assert_eq!(backoff.next_delay(1).unwrap(), Duration::from_millis(200));

        // Attempt 2: 100ms * 2^2 = 400ms
        assert_eq!(backoff.next_delay(2).unwrap(), Duration::from_millis(400));

        // Attempt 3: 100ms * 2^3 = 800ms
        assert_eq!(backoff.next_delay(3).unwrap(), Duration::from_millis(800));
    }

    #[test]
    fn test_max_delay_cap() {
        let backoff = ExponentialBackoff {
            max_retries: 100,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(5), // Cap at 5 seconds
            multiplier: 10.0,                  // Aggressive multiplier
            jitter: 0.0,
        };

        // After several attempts, should be capped at max_delay
        for attempt in 5..10 {
            let delay = backoff.next_delay(attempt).unwrap();
            assert!(
                delay <= Duration::from_secs(5),
                "Delay at attempt {} ({:?}) exceeded max_delay",
                attempt,
                delay
            );
        }
    }

    #[tokio::test]
    async fn test_retry_success_on_third_attempt() {
        let backoff = ExponentialBackoff::builder()
            .max_retries(5)
            .initial_delay(Duration::from_millis(1)) // Fast for testing
            .jitter(0.0)
            .build();

        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = Arc::clone(&attempts);

        let result = backoff
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

    #[tokio::test]
    async fn test_max_retries_exceeded() {
        let backoff = ExponentialBackoff::builder()
            .max_retries(2)
            .initial_delay(Duration::from_millis(1))
            .jitter(0.0)
            .build();

        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = Arc::clone(&attempts);

        let result = backoff
            .execute(|| {
                let attempts = Arc::clone(&attempts_clone);
                async move {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Err::<(), _>(std::io::Error::other("always fail"))
                }
            })
            .await;

        assert!(result.is_err());
        // Should try: initial attempt + 2 retries = 3 total
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_jitter_variation() {
        let backoff = ExponentialBackoff {
            max_retries: 10,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            multiplier: 2.0,
            jitter: 0.5, // 50% jitter
        };

        // Generate multiple delays for the same attempt
        let mut delays = Vec::new();
        for _ in 0..20 {
            delays.push(backoff.next_delay(0).unwrap());
        }

        // With 50% jitter, delays should be between 0.5s and 1.5s
        for delay in &delays {
            let millis = delay.as_millis();
            assert!(
                (500..=1500).contains(&millis),
                "Delay with 50% jitter should be in range [500ms, 1500ms], got {}ms",
                millis
            );
        }

        // Check that not all delays are identical (very unlikely with jitter)
        let all_same = delays.windows(2).all(|w| w[0] == w[1]);
        assert!(!all_same, "With randomization, delays should vary");
    }

    #[test]
    fn test_builder_defaults() {
        let backoff = ExponentialBackoff::builder().build();

        assert_eq!(backoff.max_retries, 3);
        assert_eq!(backoff.initial_delay, Duration::from_millis(100));
        assert_eq!(backoff.max_delay, Duration::from_secs(60));
        assert_eq!(backoff.multiplier, 2.0);
        assert_eq!(backoff.jitter, 0.1);
    }

    #[test]
    fn test_builder_custom_values() {
        let backoff = ExponentialBackoff::builder()
            .max_retries(5)
            .initial_delay(Duration::from_millis(200))
            .max_delay(Duration::from_secs(30))
            .multiplier(1.5)
            .jitter(0.2)
            .build();

        assert_eq!(backoff.max_retries, 5);
        assert_eq!(backoff.initial_delay, Duration::from_millis(200));
        assert_eq!(backoff.max_delay, Duration::from_secs(30));
        assert_eq!(backoff.multiplier, 1.5);
        assert_eq!(backoff.jitter, 0.2);
    }

    #[test]
    fn test_jitter_clamped() {
        // Jitter > 1.0 should be clamped to 1.0
        let backoff = ExponentialBackoff::builder().jitter(2.0).build();

        assert_eq!(backoff.jitter, 1.0);

        // Jitter < 0.0 should be clamped to 0.0
        let backoff = ExponentialBackoff::builder().jitter(-0.5).build();

        assert_eq!(backoff.jitter, 0.0);
    }

    #[tokio::test]
    async fn test_immediate_success() {
        let backoff = ExponentialBackoff::default();

        let result = backoff
            .execute(|| async { Ok::<_, std::io::Error>(42) })
            .await;

        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_custom_retry_predicate() {
        // Create a custom backoff that only retries "network" errors
        struct NetworkOnlyBackoff {
            inner: ExponentialBackoff,
        }

        #[async_trait]
        impl BackoffStrategy for NetworkOnlyBackoff {
            async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T, E>
            where
                F: Fn() -> Fut + Send + Sync,
                Fut: Future<Output = Result<T, E>> + Send,
                T: Send,
                E: Error + Send + Sync + 'static,
            {
                // Must implement own loop to use custom should_retry
                let mut attempt = 0;
                loop {
                    match operation().await {
                        Ok(result) => return Ok(result),
                        Err(err) if !self.should_retry(&err, attempt) => return Err(err),
                        Err(err) if attempt >= self.max_retries() => return Err(err),
                        Err(_) => {
                            if let Some(delay) = self.next_delay(attempt) {
                                tokio::time::sleep(delay).await;
                            }
                            attempt += 1;
                        }
                    }
                }
            }

            fn should_retry(&self, error: &dyn Error, _attempt: u32) -> bool {
                error.to_string().contains("network")
            }

            fn next_delay(&self, attempt: u32) -> Option<Duration> {
                self.inner.next_delay(attempt)
            }

            fn max_retries(&self) -> u32 {
                self.inner.max_retries()
            }
        }

        let backoff = NetworkOnlyBackoff {
            inner: ExponentialBackoff::builder()
                .max_retries(5)
                .initial_delay(Duration::from_millis(1))
                .build(),
        };

        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = Arc::clone(&attempts);

        // Should NOT retry "auth" errors
        let result = backoff
            .execute(|| {
                let attempts = Arc::clone(&attempts_clone);
                async move {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Err::<(), _>(std::io::Error::other("auth failed"))
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 1); // Only initial attempt

        // Reset counter
        attempts.store(0, Ordering::SeqCst);

        // Should retry "network" errors
        let result = backoff
            .execute(|| {
                let attempts = Arc::clone(&attempts_clone);
                async move {
                    let current = attempts.fetch_add(1, Ordering::SeqCst);
                    if current < 2 {
                        Err(std::io::Error::other("network error"))
                    } else {
                        Ok(42)
                    }
                }
            })
            .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 3); // Retried twice
    }
}
