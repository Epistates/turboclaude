//! SKILL.md file parsing

use crate::error::{Result, SkillError};
use crate::skill::SkillMetadata;

/// Parse a SKILL.md file into metadata and body
///
/// Expected format:
/// ```yaml
/// ---
/// name: skill-name
/// description: Description here
/// ---
///
/// # Markdown body
/// Content here...
/// ```
///
/// # Errors
///
/// Returns error if:
/// - Frontmatter delimiters (---) are missing
/// - YAML is invalid
/// - Required fields are missing
///
/// # Notes
///
/// - Only the first two `---` delimiters are treated as frontmatter boundaries
/// - Additional `---` in the body are preserved as content (e.g., Markdown horizontal rules)
/// - Empty body is valid
pub fn parse_skill_file(content: &str) -> Result<(SkillMetadata, String)> {
    // Split by --- delimiters
    let parts: Vec<&str> = content.split("---").collect();

    // Need at least 3 parts: [empty/content before first ---, frontmatter, body...]
    if parts.len() < 3 {
        return Err(SkillError::MissingFrontmatter);
    }

    // First part should be empty or whitespace (before first ---)
    let first_part = parts[0].trim();
    if !first_part.is_empty() {
        return Err(SkillError::invalid_format(
            "SKILL.md must start with --- delimiter",
        ));
    }

    // Second part is the YAML frontmatter
    let frontmatter = parts[1];

    // Parse metadata
    let metadata: SkillMetadata = serde_yaml::from_str(frontmatter)
        .map_err(|e| SkillError::invalid_format(format!("Invalid YAML frontmatter: {e}")))?;

    // Remaining parts are the body (joined with --- to preserve any --- in content)
    let body = parts[2..].join("---").trim().to_string();

    Ok((metadata, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_skill() {
        let content = r"---
name: test-skill
description: A test skill
---

# Test Content

This is the body.
";

        let (metadata, body) = parse_skill_file(content).unwrap();

        assert_eq!(metadata.name, "test-skill");
        assert_eq!(metadata.description, "A test skill");
        assert!(metadata.license.is_none());
        assert!(metadata.allowed_tools.is_none()); // No allowed_tools field = all allowed
        assert!(metadata.metadata.is_empty());
        assert!(body.contains("# Test Content"));
        assert!(body.contains("This is the body."));
    }

    #[test]
    fn test_parse_full_skill() {
        let content = r#"---
name: full-skill
description: |
  Multi-line description
  with multiple lines
license: MIT
allowed-tools:
  - bash
  - read
  - write
metadata:
  author: Test Author
  version: "1.0.0"
---

# Full Skill

Content here.
"#;

        let (metadata, body) = parse_skill_file(content).unwrap();

        assert_eq!(metadata.name, "full-skill");
        assert!(metadata.description.contains("Multi-line"));
        assert_eq!(metadata.license, Some("MIT".to_string()));

        let tools = metadata.allowed_tools.as_ref().unwrap();
        assert_eq!(tools.len(), 3);
        assert!(tools.contains("bash"));
        assert!(tools.contains("read"));
        assert!(tools.contains("write"));

        assert_eq!(metadata.metadata.len(), 2);
        assert!(body.contains("# Full Skill"));
    }

    #[test]
    fn test_parse_empty_body() {
        let content = r"---
name: minimal
description: Minimal skill
---
";

        let (metadata, body) = parse_skill_file(content).unwrap();

        assert_eq!(metadata.name, "minimal");
        assert_eq!(metadata.description, "Minimal skill");
        assert!(body.is_empty());
    }

    #[test]
    fn test_parse_body_with_horizontal_rules() {
        let content = r"---
name: test
description: Test
---

# Content

Some text

---

More content after horizontal rule

---

Even more content
";

        let (metadata, body) = parse_skill_file(content).unwrap();

        assert_eq!(metadata.name, "test");
        // Body should preserve the --- markers
        assert!(body.contains("---"));
        // Count occurrences - should have 2 --- in body
        let count = body.matches("---").count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_parse_missing_frontmatter() {
        let content = "# No frontmatter here";

        let result = parse_skill_file(content);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SkillError::MissingFrontmatter
        ));
    }

    #[test]
    fn test_parse_content_before_first_delimiter() {
        let content = r"Content before
---
name: test
description: Test
---
Body
";

        let result = parse_skill_file(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let content = r"---
name: test
description: Test
invalid_yaml: [unclosed
---
Body
";

        let result = parse_skill_file(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_required_name() {
        let content = r"---
description: Test
---
Body
";

        let result = parse_skill_file(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_required_description() {
        let content = r"---
name: test
---
Body
";

        let result = parse_skill_file(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_multiline_description() {
        let content = r"---
name: test
description: |
  Line 1
  Line 2
  Line 3
---
Body
";

        let (metadata, _) = parse_skill_file(content).unwrap();
        assert!(metadata.description.contains("Line 1"));
        assert!(metadata.description.contains("Line 2"));
        assert!(metadata.description.contains("Line 3"));
    }

    #[test]
    fn test_parse_empty_allowed_tools() {
        let content = r"---
name: test
description: Test
allowed-tools: []
---
Body
";

        let (metadata, _) = parse_skill_file(content).unwrap();

        let tools = metadata.allowed_tools.as_ref().unwrap();
        assert_eq!(tools.len(), 0);
        // Empty list means NO tools allowed
        assert!(!metadata.allows_tool("bash"));
    }

    #[test]
    fn test_parse_custom_metadata() {
        let content = r"---
name: test
description: Test
metadata:
  custom-field: value
  another: 123
  nested:
    key: nested-value
---
Body
";

        let (metadata, _) = parse_skill_file(content).unwrap();
        assert!(metadata.metadata.contains_key("custom-field"));
        assert!(metadata.metadata.contains_key("another"));
        assert!(metadata.metadata.contains_key("nested"));
    }
}
