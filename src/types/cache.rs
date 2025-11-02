//! Prompt caching types

use serde::{Deserialize, Serialize};

/// Cache control configuration for prompt caching.
///
/// Enables caching of content blocks to reduce costs and latency.
/// See [Prompt Caching documentation](https://docs.anthropic.com/en/docs/build-with-claude/prompt-caching).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CacheControl {
    /// Ephemeral cache control with configurable TTL
    Ephemeral {
        /// Time-to-live for the cache control breakpoint
        #[serde(skip_serializing_if = "Option::is_none")]
        ttl: Option<CacheTTL>,
    },
}

impl CacheControl {
    /// Create an ephemeral cache control with default TTL (5 minutes).
    pub fn ephemeral() -> Self {
        Self::Ephemeral { ttl: None }
    }

    /// Create an ephemeral cache control with a specific TTL.
    pub fn ephemeral_with_ttl(ttl: CacheTTL) -> Self {
        Self::Ephemeral { ttl: Some(ttl) }
    }
}

/// Time-to-live options for cache control.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CacheTTL {
    /// 5 minutes (default)
    #[serde(rename = "5m")]
    FiveMinutes,
    /// 1 hour
    #[serde(rename = "1h")]
    OneHour,
}

impl Default for CacheTTL {
    fn default() -> Self {
        Self::FiveMinutes
    }
}

/// System prompt block (can be cached).
///
/// System prompts can be either plain strings or structured blocks with cache control.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum SystemPromptBlock {
    /// Text block with optional cache control
    #[serde(rename = "text")]
    Text {
        /// The text content
        text: String,
        /// Cache control configuration
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
}

impl SystemPromptBlock {
    /// Create a text block without caching.
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text {
            text: content.into(),
            cache_control: None,
        }
    }

    /// Create a cached text block with default TTL.
    pub fn text_cached(content: impl Into<String>) -> Self {
        Self::Text {
            text: content.into(),
            cache_control: Some(CacheControl::ephemeral()),
        }
    }

    /// Create a cached text block with specific TTL.
    pub fn text_cached_with_ttl(content: impl Into<String>, ttl: CacheTTL) -> Self {
        Self::Text {
            text: content.into(),
            cache_control: Some(CacheControl::ephemeral_with_ttl(ttl)),
        }
    }
}
