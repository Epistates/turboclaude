//! Legacy completions API endpoint

use crate::{client::Client, error::Result};

/// Completions API resource (legacy).
///
/// This is the legacy API for text completions. New applications should use
/// the Messages API instead.
#[derive(Clone)]
pub struct Completions {
    client: Client,
}

impl Completions {
    /// Create a new Completions resource.
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Create a completion (legacy).
    ///
    /// **Note**: This is a legacy endpoint. Use the Messages API for new applications.
    #[allow(dead_code)]
    pub async fn create(&self, _request: CompletionRequest) -> Result<Completion> {
        Err(crate::error::Error::FeatureNotAvailable(
            "Legacy completions API is deprecated",
            "messages"
        ))
    }
}

/// Legacy completion request.
#[derive(Debug, serde::Serialize)]
pub struct CompletionRequest {
    /// Model to use
    pub model: String,

    /// The prompt
    pub prompt: String,

    /// Maximum tokens to generate
    pub max_tokens_to_sample: u32,

    /// Temperature
    pub temperature: Option<f32>,

    /// Stop sequences
    pub stop_sequences: Option<Vec<String>>,
}

/// Legacy completion response.
#[derive(Debug, serde::Deserialize)]
pub struct Completion {
    /// Generated completion
    pub completion: String,

    /// Stop reason
    pub stop_reason: Option<String>,

    /// Model used
    pub model: String,
}