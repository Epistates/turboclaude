//! Content block transformation utilities
//!
//! Consolidates duplicate content block filtering/transformation logic from multiple providers.
//! This is currently designed for Bedrock but can be extended for other providers.

use crate::error::Result;
use crate::types::ContentBlockParam;

/// Transform turboclaude content blocks into provider-specific format
///
/// This logic was duplicated across Bedrock client and translate modules.
/// Provides centralized content block transformation.
///
/// # Arguments
///
/// * `blocks` - The turboclaude content blocks to transform
///
/// # Returns
///
/// A vector of provider-specific content blocks, or an error if transformation fails
///
/// # Note
///
/// Currently this is a placeholder that validates the blocks exist.
/// Provider-specific implementations will extend this with actual transformation logic.
///
/// # Example
///
/// ```ignore
/// use turboclaude::providers::shared::transform_content_blocks;
/// use turboclaude::types::ContentBlockParam;
///
/// let blocks = vec![
///     ContentBlockParam::Text { text: "Hello".to_string() }
/// ];
/// let result = transform_content_blocks(&blocks)?;
/// ```
pub fn transform_content_blocks(blocks: &[ContentBlockParam]) -> Result<()> {
    // Validate that we have content blocks
    if blocks.is_empty() {
        return Err(crate::error::Error::InvalidRequest(
            "Content blocks cannot be empty".to_string(),
        ));
    }

    // Basic validation - can be extended by providers
    for (idx, block) in blocks.iter().enumerate() {
        match block {
            ContentBlockParam::Text { text } => {
                if text.is_empty() {
                    return Err(crate::error::Error::InvalidRequest(format!(
                        "Text block at index {} is empty",
                        idx
                    )));
                }
            }
            ContentBlockParam::Image { source } => {
                if source.data.is_empty() {
                    return Err(crate::error::Error::InvalidRequest(format!(
                        "Image data at index {} is empty",
                        idx
                    )));
                }
            }
            ContentBlockParam::Document { source, .. } => match source {
                crate::types::DocumentSource::PlainText { text } => {
                    if text.is_empty() {
                        return Err(crate::error::Error::InvalidRequest(format!(
                            "Document text at index {} is empty",
                            idx
                        )));
                    }
                }
                crate::types::DocumentSource::Base64PDF { data, .. } => {
                    if data.is_empty() {
                        return Err(crate::error::Error::InvalidRequest(format!(
                            "Document data at index {} is empty",
                            idx
                        )));
                    }
                }
                crate::types::DocumentSource::URL { url } => {
                    if url.is_empty() {
                        return Err(crate::error::Error::InvalidRequest(format!(
                            "Document URL at index {} is empty",
                            idx
                        )));
                    }
                }
            },
            ContentBlockParam::ToolResult { tool_use_id, .. } => {
                if tool_use_id.is_empty() {
                    return Err(crate::error::Error::InvalidRequest(format!(
                        "Tool use ID at index {} is empty",
                        idx
                    )));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_empty_blocks() {
        let blocks: Vec<ContentBlockParam> = vec![];
        let result = transform_content_blocks(&blocks);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_validate_text_block() {
        let blocks = vec![ContentBlockParam::Text {
            text: "Hello".to_string(),
        }];
        let result = transform_content_blocks(&blocks);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_empty_text_block() {
        let blocks = vec![ContentBlockParam::Text {
            text: String::new(),
        }];
        let result = transform_content_blocks(&blocks);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_validate_multiple_blocks() {
        let blocks = vec![
            ContentBlockParam::Text {
                text: "Hello".to_string(),
            },
            ContentBlockParam::Text {
                text: "World".to_string(),
            },
        ];
        let result = transform_content_blocks(&blocks);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_tool_result_empty_id() {
        let blocks = vec![ContentBlockParam::ToolResult {
            tool_use_id: String::new(),
            content: "Result".to_string(),
            is_error: None,
        }];
        let result = transform_content_blocks(&blocks);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Tool use ID"));
    }
}
