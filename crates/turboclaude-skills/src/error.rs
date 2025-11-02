//! Error types for the skills system

use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

/// Result type for skill operations
pub type Result<T> = std::result::Result<T, SkillError>;

/// Errors that can occur when working with skills
#[derive(Debug, Error)]
pub enum SkillError {
    // Parsing errors
    /// Invalid SKILL.md format
    #[error("Invalid SKILL.md format: {0}")]
    InvalidFormat(String),

    /// YAML frontmatter parse error
    #[error("YAML parse error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    /// Missing required field in SKILL.md frontmatter
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// SKILL.md file has no frontmatter delimiters
    #[error("Missing frontmatter delimiters (---) in SKILL.md")]
    MissingFrontmatter,

    // Validation errors
    /// Invalid skill name format
    #[error(
        "Invalid skill name: '{0}'. Must be hyphen-case (lowercase alphanumeric + hyphens, no leading/trailing/consecutive hyphens)"
    )]
    InvalidName(String),

    /// Skill name in metadata doesn't match directory name
    #[error("Skill name mismatch: directory name is '{0}', but SKILL.md declares name '{1}'")]
    NameMismatch(String, String),

    /// Invalid directory structure
    #[error("Invalid skill directory: {0}")]
    InvalidDirectory(String),

    // Not found errors
    /// Skill not found in registry
    #[error("Skill not found: {0}")]
    NotFound(String),

    /// Reference document not found
    #[error("Reference not found: {0}")]
    ReferenceNotFound(PathBuf),

    /// Script not found
    #[error("Script not found: {0}")]
    ScriptNotFound(PathBuf),

    // Tool errors
    /// Tool not allowed by skill's allowed-tools list
    #[error("Tool '{0}' is not allowed by this skill. Allowed tools: {1:?}")]
    ToolNotAllowed(String, Vec<String>),

    // Script errors
    /// Script execution failed
    #[error("Script execution failed: {0}")]
    ScriptFailed(String),

    /// Script execution error (generic)
    #[error("Script execution error: {0}")]
    ScriptExecution(String),

    /// Script execution timed out
    #[error("Script timeout after {0:?}")]
    ScriptTimeout(Duration),

    /// Script returned non-zero exit code
    #[error("Script exited with code {code}: {stderr}")]
    ScriptExitCode {
        /// Exit code
        code: i32,
        /// Standard error output
        stderr: String,
    },

    /// Unsupported script type
    #[error("Unsupported script type: {0}")]
    UnsupportedScriptType(String),

    // I/O errors
    /// Filesystem I/O error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Regex compilation error
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    /// Walkdir error during directory traversal
    #[error("Directory traversal error: {0}")]
    WalkDir(#[from] walkdir::Error),

    // Composed errors
    /// Generic error with context
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl SkillError {
    /// Create a new `InvalidFormat` error
    pub fn invalid_format(msg: impl Into<String>) -> Self {
        Self::InvalidFormat(msg.into())
    }

    /// Create a new `MissingField` error
    pub fn missing_field(field: impl Into<String>) -> Self {
        Self::MissingField(field.into())
    }

    /// Create a new `InvalidName` error
    pub fn invalid_name(name: impl Into<String>) -> Self {
        Self::InvalidName(name.into())
    }

    /// Create a new `NameMismatch` error
    pub fn name_mismatch(dir_name: impl Into<String>, metadata_name: impl Into<String>) -> Self {
        Self::NameMismatch(dir_name.into(), metadata_name.into())
    }

    /// Create a new `InvalidDirectory` error
    pub fn invalid_directory(msg: impl Into<String>) -> Self {
        Self::InvalidDirectory(msg.into())
    }

    /// Create a new `NotFound` error
    pub fn not_found(name: impl Into<String>) -> Self {
        Self::NotFound(name.into())
    }

    /// Create a new `ToolNotAllowed` error
    pub fn tool_not_allowed(tool: impl Into<String>, allowed: Vec<String>) -> Self {
        Self::ToolNotAllowed(tool.into(), allowed)
    }

    /// Create a new `ScriptFailed` error
    pub fn script_failed(msg: impl Into<String>) -> Self {
        Self::ScriptFailed(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages() {
        let err = SkillError::invalid_name("InvalidName");
        assert!(err.to_string().contains("hyphen-case"));

        let err = SkillError::name_mismatch("dir-name", "skill-name");
        assert!(err.to_string().contains("dir-name"));
        assert!(err.to_string().contains("skill-name"));

        let err = SkillError::missing_field("description");
        assert!(err.to_string().contains("description"));
    }
}
