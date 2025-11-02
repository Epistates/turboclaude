//! Validation functions for skill metadata and structure

use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

use crate::error::{Result, SkillError};

/// Regex pattern for valid skill names
///
/// Valid format: hyphen-case (lowercase alphanumeric + hyphens)
/// - Must start and end with alphanumeric
/// - Cannot have consecutive hyphens
/// - Pattern: ^[a-z0-9]+(-[a-z0-9]+)*$
static SKILL_NAME_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").expect("Failed to compile skill name regex")
});

/// Validate a skill name according to the Agent Skills Spec v1.0
///
/// Valid names:
/// - Lowercase alphanumeric characters and hyphens only
/// - Must start and end with alphanumeric (not hyphen)
/// - Cannot have consecutive hyphens
/// - Examples: "skill", "my-skill", "skill-1-2"
///
/// Invalid names:
/// - Uppercase: "Skill", "`MySkill`"
/// - Underscores: "`my_skill`"
/// - Spaces: "my skill"
/// - Leading/trailing hyphens: "-skill", "skill-"
/// - Consecutive hyphens: "skill--name"
/// - Empty: ""
///
/// # Errors
///
/// Returns `SkillError::InvalidName` if the name doesn't match the required pattern.
pub fn validate_skill_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(SkillError::invalid_name("empty string"));
    }

    if !SKILL_NAME_PATTERN.is_match(name) {
        return Err(SkillError::invalid_name(name));
    }

    Ok(())
}

/// Validate that the skill name matches its directory name
///
/// # Errors
///
/// Returns `SkillError::NameMismatch` if names don't match.
pub fn validate_name_matches_directory(skill_path: &Path, metadata_name: &str) -> Result<()> {
    let dir_name = skill_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| SkillError::invalid_directory("cannot determine directory name"))?;

    if dir_name != metadata_name {
        return Err(SkillError::name_mismatch(dir_name, metadata_name));
    }

    Ok(())
}

/// Validate that required fields are present in metadata
#[allow(dead_code)] // Part of public API, used in tests
pub fn validate_required_fields(name: Option<&str>, description: Option<&str>) -> Result<()> {
    if name.is_none() {
        return Err(SkillError::missing_field("name"));
    }

    if description.is_none() {
        return Err(SkillError::missing_field("description"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_skill_names() {
        // Single character
        assert!(validate_skill_name("a").is_ok());
        assert!(validate_skill_name("x").is_ok());
        assert!(validate_skill_name("1").is_ok());

        // Simple names
        assert!(validate_skill_name("skill").is_ok());
        assert!(validate_skill_name("pdf").is_ok());
        assert!(validate_skill_name("test123").is_ok());

        // Hyphenated names
        assert!(validate_skill_name("my-skill").is_ok());
        assert!(validate_skill_name("skill-name").is_ok());
        assert!(validate_skill_name("slack-gif-creator").is_ok());
        assert!(validate_skill_name("mcp-builder").is_ok());

        // With numbers
        assert!(validate_skill_name("skill-1").is_ok());
        assert!(validate_skill_name("skill-1-2-3").is_ok());
        assert!(validate_skill_name("test-v2").is_ok());

        // Long names
        assert!(validate_skill_name("very-long-skill-name-with-many-parts").is_ok());
    }

    #[test]
    fn test_invalid_skill_names() {
        // Empty
        assert!(validate_skill_name("").is_err());

        // Uppercase
        assert!(validate_skill_name("Skill").is_err());
        assert!(validate_skill_name("MySkill").is_err());
        assert!(validate_skill_name("SKILL").is_err());
        assert!(validate_skill_name("skill-Name").is_err());

        // Underscores
        assert!(validate_skill_name("skill_name").is_err());
        assert!(validate_skill_name("my_skill").is_err());

        // Spaces
        assert!(validate_skill_name("my skill").is_err());
        assert!(validate_skill_name("skill name").is_err());

        // Leading hyphen
        assert!(validate_skill_name("-skill").is_err());
        assert!(validate_skill_name("-my-skill").is_err());

        // Trailing hyphen
        assert!(validate_skill_name("skill-").is_err());
        assert!(validate_skill_name("my-skill-").is_err());

        // Consecutive hyphens
        assert!(validate_skill_name("skill--name").is_err());
        assert!(validate_skill_name("my---skill").is_err());

        // Special characters
        assert!(validate_skill_name("skill@name").is_err());
        assert!(validate_skill_name("skill.name").is_err());
        assert!(validate_skill_name("skill/name").is_err());
        assert!(validate_skill_name("skill\\name").is_err());

        // Only hyphen
        assert!(validate_skill_name("-").is_err());
        assert!(validate_skill_name("--").is_err());
    }

    #[test]
    fn test_name_matches_directory() {
        use std::path::PathBuf;

        // Matching names
        let path = PathBuf::from("/skills/my-skill");
        assert!(validate_name_matches_directory(&path, "my-skill").is_ok());

        let path = PathBuf::from("/path/to/pdf");
        assert!(validate_name_matches_directory(&path, "pdf").is_ok());

        // Mismatched names
        let path = PathBuf::from("/skills/my-skill");
        assert!(validate_name_matches_directory(&path, "other-skill").is_err());

        let path = PathBuf::from("/skills/pdf");
        assert!(validate_name_matches_directory(&path, "document-pdf").is_err());
    }

    #[test]
    fn test_required_fields() {
        // Both present
        assert!(validate_required_fields(Some("name"), Some("description")).is_ok());

        // Missing name
        assert!(validate_required_fields(None, Some("description")).is_err());

        // Missing description
        assert!(validate_required_fields(Some("name"), None).is_err());

        // Both missing
        assert!(validate_required_fields(None, None).is_err());
    }

    #[test]
    fn test_error_messages() {
        let err = validate_skill_name("Invalid-Name").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Invalid-Name"));
        assert!(msg.contains("hyphen-case"));

        let err = validate_skill_name("skill_name").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("skill_name"));
    }
}
