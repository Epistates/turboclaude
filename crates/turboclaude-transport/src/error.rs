//! Transport error types

use std::fmt;

/// Result type for transport operations
pub type Result<T> = std::result::Result<T, TransportError>;

/// Errors that can occur in transport operations
#[derive(Debug)]
pub enum TransportError {
    /// HTTP request/response error
    Http(String),

    /// Connection error
    Connection(String),

    /// I/O error
    Io(std::io::Error),

    /// Timeout error
    Timeout,

    /// Serialization error
    Serialization(String),

    /// Process error (for subprocess transport)
    Process(String),

    /// Generic transport error
    Other(String),
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(msg) => write!(f, "HTTP error: {}", msg),
            Self::Connection(msg) => write!(f, "Connection error: {}", msg),
            Self::Io(err) => write!(f, "I/O error: {}", err),
            Self::Timeout => write!(f, "Timeout"),
            Self::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            Self::Process(msg) => write!(f, "Process error: {}", msg),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for TransportError {}

impl From<std::io::Error> for TransportError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<reqwest::Error> for TransportError {
    fn from(err: reqwest::Error) -> Self {
        Self::Http(err.to_string())
    }
}

impl From<serde_json::Error> for TransportError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}
