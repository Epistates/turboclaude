//! Error types specific to AWS Bedrock provider

use thiserror::Error;

/// Errors specific to AWS Bedrock provider
#[derive(Debug, Error)]
pub enum BedrockError {
    /// AWS SDK configuration error
    #[error("AWS configuration error: {0}")]
    Configuration(String),

    /// AWS credentials error
    #[error("AWS credentials error: {0}")]
    Credentials(String),

    /// AWS service error
    #[error("AWS Bedrock service error: {0}")]
    Service(String),

    /// Type translation error
    #[error("Type translation error: {0}")]
    Translation(String),

    /// Feature not supported in Bedrock
    #[error("Feature not supported in AWS Bedrock: {0}")]
    UnsupportedFeature(&'static str),

    /// Model ID format error
    #[error("Invalid model ID format: {0}")]
    InvalidModelId(String),

    /// General SDK error
    #[error("SDK error: {0}")]
    Sdk(#[from] crate::Error),
}

impl From<BedrockError> for crate::Error {
    fn from(err: BedrockError) -> Self {
        match err {
            BedrockError::Sdk(e) => e,
            other => crate::Error::Other(anyhow::anyhow!(other)),
        }
    }
}
