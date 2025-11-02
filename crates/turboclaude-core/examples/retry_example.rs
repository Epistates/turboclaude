//! Example: Using the BackoffStrategy trait for retry logic
//!
//! This example demonstrates:
//! 1. Simple retry with exponential backoff
//! 2. Custom retry predicate (only retry network errors)
//! 3. Jitter impact (run multiple times to see variance)
//!
//! Run with:
//! ```bash
//! cargo run -p turboclaude-core --example retry_example
//! ```

use async_trait::async_trait;
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};
use turboclaude_core::prelude::*;

/// A simulated API that fails the first few times
struct UnreliableApi {
    attempts: Arc<AtomicU32>,
    fail_count: u32,
}

impl UnreliableApi {
    fn new(fail_count: u32) -> Self {
        Self {
            attempts: Arc::new(AtomicU32::new(0)),
            fail_count,
        }
    }

    async fn call(&self) -> Result<String, std::io::Error> {
        let attempt = self.attempts.fetch_add(1, Ordering::SeqCst);

        if attempt < self.fail_count {
            println!(
                "  Attempt {}: FAILED (simulating transient error)",
                attempt + 1
            );
            Err(std::io::Error::other(format!(
                "Transient error on attempt {}",
                attempt + 1
            )))
        } else {
            println!("  Attempt {}: SUCCESS", attempt + 1);
            Ok("API response data".to_string())
        }
    }

    #[allow(dead_code)]
    fn reset(&self) {
        self.attempts.store(0, Ordering::SeqCst);
    }

    fn total_attempts(&self) -> u32 {
        self.attempts.load(Ordering::SeqCst)
    }
}

/// Example 1: Simple retry with exponential backoff
async fn example_simple_retry() -> Result<(), Box<dyn Error>> {
    println!("\n=== Example 1: Simple Retry with Exponential Backoff ===\n");

    let backoff = ExponentialBackoff::builder()
        .max_retries(3)
        .initial_delay(Duration::from_millis(100))
        .multiplier(2.0)
        .jitter(0.0) // No jitter for predictable output
        .build();

    let api = UnreliableApi::new(2); // Fail first 2 attempts

    println!("Calling unreliable API (will fail 2 times before succeeding)...");
    let start = Instant::now();

    let result = backoff
        .execute(|| {
            let api = &api;
            async move { api.call().await }
        })
        .await?;

    let elapsed = start.elapsed();

    println!("\nResult: {}", result);
    println!("Total attempts: {}", api.total_attempts());
    println!("Total time: {:?}", elapsed);
    println!("Expected delays: 0ms (attempt 1) + 100ms + 200ms = ~300ms");

    Ok(())
}

/// Example 2: Custom retry predicate (only retry network errors)
async fn example_custom_predicate() -> Result<(), Box<dyn Error>> {
    println!("\n=== Example 2: Custom Retry Predicate (Network Errors Only) ===\n");

    // Create a custom backoff that only retries "network" errors
    struct NetworkOnlyBackoff {
        inner: ExponentialBackoff,
    }

    #[async_trait]
    impl BackoffStrategy for NetworkOnlyBackoff {
        async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T, E>
        where
            F: Fn() -> Fut + Send + Sync,
            Fut: std::future::Future<Output = Result<T, E>> + Send,
            T: Send,
            E: Error + Send + Sync + 'static,
        {
            // Must implement own loop to use custom should_retry
            let mut attempt = 0;
            loop {
                match operation().await {
                    Ok(result) => return Ok(result),
                    Err(err) if !self.should_retry(&err, attempt) => {
                        println!("  Error is NOT retryable: {}", err);
                        return Err(err);
                    }
                    Err(err) if attempt >= self.max_retries() => {
                        println!("  Max retries ({}) exceeded", self.max_retries());
                        return Err(err);
                    }
                    Err(_) => {
                        println!("  Error is retryable, retrying...");
                        if let Some(delay) = self.next_delay(attempt) {
                            tokio::time::sleep(delay).await;
                        }
                        attempt += 1;
                    }
                }
            }
        }

        fn should_retry(&self, error: &dyn Error, _attempt: u32) -> bool {
            // Only retry if error message contains "network"
            error.to_string().to_lowercase().contains("network")
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
            .max_retries(3)
            .initial_delay(Duration::from_millis(10))
            .jitter(0.0)
            .build(),
    };

    // Test 1: Non-network error should NOT retry
    println!("Test 1: Auth error (should NOT retry)");
    let result = backoff
        .execute(|| async {
            Err::<(), _>(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "auth failed",
            ))
        })
        .await;
    assert!(result.is_err());

    // Test 2: Network error SHOULD retry
    println!("\nTest 2: Network error (should retry)");
    let attempts = Arc::new(AtomicU32::new(0));
    let result = backoff
        .execute(|| {
            let attempts = Arc::clone(&attempts);
            async move {
                let current = attempts.fetch_add(1, Ordering::SeqCst);
                if current < 2 {
                    println!("  Attempt {}: Returning network error", current + 1);
                    Err(std::io::Error::other("network timeout"))
                } else {
                    println!("  Attempt {}: Success!", current + 1);
                    Ok("success")
                }
            }
        })
        .await;
    assert!(result.is_ok());
    println!("Total attempts: {}", attempts.load(Ordering::SeqCst));

    Ok(())
}

/// Example 3: Jitter demonstration
async fn example_jitter_impact() -> Result<(), Box<dyn Error>> {
    println!("\n=== Example 3: Jitter Impact (Run 10 Times) ===\n");

    let backoff_no_jitter = ExponentialBackoff::builder()
        .max_retries(1)
        .initial_delay(Duration::from_millis(100))
        .jitter(0.0) // No jitter
        .build();

    let backoff_with_jitter = ExponentialBackoff::builder()
        .max_retries(1)
        .initial_delay(Duration::from_millis(100))
        .jitter(0.3) // 30% jitter
        .build();

    println!("Without jitter (10 runs):");
    let mut delays_no_jitter = Vec::new();
    for i in 0..10 {
        let api = UnreliableApi::new(1);
        let start = Instant::now();

        let _ = backoff_no_jitter
            .execute(|| {
                let api = &api;
                async move { api.call().await }
            })
            .await;

        let elapsed = start.elapsed();
        delays_no_jitter.push(elapsed);
        println!("  Run {}: {:?}", i + 1, elapsed);
    }

    println!("\nWith 30% jitter (10 runs):");
    let mut delays_with_jitter = Vec::new();
    for i in 0..10 {
        let api = UnreliableApi::new(1);
        let start = Instant::now();

        let _ = backoff_with_jitter
            .execute(|| {
                let api = &api;
                async move { api.call().await }
            })
            .await;

        let elapsed = start.elapsed();
        delays_with_jitter.push(elapsed);
        println!("  Run {}: {:?}", i + 1, elapsed);
    }

    println!("\nAnalysis:");
    println!("  No jitter: All delays should be very similar (~100ms)");
    println!("  With jitter: Delays should vary (70-130ms range)");

    // Calculate variance
    let avg_no_jitter: f64 = delays_no_jitter
        .iter()
        .map(|d| d.as_millis() as f64)
        .sum::<f64>()
        / delays_no_jitter.len() as f64;

    let avg_with_jitter: f64 = delays_with_jitter
        .iter()
        .map(|d| d.as_millis() as f64)
        .sum::<f64>()
        / delays_with_jitter.len() as f64;

    println!("  Average without jitter: {:.1}ms", avg_no_jitter);
    println!("  Average with jitter: {:.1}ms", avg_with_jitter);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("==============================================");
    println!("   TurboClaude Core: Retry Strategy Examples");
    println!("==============================================");

    example_simple_retry().await?;
    example_custom_predicate().await?;
    example_jitter_impact().await?;

    println!("\n==============================================");
    println!("   All examples completed successfully!");
    println!("==============================================\n");

    Ok(())
}
