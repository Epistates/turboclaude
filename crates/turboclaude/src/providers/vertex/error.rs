//! Error types specific to Google Vertex AI provider

use thiserror::Error;

/// Errors specific to Google Vertex AI provider
#[derive(Debug, Error)]
pub enum VertexError {
    /// Google Cloud authentication error
    #[error("Google Cloud authentication error: {0}")]
    Authentication(String),

    /// Google Cloud API error
    #[error("Google Cloud API error: {0}")]
    Api(String),

    /// Invalid project ID
    #[error("Invalid project ID: {0}")]
    InvalidProjectId(String),

    /// Invalid region
    #[error("Invalid region: {0}")]
    InvalidRegion(String),

    /// Feature not supported in Vertex AI
    #[error("Feature not supported in Google Vertex AI: {0}")]
    UnsupportedFeature(&'static str),

    /// HTTP request error
    #[error("HTTP error: {0}")]
    Http(String),

    /// General SDK error
    #[error("SDK error: {0}")]
    Sdk(#[from] crate::Error),
}

impl From<VertexError> for crate::Error {
    fn from(err: VertexError) -> Self {
        match err {
            VertexError::Sdk(e) => e,
            other => crate::Error::Other(anyhow::anyhow!(other)),
        }
    }
}
