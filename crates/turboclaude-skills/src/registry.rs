//! Skill registry for discovery and management

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use walkdir::WalkDir;

use crate::error::{Result, SkillError};
use crate::matcher::{KeywordMatcher, SkillMatcher};
use crate::skill::{Skill, SkillMetadata};

/// Registry for discovering and managing skills
///
/// Provides:
/// - Discovery via directory scanning
/// - In-memory caching
/// - Retrieval by name or semantic search
/// - Thread-safe concurrent access
#[derive(Clone)]
pub struct SkillRegistry {
    /// Cached skills (name → skill)
    skills: Arc<RwLock<HashMap<String, Skill>>>,

    /// Directories to scan for skills
    skill_dirs: Vec<PathBuf>,

    /// Matcher for semantic search
    matcher: Arc<dyn SkillMatcher>,
}

impl SkillRegistry {
    /// Create a new builder for configuring the registry
    #[must_use]
    pub fn builder() -> SkillRegistryBuilder {
        SkillRegistryBuilder::default()
    }

    /// Discover all skills in configured directories
    ///
    /// Scans each directory recursively for SKILL.md files.
    /// Invalid skills are logged and skipped.
    ///
    /// # Errors
    ///
    /// Returns error if directories cannot be accessed.
    pub async fn discover(&mut self) -> Result<DiscoveryReport> {
        let mut report = DiscoveryReport::default();

        for skill_dir in &self.skill_dirs {
            match discover_in_dir(skill_dir).await {
                Ok(skills) => {
                    report.loaded += skills.len();
                    let mut cache = self.skills.write().await;
                    for skill in skills {
                        cache.insert(skill.metadata.name.clone(), skill);
                    }
                }
                Err(e) => {
                    report.errors.push((skill_dir.clone(), e));
                    report.failed += 1;
                }
            }
        }

        Ok(report)
    }

    /// Get a skill by exact name
    ///
    /// # Errors
    ///
    /// Returns `SkillError::NotFound` if skill doesn't exist.
    pub async fn get(&self, name: &str) -> Result<Skill> {
        let skills = self.skills.read().await;
        skills
            .get(name)
            .cloned()
            .ok_or_else(|| SkillError::not_found(name))
    }

    /// Find skills matching a query (semantic search)
    ///
    /// Uses the configured matcher to find relevant skills.
    pub async fn find(&self, query: &str) -> Result<Vec<Skill>> {
        let skills = self.skills.read().await;
        let skill_vec: Vec<Skill> = skills.values().cloned().collect();
        self.matcher.find_matching(&skill_vec, query).await
    }

    /// List all available skills (metadata only)
    pub async fn list(&self) -> Vec<SkillMetadata> {
        let skills = self.skills.read().await;
        skills.values().map(|s| s.metadata.clone()).collect()
    }

    /// Check if a skill exists
    pub async fn contains(&self, name: &str) -> bool {
        let skills = self.skills.read().await;
        skills.contains_key(name)
    }

    /// Get the number of loaded skills
    pub async fn len(&self) -> usize {
        let skills = self.skills.read().await;
        skills.len()
    }

    /// Check if the registry is empty
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}

/// Report from skill discovery operation
#[derive(Debug, Default)]
pub struct DiscoveryReport {
    /// Number of skills successfully loaded
    pub loaded: usize,

    /// Number of directories that failed to scan
    pub failed: usize,

    /// Errors encountered during discovery
    pub errors: Vec<(PathBuf, SkillError)>,
}

impl DiscoveryReport {
    /// Check if discovery completed without errors
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get total directories processed
    #[must_use]
    pub fn total(&self) -> usize {
        self.loaded + self.failed
    }
}

/// Builder for configuring a `SkillRegistry`
#[derive(Default)]
pub struct SkillRegistryBuilder {
    skill_dirs: Vec<PathBuf>,
    matcher: Option<Arc<dyn SkillMatcher>>,
}

impl SkillRegistryBuilder {
    /// Add a skill directory to scan
    #[must_use]
    pub fn skill_dir(mut self, dir: PathBuf) -> Self {
        self.skill_dirs.push(dir);
        self
    }

    /// Add multiple skill directories
    #[must_use]
    pub fn skill_dirs(mut self, dirs: Vec<PathBuf>) -> Self {
        self.skill_dirs.extend(dirs);
        self
    }

    /// Set the matcher for semantic search (default: `KeywordMatcher`)
    #[must_use]
    pub fn matcher(mut self, matcher: Arc<dyn SkillMatcher>) -> Self {
        self.matcher = Some(matcher);
        self
    }

    /// Build the registry
    ///
    /// # Errors
    ///
    /// Returns error if no skill directories are configured.
    pub fn build(self) -> Result<SkillRegistry> {
        if self.skill_dirs.is_empty() {
            return Err(SkillError::invalid_directory(
                "No skill directories configured",
            ));
        }

        Ok(SkillRegistry {
            skills: Arc::new(RwLock::new(HashMap::new())),
            skill_dirs: self.skill_dirs,
            matcher: self.matcher.unwrap_or_else(|| Arc::new(KeywordMatcher)),
        })
    }
}

/// Discover skills in a single directory
async fn discover_in_dir(dir: &PathBuf) -> Result<Vec<Skill>> {
    if !dir.exists() {
        return Err(SkillError::invalid_directory(format!(
            "Directory does not exist: {}",
            dir.display()
        )));
    }

    if !dir.is_dir() {
        return Err(SkillError::invalid_directory(format!(
            "Not a directory: {}",
            dir.display()
        )));
    }

    let mut skills = Vec::new();

    // Walk directory tree looking for SKILL.md files
    for entry in WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
    {
        let entry = entry?;
        let path = entry.path();

        // Check if this is a SKILL.md file
        if path.is_file() && path.file_name() == Some(std::ffi::OsStr::new("SKILL.md")) {
            match Skill::from_file(path).await {
                Ok(skill) => {
                    skills.push(skill);
                }
                Err(e) => {
                    // Log error but continue discovering other skills
                    eprintln!("Warning: Failed to load skill from {}: {e}", path.display());
                }
            }
        }
    }

    Ok(skills)
}

/// Check if a directory entry should be skipped (hidden files/dirs)
fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .is_some_and(|s| s.starts_with('.'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_requires_skill_dirs() {
        let result = SkillRegistry::builder().build();
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_with_single_dir() {
        let result = SkillRegistry::builder()
            .skill_dir(PathBuf::from("./skills"))
            .build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_with_multiple_dirs() {
        let result = SkillRegistry::builder()
            .skill_dirs(vec![
                PathBuf::from("./skills"),
                PathBuf::from("/usr/local/share/skills"),
            ])
            .build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_hidden() {
        let temp_dir = tempfile::tempdir().unwrap();
        let hidden = temp_dir.path().join(".hidden");
        std::fs::create_dir(&hidden).unwrap();

        for entry in WalkDir::new(temp_dir.path()) {
            let entry = entry.unwrap();
            if entry.file_name() == ".hidden" {
                assert!(is_hidden(&entry));
            }
        }
    }
}
