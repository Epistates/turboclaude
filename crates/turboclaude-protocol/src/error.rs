//! Error types for protocol operations
//!
//! Provides error types for serialization, deserialization, and protocol validation.

use std::fmt;

/// Result type for protocol operations
pub type Result<T> = std::result::Result<T, ProtocolError>;

/// Errors that can occur during protocol operations
#[derive(Debug, Clone)]
pub enum ProtocolError {
    /// JSON serialization/deserialization error
    SerializationError(String),

    /// Invalid message format
    InvalidMessage(String),

    /// Missing required field
    MissingField(String),

    /// Invalid content block type
    InvalidContentBlock(String),

    /// Protocol version mismatch
    VersionMismatch {
        /// The expected protocol version.
        expected: u32,
        /// The version that was actually received.
        got: u32,
    },

    /// Invalid control request
    InvalidControlRequest(String),

    /// Permission denied
    PermissionDenied(String),

    /// Generic protocol error
    Other(String),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Self::InvalidMessage(msg) => write!(f, "Invalid message: {}", msg),
            Self::MissingField(field) => write!(f, "Missing required field: {}", field),
            Self::InvalidContentBlock(msg) => write!(f, "Invalid content block: {}", msg),
            Self::VersionMismatch { expected, got } => {
                write!(f, "Version mismatch: expected {}, got {}", expected, got)
            }
            Self::InvalidControlRequest(msg) => write!(f, "Invalid control request: {}", msg),
            Self::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ProtocolError {}

impl From<serde_json::Error> for ProtocolError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError(err.to_string())
    }
}
