//! Microsoft Azure Foundry provider for Claude models
//!
//! This module provides access to Claude models through Microsoft Azure Foundry,
//! enabling enterprise integration with Azure's AI infrastructure.
//!
//! ## Authentication
//!
//! The Foundry provider supports two authentication methods:
//!
//! ### API Key Authentication (Simple)
//! Use the `ANTHROPIC_FOUNDRY_API_KEY` environment variable or provide directly:
//!
//! ```rust,no_run
//! use turboclaude::providers::foundry::FoundryHttpProvider;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let provider = FoundryHttpProvider::builder()
//!     .api_key("your-foundry-api-key")
//!     .resource("your-foundry-resource")
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Azure AD Token Authentication (Enterprise)
//! For production deployments, use Azure AD managed identity or service principal:
//!
//! ```rust,no_run
//! use turboclaude::providers::foundry::FoundryHttpProvider;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let provider = FoundryHttpProvider::builder()
//!     .resource("your-foundry-resource")
//!     .azure_ad_token_provider(|| async {
//!         // Your token acquisition logic here
//!         Ok("azure-ad-token".to_string())
//!     })
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Environment Variables
//!
//! The provider reads configuration from these environment variables:
//!
//! - `ANTHROPIC_FOUNDRY_API_KEY` - API key for authentication
//! - `ANTHROPIC_FOUNDRY_RESOURCE` - Azure resource name (e.g., "my-foundry-resource")
//! - `ANTHROPIC_FOUNDRY_BASE_URL` - Override the default endpoint URL
//!
//! ## Example
//!
//! ```rust,no_run
//! use turboclaude::Client;
//! use turboclaude::providers::foundry::FoundryHttpProvider;
//! use turboclaude::types::MessageRequest;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create Foundry provider
//! let provider = Arc::new(FoundryHttpProvider::builder()
//!     .resource("my-foundry-resource")
//!     .build()?);
//!
//! // Foundry uses standard Anthropic model IDs
//! // e.g., "claude-sonnet-4-5-20250514", "claude-3-5-sonnet-20241022"
//! # Ok(())
//! # }
//! ```
//!
//! ## Limitations
//!
//! The following features are not available through Azure Foundry:
//! - Message Batches API
//! - Models list API
//! - Admin endpoints
//!
//! ## References
//!
//! - [Microsoft Foundry Documentation](https://azure.microsoft.com/en-us/blog/introducing-anthropics-claude-models-in-microsoft-foundry/)
//! - [Anthropic Claude on Azure](https://docs.anthropic.com/en/api/claude-on-microsoft-azure)

mod error;
mod http;

pub use error::FoundryError;
pub use http::{FoundryHttpProvider, FoundryHttpProviderBuilder};

/// Default API version for Foundry
pub const FOUNDRY_API_VERSION: &str = "2023-06-01";
