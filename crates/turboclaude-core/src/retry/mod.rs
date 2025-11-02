//! Retry strategies and backoff implementations.
//!
//! This module provides a universal abstraction for retry logic with exponential
//! backoff, jitter, and custom retry predicates.
//!
//! # Key Types
//!
//! - [`BackoffStrategy`] - Core trait for retry strategies
//! - [`ExponentialBackoff`] - Exponential backoff with jitter
//!
//! # Examples
//!
//! ```rust
//! use turboclaude_core::retry::{BackoffStrategy, ExponentialBackoff};
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let backoff = ExponentialBackoff::builder()
//!     .max_retries(3)
//!     .initial_delay(Duration::from_millis(100))
//!     .build();
//!
//! let result = backoff.execute(|| async {
//!     // Your operation here
//!     Ok::<_, std::io::Error>(42)
//! }).await?;
//! # Ok(())
//! # }
//! ```

mod exponential;
mod strategy;

pub use exponential::{ExponentialBackoff, ExponentialBackoffBuilder};
pub use strategy::{BackoffBuilder, BackoffStrategy};
