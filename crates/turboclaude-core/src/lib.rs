#![deny(unsafe_code)]
#![warn(missing_docs)]

//! Core abstractions for the TurboClaude ecosystem.
//!
//! This crate provides foundational traits and utilities that are shared
//! across all TurboClaude crates, enabling:
//!
//! - **Universal retry strategies** via `BackoffStrategy` trait
//!   - Exponential backoff with jitter
//!   - Custom retry predicates
//!   - Circuit breaker support (future)
//! - **Consistent resource lifecycle management** via `Resource<T>` and `LazyResource<T>`
//! - **Declarative error boundaries** via `error_boundary!` macro
//! - **Standardized serialization** via `SerializePipeline` trait
//!
//! # Design Philosophy
//!
//! This crate consolidates duplicated patterns across the TurboClaude ecosystem:
//!
//! - **Before**: Retry logic duplicated in 2 places with different APIs
//! - **After**: One universal abstraction, multiple implementations
//! - **Before**: Error conversion via repetitive `map_err()` chains
//! - **After**: Declarative `error_boundary!` definitions at module boundaries
//! - **Before**: Manual serde method calls scattered throughout codebase
//! - **After**: Unified `SerializePipeline` interface for all protocol types
//!
//! # Examples
//!
//! Using the prelude for convenient imports:
//!
//! ```rust
//! use turboclaude_core::prelude::*;
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let backoff = ExponentialBackoff::builder()
//!     .max_retries(3)
//!     .initial_delay(Duration::from_millis(100))
//!     .build();
//!
//! let result = backoff.execute(|| async {
//!     Ok::<_, std::io::Error>(42)
//! }).await?;
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod resource;
pub mod retry;
pub mod serde;

/// Convenient re-exports of commonly used items.
///
/// Import all core abstractions with:
///
/// ```rust
/// use turboclaude_core::prelude::*;
/// ```
pub mod prelude {
    pub use crate::error::ErrorBoundary;
    pub use crate::error_boundary;
    pub use crate::resource::{LazyResource, Resource};
    pub use crate::retry::{BackoffStrategy, ExponentialBackoff, ExponentialBackoffBuilder};
    pub use crate::serde::SerializePipeline;
}
