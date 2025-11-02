//! Core skill types and structures

use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::error::{Result, SkillError};
use crate::parser::parse_skill_file;
use crate::validation::{validate_name_matches_directory, validate_skill_name};

/// Maximum size for a SKILL.md file (10 MB)
///
/// This limit prevents memory exhaustion from extremely large skill definitions.
const MAX_SKILL_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Maximum size for reference files (50 MB)
///
/// Reference documents should not be excessively large.
const MAX_REFERENCE_FILE_SIZE: u64 = 50 * 1024 * 1024;

/// Metadata from the YAML frontmatter of a SKILL.md file
///
/// Corresponds to the Agent Skills Spec v1.0 (2025-10-16)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillMetadata {
    /// Skill name in hyphen-case (required)
    ///
    /// Must match directory name exactly.
    /// Valid pattern: `^[a-z0-9]+(-[a-z0-9]+)*$`
    pub name: String,

    /// Description of what the skill does and when to use it (required)
    ///
    /// Used for semantic matching and discovery.
    pub description: String,

    /// License information (optional)
    ///
    /// Common pattern: "MIT", "Apache-2.0", or "Complete terms in LICENSE.txt"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// List of tools that are pre-approved to run (optional)
    ///
    /// Semantics:
    /// - Missing field (None): ALL tools allowed
    /// - Empty array (Some(empty)): NO tools allowed
    /// - Non-empty array (Some(tools)): ONLY listed tools allowed
    #[serde(
        default,
        rename = "allowed-tools",
        skip_serializing_if = "Option::is_none"
    )]
    pub allowed_tools: Option<HashSet<String>>,

    /// Custom metadata fields (optional)
    ///
    /// Free-form key-value pairs for client use.
    /// Recommended: use unique key names to avoid conflicts.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_yaml::Value>,
}

impl SkillMetadata {
    /// Validate the metadata
    ///
    /// Checks:
    /// - Name format (hyphen-case)
    /// - Required fields present
    pub fn validate(&self) -> Result<()> {
        validate_skill_name(&self.name)?;
        Ok(())
    }

    /// Check if a tool is allowed by this skill
    ///
    /// Returns `true` if:
    /// - No `allowed_tools` list is specified (None = all tools allowed)
    /// - The tool is in the `allowed_tools` list (Some(set) where set contains tool)
    ///
    /// Returns `false` if:
    /// - `allowed_tools` is Some(empty) = no tools allowed
    /// - `allowed_tools` is Some(set) but tool is not in set
    #[must_use]
    pub fn allows_tool(&self, tool_name: &str) -> bool {
        match &self.allowed_tools {
            None => true,                             // Missing field = all tools allowed
            Some(tools) => tools.contains(tool_name), // Check if in set (empty set = none allowed)
        }
    }

    /// Get all allowed tools as a sorted vector
    ///
    /// Returns empty vec if:
    /// - All tools are allowed (None)
    /// - No tools are allowed (Some(empty))
    #[must_use]
    pub fn get_allowed_tools(&self) -> Vec<String> {
        match &self.allowed_tools {
            None => Vec::new(), // All allowed = return empty (no restrictions)
            Some(tools) => {
                let mut tools_vec: Vec<_> = tools.iter().cloned().collect();
                tools_vec.sort();
                tools_vec
            }
        }
    }
}

/// A reference document within a skill
#[derive(Debug, Clone)]
pub struct Reference {
    /// Path to the reference file (relative to skill root)
    pub path: PathBuf,

    /// Optional title extracted from the file
    pub title: Option<String>,

    /// Lazy-loaded content
    content: OnceCell<String>,
}

impl Reference {
    /// Create a new reference
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            title: None,
            content: OnceCell::new(),
        }
    }

    /// Load the reference content
    ///
    /// Content is cached after first load.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or exceeds the 50 MB limit.
    pub async fn content(&self) -> Result<&str> {
        if let Some(content) = self.content.get() {
            return Ok(content);
        }

        // Check file size to prevent memory exhaustion
        let metadata = tokio::fs::metadata(&self.path).await?;
        if metadata.len() > MAX_REFERENCE_FILE_SIZE {
            return Err(SkillError::ScriptExecution(format!(
                "Reference file is too large ({} bytes, max {} bytes): {}",
                metadata.len(),
                MAX_REFERENCE_FILE_SIZE,
                self.path.display()
            )));
        }

        let content = tokio::fs::read_to_string(&self.path).await?;
        let _ = self.content.set(content);

        Ok(self.content.get().expect("Just set"))
    }
}

/// A skill loaded from a SKILL.md file
#[derive(Debug, Clone)]
pub struct Skill {
    /// Metadata from YAML frontmatter
    pub metadata: SkillMetadata,

    /// Markdown body content
    pub content: String,

    /// Root directory of the skill
    pub root: PathBuf,

    /// Lazy-loaded references
    pub(crate) references: OnceCell<Vec<Reference>>,

    /// Lazy-loaded scripts
    pub(crate) scripts: OnceCell<HashMap<String, PathBuf>>,
}

impl Skill {
    /// Load a skill from a SKILL.md file
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - File cannot be read
    /// - File size exceeds 10 MB
    /// - YAML frontmatter is invalid
    /// - Required fields are missing
    /// - Name validation fails
    /// - Name doesn't match directory
    pub async fn from_file(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();

        // Check file size to prevent memory exhaustion
        let metadata = tokio::fs::metadata(&path).await?;
        if metadata.len() > MAX_SKILL_FILE_SIZE {
            return Err(SkillError::ScriptExecution(format!(
                "Skill file is too large ({} bytes, max {} bytes): {}",
                metadata.len(),
                MAX_SKILL_FILE_SIZE,
                path.display()
            )));
        }

        // Read file
        let content = tokio::fs::read_to_string(&path).await?;

        // Parse SKILL.md
        let (metadata, body) = parse_skill_file(&content)?;

        // Validate metadata
        metadata.validate()?;

        // Get skill root directory
        let root = path
            .parent()
            .ok_or_else(|| SkillError::invalid_directory("SKILL.md has no parent directory"))?
            .to_path_buf();

        // Validate name matches directory
        validate_name_matches_directory(&root, &metadata.name)?;

        Ok(Self {
            metadata,
            content: body,
            root,
            references: OnceCell::new(),
            scripts: OnceCell::new(),
        })
    }

    /// Get the full context for this skill (metadata + content)
    ///
    /// Formats as:
    /// ```markdown
    /// # Skill: skill-name
    ///
    /// {markdown body}
    /// ```
    pub fn context(&self) -> String {
        format!("# Skill: {}\n\n{}", self.metadata.name, self.content)
    }

    /// Check if a tool is allowed by this skill
    pub fn allows_tool(&self, tool_name: &str) -> bool {
        self.metadata.allows_tool(tool_name)
    }

    /// Get all allowed tools
    pub fn get_allowed_tools(&self) -> Vec<String> {
        self.metadata.get_allowed_tools()
    }

    /// Discover and cache references in the skill's reference/ directory
    ///
    /// # Errors
    ///
    /// Returns error if directory cannot be read.
    pub async fn references(&self) -> Result<&[Reference]> {
        if let Some(refs) = self.references.get() {
            return Ok(refs);
        }

        let ref_dir = self.root.join("reference");
        let references = if ref_dir.exists() {
            discover_references(&ref_dir).await?
        } else {
            Vec::new()
        };

        let _ = self.references.set(references);
        Ok(self.references.get().expect("Just set"))
    }

    /// Load a specific reference by relative path
    ///
    /// # Errors
    ///
    /// Returns error if reference doesn't exist or cannot be read.
    pub async fn load_reference(&self, relative_path: &str) -> Result<String> {
        let full_path = self.root.join("reference").join(relative_path);

        if !full_path.exists() {
            return Err(SkillError::ReferenceNotFound(full_path));
        }

        Ok(tokio::fs::read_to_string(full_path).await?)
    }

    /// Discover and cache scripts in the skill's scripts/ directory
    ///
    /// # Errors
    ///
    /// Returns error if directory cannot be read.
    pub async fn scripts(&self) -> Result<&HashMap<String, PathBuf>> {
        if let Some(scripts) = self.scripts.get() {
            return Ok(scripts);
        }

        let scripts_dir = self.root.join("scripts");
        let scripts = if scripts_dir.exists() {
            discover_scripts(&scripts_dir).await?
        } else {
            HashMap::new()
        };

        let _ = self.scripts.set(scripts);
        Ok(self.scripts.get().expect("Just set"))
    }

    /// Get a script path by name (without extension)
    ///
    /// # Errors
    ///
    /// Returns error if script doesn't exist.
    pub async fn get_script(&self, name: &str) -> Result<&PathBuf> {
        let scripts = self.scripts().await?;
        scripts
            .get(name)
            .ok_or_else(|| SkillError::ScriptNotFound(self.root.join("scripts").join(name)))
    }

    /// Execute a script by name
    ///
    /// Finds the script in the scripts/ directory and executes it with the
    /// appropriate executor (Python for .py, Bash for .sh).
    ///
    /// # Arguments
    ///
    /// * `script_name` - Name of the script (without extension)
    /// * `args` - Command-line arguments to pass to the script
    /// * `timeout` - Maximum execution time (default: 30 seconds if None)
    ///
    /// # Returns
    ///
    /// `ScriptOutput` containing exit code, stdout, stderr, duration, and timeout status
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Script not found
    /// - Script fails to execute
    /// - Unsupported script type
    ///
    /// Note: Timeout is not an error - it returns Ok(ScriptOutput) with `timed_out=true`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use turboclaude_skills::Skill;
    /// # use std::time::Duration;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let skill = Skill::from_file("./skills/pdf/SKILL.md").await?;
    ///
    /// // Execute with default timeout (30 seconds)
    /// let output = skill.execute_script("process", &["input.pdf"], None).await?;
    ///
    /// if output.success() {
    ///     println!("Success: {}", output.stdout);
    /// } else if output.timed_out {
    ///     println!("Script timed out");
    /// } else {
    ///     println!("Failed with code {}: {}", output.exit_code, output.stderr);
    /// }
    ///
    /// // Execute with custom timeout
    /// let output = skill.execute_script(
    ///     "process",
    ///     &["input.pdf"],
    ///     Some(Duration::from_secs(60))
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_script(
        &self,
        script_name: &str,
        args: &[&str],
        timeout: Option<std::time::Duration>,
    ) -> Result<crate::executor::ScriptOutput> {
        use crate::executor::{CompositeExecutor, ScriptExecutor};

        // Get script path
        let script_path = self.get_script(script_name).await?;

        // Create executor
        let executor = CompositeExecutor::new();

        // Execute with timeout (default: 30 seconds)
        let timeout_duration = timeout.unwrap_or(std::time::Duration::from_secs(30));

        executor.execute(script_path, args, timeout_duration).await
    }

    /// List all available scripts
    ///
    /// Returns the names of all scripts in the scripts/ directory (without extensions).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use turboclaude_skills::Skill;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let skill = Skill::from_file("./skills/pdf/SKILL.md").await?;
    /// let scripts = skill.list_scripts().await?;
    /// println!("Available scripts: {:?}", scripts);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_scripts(&self) -> Result<Vec<String>> {
        let scripts = self.scripts().await?;
        Ok(scripts.keys().cloned().collect())
    }

    /// Check if a script exists
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use turboclaude_skills::Skill;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let skill = Skill::from_file("./skills/pdf/SKILL.md").await?;
    /// if skill.has_script("process").await? {
    ///     println!("Process script exists");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn has_script(&self, script_name: &str) -> Result<bool> {
        let scripts = self.scripts().await?;
        Ok(scripts.contains_key(script_name))
    }
}

/// Discover all markdown files in a reference directory
async fn discover_references(dir: &PathBuf) -> Result<Vec<Reference>> {
    let mut references = Vec::new();

    let mut entries = tokio::fs::read_dir(dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
            references.push(Reference::new(path));
        }
    }

    Ok(references)
}

/// Discover all scripts (.py, .sh) in a scripts directory
async fn discover_scripts(dir: &PathBuf) -> Result<HashMap<String, PathBuf>> {
    let mut scripts = HashMap::new();

    let mut entries = tokio::fs::read_dir(dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if path.is_file()
            && let Some(ext) = path.extension()
            && (ext == "py" || ext == "sh")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
        {
            scripts.insert(stem.to_string(), path);
        }
    }

    Ok(scripts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_metadata_allows_tool() {
        // No restrictions (None = all allowed)
        let metadata = SkillMetadata {
            name: "test".to_string(),
            description: "Test".to_string(),
            license: None,
            allowed_tools: None,
            metadata: HashMap::new(),
        };
        assert!(metadata.allows_tool("bash"));
        assert!(metadata.allows_tool("read"));
        assert!(metadata.allows_tool("any-tool"));

        // Empty set (Some(empty) = none allowed)
        let metadata = SkillMetadata {
            name: "test".to_string(),
            description: "Test".to_string(),
            license: None,
            allowed_tools: Some(HashSet::new()),
            metadata: HashMap::new(),
        };
        assert!(!metadata.allows_tool("bash"));
        assert!(!metadata.allows_tool("read"));
        assert!(!metadata.allows_tool("any-tool"));

        // Specific tools allowed
        let mut allowed_tools = HashSet::new();
        allowed_tools.insert("bash".to_string());
        allowed_tools.insert("read".to_string());

        let metadata = SkillMetadata {
            name: "test".to_string(),
            description: "Test".to_string(),
            license: None,
            allowed_tools: Some(allowed_tools),
            metadata: HashMap::new(),
        };
        assert!(metadata.allows_tool("bash"));
        assert!(metadata.allows_tool("read"));
        assert!(!metadata.allows_tool("write"));
        assert!(!metadata.allows_tool("dangerous"));
    }

    #[test]
    fn test_skill_metadata_get_allowed_tools() {
        // None = all allowed, return empty vec
        let metadata = SkillMetadata {
            name: "test".to_string(),
            description: "Test".to_string(),
            license: None,
            allowed_tools: None,
            metadata: HashMap::new(),
        };
        assert_eq!(metadata.get_allowed_tools(), Vec::<String>::new());

        // Some with tools = return sorted
        let mut allowed_tools = HashSet::new();
        allowed_tools.insert("write".to_string());
        allowed_tools.insert("bash".to_string());
        allowed_tools.insert("read".to_string());

        let metadata = SkillMetadata {
            name: "test".to_string(),
            description: "Test".to_string(),
            license: None,
            allowed_tools: Some(allowed_tools),
            metadata: HashMap::new(),
        };

        let tools = metadata.get_allowed_tools();
        // Should be sorted
        assert_eq!(tools, vec!["bash", "read", "write"]);
    }
}
