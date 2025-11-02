//! Error Recovery Example
//!
//! This example demonstrates the error recovery system that provides:
//! 1. Self-documenting errors with suggested actions
//! 2. Automatic retry logic based on error types
//! 3. Configurable backoff strategies
//! 4. Clear guidance for users on how to handle each error
//!
//! Key concepts:
//! - ErrorRecovery trait: Provides error classification and recovery guidance
//! - BackoffStrategy: Controls retry delays (Linear, Exponential, or None)
//! - retry_with_recovery(): Automatic retry with error-specific logic
//! - retry(): Simplified retry using error's built-in policy
//!
//! Run with: cargo run --example error_recovery

use turboclaudeagent::{AgentError, BackoffStrategy, ErrorRecovery, retry, retry_with_recovery};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔄 Error Recovery System Demonstration\n");

    // ============================================================
    // SECTION 1: Understanding Error Classification
    // ============================================================
    println!("📋 Section 1: Error Classification & Recovery Guidance");
    println!("═════════════════════════════════════════════════════\n");

    demonstrate_error_classification().await;

    // ============================================================
    // SECTION 2: Understanding Backoff Strategies
    // ============================================================
    println!("\n📋 Section 2: Backoff Strategies");
    println!("═════════════════════════════════════════════════════\n");

    demonstrate_backoff_strategies();

    // ============================================================
    // SECTION 3: Manual Error Handling with Recovery Guidance
    // ============================================================
    println!("\n📋 Section 3: Manual Error Handling");
    println!("═════════════════════════════════════════════════════\n");

    demonstrate_manual_handling().await;

    // ============================================================
    // SECTION 4: Automatic Retry with recovery
    // ============================================================
    println!("\n📋 Section 4: Automatic Retry with Backoff");
    println!("═════════════════════════════════════════════════════\n");

    demonstrate_automatic_retry().await;

    // ============================================================
    // SECTION 5: Real-World Error Handling Patterns
    // ============================================================
    println!("\n📋 Section 5: Real-World Patterns");
    println!("═════════════════════════════════════════════════════\n");

    demonstrate_real_world_patterns().await;

    println!("\n✨ Error Recovery Demonstration Complete!\n");
    Ok(())
}

/// Demonstrate error classification
async fn demonstrate_error_classification() {
    let errors = vec![
        // Retriable errors
        AgentError::Transport("connection timeout".to_string()),
        AgentError::Transport("subprocess closed".to_string()),
        // Non-retriable errors
        AgentError::Protocol("max_tokens exceeded".to_string()),
        AgentError::PermissionDenied("edit denied".to_string()),
        AgentError::Config("invalid configuration".to_string()),
    ];

    for error in errors {
        println!("Error: {}", error);
        println!("  Retriable: {}", error.is_retriable());
        println!("  Max retries: {:?}", error.max_retries());
        println!("  Suggested action: {}", error.suggested_action());
        println!();
    }
}

/// Demonstrate backoff strategy calculations
fn demonstrate_backoff_strategies() {
    println!("Linear Backoff (base_ms=100):");
    let linear = BackoffStrategy::Linear { base_ms: 100 };
    for attempt in 1..=3 {
        if let Some(delay) = linear.delay_for_attempt(attempt) {
            println!("  Attempt {}: wait {:?}", attempt, delay);
        }
    }

    println!("\nExponential Backoff (base=100ms, max=10s):");
    let exponential = BackoffStrategy::Exponential {
        base_ms: 100,
        max_ms: 10_000,
    };
    for attempt in 1..=6 {
        if let Some(delay) = exponential.delay_for_attempt(attempt) {
            println!("  Attempt {}: wait {:?}", attempt, delay);
        }
    }

    println!("\nNo Backoff:");
    let none = BackoffStrategy::None;
    println!("  No delay, don't retry: {:?}", none.delay_for_attempt(1));
}

/// Manual error handling with recovery guidance
async fn demonstrate_manual_handling() {
    println!("Pattern: Check error type and suggested action\n");

    let transport_error = AgentError::Transport("connection lost".to_string());

    match &transport_error {
        AgentError::Transport(_) => {
            println!("Got Transport Error");
            println!(
                "  Retriable: {} → Should retry",
                transport_error.is_retriable()
            );
            println!(
                "  Max attempts: {}",
                transport_error.max_retries().unwrap_or(0)
            );
            println!("  User guidance: {}", transport_error.suggested_action());
            println!("\n  Action: Session will auto-reconnect on next query");
        }
        _ => {}
    }

    println!("\nPattern: Non-retriable error handling\n");

    let config_error = AgentError::Config("missing api_key".to_string());
    println!("Got Config Error");
    println!("  Retriable: {} → Don't retry", config_error.is_retriable());
    println!("  User guidance: {}", config_error.suggested_action());
    println!("\n  Action: User must fix configuration");
}

/// Demonstrate automatic retry with recovery
async fn demonstrate_automatic_retry() {
    println!("Pattern 1: retry_with_recovery with explicit max attempts\n");

    let attempt_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let counter = attempt_count.clone();

    let result: Result<String, AgentError> = retry_with_recovery(
        Box::new(move || {
            let counter = counter.clone();
            Box::pin(async move {
                let attempt = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                println!("  Attempt {}", attempt);

                if attempt < 3 {
                    // Fail on first 2 attempts
                    Err(AgentError::Transport("transient failure".to_string()))
                } else {
                    // Succeed on third attempt
                    Ok("Success!".to_string())
                }
            })
        }),
        Some(3), // max 3 attempts
    )
    .await;

    match result {
        Ok(msg) => println!("✅ Result: {}\n", msg),
        Err(e) => println!("❌ Failed: {}\n", e),
    }

    println!("Pattern 2: retry using error's built-in policy\n");

    let attempt_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let counter = attempt_count.clone();

    let result: Result<String, AgentError> = retry(Box::new(move || {
        let counter = counter.clone();
        Box::pin(async move {
            let attempt = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            println!("  Attempt {}", attempt);

            // This error is Transport, which has max_retries = 5
            if attempt < 2 {
                Err(AgentError::Transport("transient failure".to_string()))
            } else {
                Ok("Success!".to_string())
            }
        })
    }))
    .await;

    match result {
        Ok(msg) => println!("✅ Result: {}", msg),
        Err(e) => println!("❌ Failed: {}", e),
    }
}

/// Real-world error handling patterns
async fn demonstrate_real_world_patterns() {
    println!("Pattern 1: Check before deciding to retry\n");

    let error = AgentError::Transport("connection reset".to_string());

    if error.is_retriable() {
        println!("Error is retriable");
        println!("Suggested action: {}", error.suggested_action());
        println!("Backoff strategy: {:?}", error.backoff_strategy());
        println!("→ Will attempt to retry with exponential backoff\n");
    }

    println!("Pattern 2: Error-aware logging\n");

    let errors = vec![
        AgentError::Transport("timeout".to_string()),
        AgentError::Protocol("invalid input".to_string()),
        AgentError::PermissionDenied("insufficient permissions".to_string()),
    ];

    for error in errors {
        println!("Error occurred: {}", error);
        println!(
            "  Category: {}",
            if error.is_retriable() {
                "Transient"
            } else {
                "Permanent"
            }
        );
        println!("  Guidance: {}", error.suggested_action());
        println!();
    }

    println!("Pattern 3: Conditional retry based on error type\n");

    async fn attempt_operation() -> Result<String, AgentError> {
        // Simulate operation that might fail
        Err(AgentError::Transport("network issue".to_string()))
    }

    match attempt_operation().await {
        Ok(result) => println!("Success: {}", result),
        Err(error) => {
            if error.is_retriable() {
                println!("Transient error detected: {}", error);
                println!("Attempting automatic recovery with backoff...");
                println!("Max retries: {} times", error.max_retries().unwrap_or(0));
                println!("Backoff: {:?}", error.backoff_strategy());
            } else {
                println!("Permanent error detected: {}", error);
                println!("User action required: {}", error.suggested_action());
            }
        }
    }
}
