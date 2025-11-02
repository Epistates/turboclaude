//! Automatic retry logic with error recovery guidance
//!
//! Provides helpers for retrying operations with configurable backoff strategies
//! based on error recovery guidance.

use crate::error::{AgentError, ErrorRecovery};
use std::future::Future;
use std::pin::Pin;

/// Result type for retry operations
pub type Result<T> = std::result::Result<T, AgentError>;

/// Retry an operation with automatic backoff based on error recovery guidance
///
/// This function will retry the operation if:
/// 1. The error implements `ErrorRecovery` and `is_retriable()` returns true
/// 2. The number of attempts hasn't exceeded `max_retries`
/// 3. A backoff strategy is defined
///
/// # Example
///
/// ```no_run
/// # use turboclaudeagent::retry_with_recovery;
/// # use turboclaudeagent::error::{AgentError, Result};
/// # async fn example() -> Result<String> {
/// async fn get_data() -> Result<String> {
///     Ok("data".to_string())
/// }
///
/// let response = retry_with_recovery(
///     Box::new(|| Box::pin(get_data())),
///     Some(5) // max 5 attempts
/// ).await?;
/// # Ok(response)
/// # }
/// ```
pub async fn retry_with_recovery<'a, T: 'a>(
    mut operation: Box<
        dyn FnMut() -> Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>> + Send + 'a,
    >,
    max_attempts: Option<u32>,
) -> Result<T> {
    let max_attempts = max_attempts.unwrap_or(1);
    let mut attempt = 0;

    loop {
        attempt += 1;

        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                // Check if we should retry
                if !err.is_retriable() || attempt >= max_attempts {
                    return Err(err);
                }

                // Log the error and suggested action
                eprintln!("Attempt {}/{} failed: {}", attempt, max_attempts, err);
                eprintln!("  → {}", err.suggested_action());

                // Calculate backoff delay
                let backoff = err.backoff_strategy();
                if let Some(delay) = backoff.delay_for_attempt(attempt) {
                    eprintln!("  Waiting {:?} before retry...", delay);
                    tokio::time::sleep(delay).await;
                } else {
                    // No backoff strategy means don't retry
                    return Err(err);
                }
            }
        }
    }
}

/// Simplified retry helper that uses error's default retry policy
///
/// This is a convenience function that automatically uses:
/// - The error's `max_retries()` value for attempt limit
/// - The error's `backoff_strategy()` for delay
///
/// # Example
///
/// ```no_run
/// # use turboclaudeagent::retry;
/// # use turboclaudeagent::error::{AgentError, Result};
/// # async fn example() -> Result<String> {
/// async fn get_data() -> Result<String> {
///     Ok("data".to_string())
/// }
///
/// // Retries based on error's own policy (e.g., Transport errors retry 5x)
/// let response = retry(
///     Box::new(|| Box::pin(get_data()))
/// ).await?;
/// # Ok(response)
/// # }
/// ```
pub async fn retry<'a, T: 'a>(
    mut operation: Box<
        dyn FnMut() -> Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>> + Send + 'a,
    >,
) -> Result<T> {
    let mut attempt = 0;

    loop {
        attempt += 1;

        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                // Check if we should retry
                let max_retries = err.max_retries().unwrap_or(0);

                if !err.is_retriable() || attempt > max_retries {
                    eprintln!("Error (giving up): {}", err);
                    eprintln!("  → {}", err.suggested_action());
                    return Err(err);
                }

                // Log the error and suggested action
                eprintln!("Attempt {}/{} failed: {}", attempt, max_retries, err);
                eprintln!("  → {}", err.suggested_action());

                // Calculate backoff delay
                let backoff = err.backoff_strategy();
                if let Some(delay) = backoff.delay_for_attempt(attempt) {
                    eprintln!("  Waiting {:?} before retry...", delay);
                    tokio::time::sleep(delay).await;
                } else {
                    // No backoff strategy means don't retry
                    return Err(err);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_retry_succeeds_on_first_attempt() {
        let result: Result<i32> = retry_with_recovery(
            Box::new(|| Box::pin(async { Ok::<i32, AgentError>(42) })),
            Some(3),
        )
        .await;

        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn test_retry_succeeds_on_second_attempt() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result: Result<i32> = retry_with_recovery(
            Box::new(move || {
                let counter = counter_clone.clone();
                Box::pin(async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count == 0 {
                        Err(AgentError::Transport("connection lost".to_string()))
                    } else {
                        Ok::<i32, AgentError>(42)
                    }
                })
            }),
            Some(3),
        )
        .await;

        assert_eq!(result, Ok(42));
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_retry_gives_up_on_non_retriable_error() {
        let result: Result<i32> = retry_with_recovery(
            Box::new(|| {
                Box::pin(async {
                    Err::<i32, AgentError>(AgentError::Config("bad config".to_string()))
                })
            }),
            Some(3),
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_retry_respects_max_attempts() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result: Result<i32> = retry_with_recovery(
            Box::new(move || {
                let counter = counter_clone.clone();
                Box::pin(async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, AgentError>(AgentError::Transport("always fails".to_string()))
                })
            }),
            Some(3),
        )
        .await;

        assert!(result.is_err());
        // Should have attempted: 1 initial + 2 retries (capped at max_attempts - 1) + 1 final = 3
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_uses_error_policy() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result: Result<i32> = retry(Box::new(move || {
            let counter = counter_clone.clone();
            Box::pin(async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    // This error is retriable (Transport) and says max_retries = 5
                    Err::<i32, AgentError>(AgentError::Transport("failing".to_string()))
                } else {
                    Ok(42)
                }
            })
        }))
        .await;

        assert_eq!(result, Ok(42));
    }
}
