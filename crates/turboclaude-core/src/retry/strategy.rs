//! Retry strategies with exponential backoff, jitter, and circuit breaking.

use async_trait::async_trait;
use std::error::Error;
use std::future::Future;
use std::time::Duration;

/// A strategy for retrying failed operations with backoff.
///
/// Implementations determine when to retry, how long to wait between attempts,
/// and when to give up.
///
/// # Design Philosophy
///
/// This trait provides a universal abstraction for retry logic, consolidating
/// duplicated retry implementations across the TurboClaude ecosystem. It supports:
///
/// - Generic operation types (any async function)
/// - Custom retry predicates (error-specific logic)
/// - Multiple backoff strategies (exponential, linear, circuit breakers)
/// - Jitter to prevent thundering herd problems
///
/// # Examples
///
/// ```rust
/// use turboclaude_core::retry::{BackoffStrategy, ExponentialBackoff};
/// use std::time::Duration;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let backoff = ExponentialBackoff::builder()
///     .max_retries(3)
///     .initial_delay(Duration::from_millis(100))
///     .build();
///
/// let result = backoff.execute(|| async {
///     // Your async operation here
///     Ok::<_, std::io::Error>(42)
/// }).await?;
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait BackoffStrategy: Send + Sync {
    /// Execute an operation with retry logic.
    ///
    /// The operation is called repeatedly until it succeeds, a non-retryable
    /// error occurs, or the maximum number of retries is exceeded.
    ///
    /// # Type Parameters
    /// - `F`: Function that returns a future
    /// - `Fut`: The future returned by the function
    /// - `T`: Success type
    /// - `E`: Error type (must implement `std::error::Error`)
    ///
    /// # Returns
    /// - `Ok(T)`: The successful result
    /// - `Err(E)`: The final error after all retries exhausted
    ///
    /// # Examples
    ///
    /// ```rust
    /// use turboclaude_core::retry::{BackoffStrategy, ExponentialBackoff};
    /// use std::time::Duration;
    /// use std::sync::atomic::{AtomicU32, Ordering};
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let backoff = ExponentialBackoff::builder()
    ///     .max_retries(3)
    ///     .initial_delay(Duration::from_millis(100))
    ///     .build();
    ///
    /// let attempts = Arc::new(AtomicU32::new(0));
    /// let result = backoff.execute(|| {
    ///     let attempts = Arc::clone(&attempts);
    ///     async move {
    ///         let current = attempts.fetch_add(1, Ordering::SeqCst);
    ///         if current < 2 {
    ///             Err(std::io::Error::new(std::io::ErrorKind::Other, "retry me"))
    ///         } else {
    ///             Ok(42)
    ///         }
    ///     }
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: Future<Output = Result<T, E>> + Send,
        T: Send,
        E: Error + Send + Sync + 'static;

    /// Determine if an error is retryable.
    ///
    /// Default implementation returns `true` for all errors. Override this
    /// to implement custom retry logic (e.g., only retry network errors).
    ///
    /// # Parameters
    /// - `error`: The error to evaluate
    /// - `attempt`: The current attempt number (0-indexed)
    ///
    /// # Returns
    /// - `true`: The error should be retried
    /// - `false`: The error should not be retried (fail immediately)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use turboclaude_core::retry::{BackoffStrategy, ExponentialBackoff};
    /// use std::error::Error;
    ///
    /// struct NetworkOnlyBackoff {
    ///     inner: ExponentialBackoff,
    /// }
    ///
    /// #[async_trait::async_trait]
    /// impl BackoffStrategy for NetworkOnlyBackoff {
    ///     async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T, E>
    ///     where
    ///         F: Fn() -> Fut + Send + Sync,
    ///         Fut: std::future::Future<Output = Result<T, E>> + Send,
    ///         T: Send,
    ///         E: Error + Send + Sync + 'static,
    ///     {
    ///         // Must implement own loop to use custom should_retry
    ///         let mut attempt = 0;
    ///         loop {
    ///             match operation().await {
    ///                 Ok(result) => return Ok(result),
    ///                 Err(err) if !self.should_retry(&err, attempt) => return Err(err),
    ///                 Err(err) if attempt >= self.max_retries() => return Err(err),
    ///                 Err(_) => {
    ///                     if let Some(delay) = self.next_delay(attempt) {
    ///                         tokio::time::sleep(delay).await;
    ///                     }
    ///                     attempt += 1;
    ///                 }
    ///             }
    ///         }
    ///     }
    ///
    ///     fn should_retry(&self, error: &dyn Error, _attempt: u32) -> bool {
    ///         // Only retry if error message contains "network"
    ///         error.to_string().to_lowercase().contains("network")
    ///     }
    ///
    ///     fn next_delay(&self, attempt: u32) -> Option<std::time::Duration> {
    ///         self.inner.next_delay(attempt)
    ///     }
    ///
    ///     fn max_retries(&self) -> u32 {
    ///         self.inner.max_retries()
    ///     }
    /// }
    /// ```
    fn should_retry(&self, error: &dyn Error, attempt: u32) -> bool {
        let _ = (error, attempt);
        true
    }

    /// Calculate the delay before the next retry attempt.
    ///
    /// # Parameters
    /// - `attempt`: The current attempt number (0-indexed)
    ///
    /// # Returns
    /// - `Some(Duration)`: Wait this long before the next retry
    /// - `None`: No more retries should be attempted
    ///
    /// # Notes
    ///
    /// This method is called AFTER a failure and BEFORE sleeping. The first
    /// attempt (attempt=0) will call `next_delay(0)` before the second try.
    fn next_delay(&self, attempt: u32) -> Option<Duration>;

    /// Get the maximum number of retry attempts.
    ///
    /// # Returns
    /// The maximum number of times the operation will be retried after the
    /// initial attempt. For example, if `max_retries() == 3`, the operation
    /// will be attempted up to 4 times total (1 initial + 3 retries).
    fn max_retries(&self) -> u32;
}

/// Builder for configuring retry strategies.
///
/// This is a generic builder that wraps any `BackoffStrategy` implementation.
/// Most implementations provide their own builder (e.g., `ExponentialBackoffBuilder`).
pub struct BackoffBuilder<S> {
    strategy: S,
}

impl<S> BackoffBuilder<S> {
    /// Create a new builder wrapping a strategy.
    pub fn new(strategy: S) -> Self {
        Self { strategy }
    }

    /// Build the final strategy.
    pub fn build(self) -> S {
        self.strategy
    }
}
