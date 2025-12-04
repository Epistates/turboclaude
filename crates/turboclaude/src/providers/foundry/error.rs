//! Error types for the Foundry provider

use thiserror::Error;

/// Errors specific to the Foundry provider
#[derive(Debug, Error)]
pub enum FoundryError {
    /// Missing required configuration
    #[error("Missing required configuration: {0}")]
    MissingConfig(String),

    /// Invalid resource name
    #[error("Invalid Foundry resource name: {0}")]
    InvalidResource(String),

    /// Authentication error
    #[error("Foundry authentication failed: {0}")]
    Authentication(String),

    /// Token provider error
    #[error("Azure AD token provider error: {0}")]
    TokenProvider(String),

    /// HTTP error from Foundry API
    #[error("Foundry API error: {status} - {message}")]
    Api {
        /// HTTP status code
        status: u16,
        /// Error message
        message: String,
    },

    /// Request construction error
    #[error("Failed to construct request: {0}")]
    RequestConstruction(String),

    /// Response parsing error
    #[error("Failed to parse response: {0}")]
    ResponseParsing(String),

    /// Unsupported operation
    #[error("Operation not supported on Foundry: {0}")]
    UnsupportedOperation(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),
}

impl From<FoundryError> for crate::error::Error {
    fn from(err: FoundryError) -> Self {
        match err {
            FoundryError::Authentication(msg) => crate::error::Error::Authentication(msg),
            FoundryError::Api { status, message } => crate::error::Error::ApiError {
                status,
                message,
                error_type: Some("foundry_error".to_string()),
                request_id: None,
            },
            FoundryError::MissingConfig(msg) => {
                crate::error::Error::BadRequest {
                    message: format!("Missing config: {}", msg),
                    error_type: None,
                }
            }
            FoundryError::InvalidResource(msg) => {
                crate::error::Error::BadRequest {
                    message: format!("Invalid resource: {}", msg),
                    error_type: None,
                }
            }
            FoundryError::TokenProvider(msg) => {
                crate::error::Error::Authentication(format!("Token provider: {}", msg))
            }
            FoundryError::RequestConstruction(msg) => crate::error::Error::BadRequest {
                message: msg,
                error_type: None,
            },
            FoundryError::ResponseParsing(msg) => crate::error::Error::Serialization(
                serde_json::Error::io(std::io::Error::other(msg)),
            ),
            FoundryError::UnsupportedOperation(msg) => {
                crate::error::Error::BadRequest {
                    message: format!("Unsupported: {}", msg),
                    error_type: None,
                }
            }
            FoundryError::Network(msg) => crate::error::Error::Connection(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = FoundryError::MissingConfig("API key".to_string());
        assert_eq!(
            err.to_string(),
            "Missing required configuration: API key"
        );

        let err = FoundryError::Api {
            status: 401,
            message: "Unauthorized".to_string(),
        };
        assert_eq!(err.to_string(), "Foundry API error: 401 - Unauthorized");
    }

    #[test]
    fn test_error_conversion() {
        let foundry_err = FoundryError::Authentication("Invalid token".to_string());
        let sdk_err: crate::error::Error = foundry_err.into();

        match sdk_err {
            crate::error::Error::Authentication(msg) => {
                assert_eq!(msg, "Invalid token");
            }
            _ => panic!("Expected Authentication error"),
        }
    }

    #[test]
    fn test_api_error_conversion() {
        let foundry_err = FoundryError::Api {
            status: 429,
            message: "Rate limited".to_string(),
        };
        let sdk_err: crate::error::Error = foundry_err.into();

        match sdk_err {
            crate::error::Error::ApiError {
                status,
                error_type,
                message,
                request_id,
            } => {
                assert_eq!(status, 429);
                assert_eq!(error_type, Some("foundry_error".to_string()));
                assert_eq!(message, "Rate limited");
                assert_eq!(request_id, None);
            }
            _ => panic!("Expected ApiError error"),
        }
    }
}
