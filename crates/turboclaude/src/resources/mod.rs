//! API resource endpoints
//!
//! This module contains the implementation of all API endpoints,
//! organized by resource type similar to the Python SDK.

pub mod beta;
pub mod completions;
pub mod messages;
pub mod models;

pub use beta::Beta;
pub use completions::Completions;
pub use messages::{BatchRequest, Messages, TokenCount};
pub use models::Models;

use crate::client::Client;

/// Base trait for API resources.
pub trait Resource {
    /// Get a reference to the client.
    fn client(&self) -> &Client;
}
