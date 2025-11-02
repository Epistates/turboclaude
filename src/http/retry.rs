//! Retry logic for HTTP requests

use std::time::Duration;
use backoff::{ExponentialBackoff, ExponentialBackoffBuilder};
use backoff::backoff::Backoff;

/// Configuration for retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,

    /// Initial retry delay
    pub initial_interval: Duration,

    /// Maximum retry delay
    pub max_interval: Duration,

    /// Exponential backoff multiplier
    pub multiplier: f64,

    /// Randomization factor for jitter
    pub randomization_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 2,
            initial_interval: Duration::from_secs(1),
            max_interval: Duration::from_secs(60),
            multiplier: 2.0,
            randomization_factor: 0.1,
        }
    }
}

impl RetryConfig {
    /// Create an exponential backoff instance from this config.
    pub fn to_backoff(&self) -> ExponentialBackoff {
        ExponentialBackoffBuilder::new()
            .with_initial_interval(self.initial_interval)
            .with_max_interval(self.max_interval)
            .with_multiplier(self.multiplier)
            .with_randomization_factor(self.randomization_factor)
            .with_max_elapsed_time(None)
            .build()
    }
}

/// Determine if an error is retryable.
pub fn is_retryable_error(error: &crate::error::Error) -> bool {
    error.is_retryable()
}

/// Calculate retry delay based on error and attempt number.
pub fn calculate_retry_delay(
    error: &crate::error::Error,
    attempt: u32,
    config: &RetryConfig,
) -> Option<Duration> {
    // First check if the error has a specific retry-after
    if let Some(delay) = error.retry_after() {
        return Some(delay);
    }

    // Otherwise use exponential backoff
    if is_retryable_error(error) && attempt < config.max_retries {
        let mut backoff = config.to_backoff();
        for _ in 0..attempt {
            backoff.next_backoff();
        }
        backoff.next_backoff()
    } else {
        None
    }
}