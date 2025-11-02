//! Usage statistics

use serde::{Deserialize, Serialize};

/// Token usage statistics for a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// Number of input tokens
    pub input_tokens: u32,

    /// Number of output tokens
    pub output_tokens: u32,

    /// Number of cache creation input tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u32>,

    /// Number of cache read input tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u32>,
}

impl Usage {
    /// Total number of tokens used.
    pub fn total_tokens(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }
}
