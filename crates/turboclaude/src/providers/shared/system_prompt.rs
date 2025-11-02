//! System prompt extraction and transformation utilities
//!
//! Consolidates duplicate system prompt handling logic from multiple providers.

use crate::types::{SystemPrompt, SystemPromptBlock};

/// Extract system prompt text from turboclaude format
///
/// Converts a turboclaude `SystemPrompt` (string or blocks) into plain text.
/// This logic was duplicated across Bedrock and Vertex provider implementations.
///
/// # Arguments
///
/// * `system` - The system prompt to extract text from
///
/// # Returns
///
/// A string containing the combined text from the system prompt.
/// For block-based prompts, blocks are joined with newlines.
///
/// # Example
///
/// ```ignore
/// use turboclaude::types::SystemPrompt;
/// use turboclaude::providers::shared::extract_system_prompt_text;
///
/// let system = SystemPrompt::String("You are helpful".to_string());
/// let text = extract_system_prompt_text(&system);
/// assert_eq!(text, "You are helpful");
/// ```
pub fn extract_system_prompt_text(system: &SystemPrompt) -> String {
    match system {
        SystemPrompt::String(s) => s.clone(),
        SystemPrompt::Blocks(blocks) => blocks
            .iter()
            .filter_map(|block| match block {
                SystemPromptBlock::Text { text, .. } => Some(text.as_str()),
            })
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_string_prompt() {
        let system = SystemPrompt::String("You are helpful".to_string());
        let text = extract_system_prompt_text(&system);
        assert_eq!(text, "You are helpful");
    }

    #[test]
    fn test_extract_block_prompt_single() {
        let system = SystemPrompt::Blocks(vec![SystemPromptBlock::Text {
            text: "You are helpful".to_string(),
            cache_control: None,
        }]);
        let text = extract_system_prompt_text(&system);
        assert_eq!(text, "You are helpful");
    }

    #[test]
    fn test_extract_block_prompt_multiple() {
        let system = SystemPrompt::Blocks(vec![
            SystemPromptBlock::Text {
                text: "You are helpful".to_string(),
                cache_control: None,
            },
            SystemPromptBlock::Text {
                text: "Be concise".to_string(),
                cache_control: None,
            },
        ]);
        let text = extract_system_prompt_text(&system);
        assert_eq!(text, "You are helpful\nBe concise");
    }

    #[test]
    fn test_extract_empty_string_prompt() {
        let system = SystemPrompt::String(String::new());
        let text = extract_system_prompt_text(&system);
        assert_eq!(text, "");
    }
}
