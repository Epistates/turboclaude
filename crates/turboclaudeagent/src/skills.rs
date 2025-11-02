//! Skill management for agent sessions
//!
//! Provides skill discovery, loading, and validation for agent sessions.
//! Skills are optional capability packages that can be loaded at runtime to:
//! - Inject context into agent prompts
//! - Restrict tool usage via allowed-tools
//! - Provide reference documentation
//!
//! # Example
//!
//! ```no_run
//! # use turboclaudeagent::{ClaudeAgentClient, SessionConfig};
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut config = SessionConfig::default();
//! config.skill_dirs = vec!["./skills".into()];
//!
//! let client = ClaudeAgentClient::new(config);
//! let session = client.create_session().await?;
//!
//! // Discover all available skills
//! let result = session.discover_skills().await?;
//! println!("Found {} skills", result.loaded);
//!
//! // Load a specific skill
//! session.load_skill("pdf").await?;
//!
//! // Query with skill context (automatically injected)
//! let response = session.query_str("Process this PDF").await?;
//! # Ok(())
//! # }
//! ```

use crate::error::{AgentError, Result as AgentResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use turboclaude_skills::{Skill, SkillRegistry};

/// An active skill in the session
///
/// Tracks usage metrics for an activated skill.
#[derive(Debug, Clone)]
pub struct ActiveSkill {
    /// The skill metadata and content
    pub skill: Skill,

    /// When this skill was activated
    pub activated_at: Instant,

    /// Number of queries that used this skill
    pub usage_count: u32,
}

/// Result of skill discovery operation
#[derive(Debug, Clone)]
pub struct SkillDiscoveryResult {
    /// Number of skills successfully loaded
    pub loaded: usize,

    /// Number of directories that failed to scan
    pub failed: usize,

    /// Errors encountered during discovery (as formatted strings)
    pub errors: Vec<String>,
}

/// Tool validation result
///
/// Indicates whether a tool is allowed by the currently active skills.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolValidationResult {
    /// Whether the tool is allowed
    pub allowed: bool,

    /// The tool that was checked
    pub tool_name: String,

    /// The skill that blocked it (if any)
    pub blocked_by: Option<String>,
}

/// Manages skills for an agent session
///
/// Provides:
/// - Skill discovery from filesystem
/// - Loading/unloading skills at runtime
/// - Building context strings for prompt injection
/// - Tool validation against allowed-tools constraints
pub struct SkillManager {
    /// Registry for skill discovery
    registry: Arc<RwLock<SkillRegistry>>,

    /// Active skills (name -> ActiveSkill)
    active_skills: Arc<RwLock<HashMap<String, ActiveSkill>>>,
}

impl SkillManager {
    /// Create a new skill manager
    ///
    /// # Errors
    ///
    /// Returns error if the skill registry cannot be initialized.
    pub async fn new(registry: SkillRegistry) -> AgentResult<Self> {
        Ok(Self {
            registry: Arc::new(RwLock::new(registry)),
            active_skills: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Discover skills from configured directories
    ///
    /// Scans all skill directories for SKILL.md files and loads them into the registry.
    ///
    /// # Errors
    ///
    /// Returns error if discovery fails (filesystem issues, parse errors, etc).
    pub async fn discover(&self) -> AgentResult<SkillDiscoveryResult> {
        let mut registry = self.registry.write().await;

        let report = registry
            .discover()
            .await
            .map_err(|e| AgentError::Config(format!("Skill discovery failed: {}", e)))?;

        // Convert errors to strings for API convenience
        let errors = report
            .errors
            .into_iter()
            .map(|(path, err)| format!("{}: {}", path.display(), err))
            .collect();

        Ok(SkillDiscoveryResult {
            loaded: report.loaded,
            failed: report.failed,
            errors,
        })
    }

    /// Load (activate) a skill by name
    ///
    /// Once loaded, the skill's context will be automatically injected into all queries
    /// and its allowed-tools constraints will be enforced.
    ///
    /// # Errors
    ///
    /// Returns error if the skill is not found in the registry.
    pub async fn load(&self, name: &str) -> AgentResult<()> {
        let registry = self.registry.read().await;

        let skill = registry
            .get(name)
            .await
            .map_err(|e| AgentError::Config(format!("Skill '{}' not found: {}", name, e)))?;

        let mut active = self.active_skills.write().await;
        active.insert(
            name.to_string(),
            ActiveSkill {
                skill,
                activated_at: Instant::now(),
                usage_count: 0,
            },
        );

        Ok(())
    }

    /// Unload (deactivate) a skill by name
    ///
    /// Removes the skill from the active set. Future queries will not include
    /// this skill's context or constraints.
    ///
    /// # Errors
    ///
    /// Returns error if the skill is not currently active.
    pub async fn unload(&self, name: &str) -> AgentResult<()> {
        let mut active = self.active_skills.write().await;

        if active.remove(name).is_none() {
            return Err(AgentError::Config(format!(
                "Skill '{}' is not active",
                name
            )));
        }

        Ok(())
    }

    /// List active skill names
    ///
    /// Returns the names of all currently loaded skills.
    pub async fn list_active(&self) -> Vec<String> {
        let active = self.active_skills.read().await;
        active.keys().cloned().collect()
    }

    /// List all available skills in registry
    ///
    /// Returns the names of all skills that have been discovered and are available
    /// to load.
    pub async fn list_available(&self) -> Vec<String> {
        let registry = self.registry.read().await;
        registry.list().await.into_iter().map(|m| m.name).collect()
    }

    /// Find skills matching a query
    ///
    /// Performs semantic search across all available skills.
    ///
    /// # Errors
    ///
    /// Returns error if the search operation fails.
    pub async fn find(&self, query: &str) -> AgentResult<Vec<Skill>> {
        let registry = self.registry.read().await;
        registry
            .find(query)
            .await
            .map_err(|e| AgentError::Config(format!("Skill search failed: {}", e)))
    }

    /// Get skill by name
    ///
    /// Retrieves the full skill object from the registry.
    ///
    /// # Errors
    ///
    /// Returns error if the skill is not found.
    pub async fn get(&self, name: &str) -> AgentResult<Skill> {
        let registry = self.registry.read().await;
        registry
            .get(name)
            .await
            .map_err(|e| AgentError::Config(format!("Skill '{}' not found: {}", name, e)))
    }

    /// Build system prompt context from active skills
    ///
    /// Concatenates the content from all active skills into a single context string
    /// suitable for appending to the agent's system prompt.
    ///
    /// Returns an empty string if no skills are active.
    pub async fn build_context(&self) -> String {
        let active = self.active_skills.read().await;

        if active.is_empty() {
            return String::new();
        }

        let mut context = String::from("\n\n# Available Skills\n\n");
        context.push_str("You have access to the following skills:\n\n");

        for (name, active_skill) in active.iter() {
            context.push_str(&format!("## Skill: {}\n\n", name));
            context.push_str(&active_skill.skill.content);
            context.push_str("\n\n---\n\n");
        }

        context
    }

    /// Validate if a tool is allowed by active skills
    ///
    /// Checks the given tool name against the allowed-tools constraints of all
    /// active skills. If any skill blocks the tool, validation fails.
    ///
    /// # Rules
    ///
    /// - If no skills are active, all tools are allowed
    /// - If any active skill has allowed-tools = None, that skill allows all tools
    /// - If any active skill has allowed-tools = Some([]), that skill allows NO tools
    /// - If any active skill has allowed-tools = Some([tools]), the tool must be in the list
    ///
    /// Returns the validation result indicating whether the tool is allowed and
    /// which skill blocked it (if any).
    pub async fn validate_tool(&self, tool_name: &str) -> ToolValidationResult {
        let active = self.active_skills.read().await;

        // If no skills are active, all tools are allowed
        if active.is_empty() {
            return ToolValidationResult {
                allowed: true,
                tool_name: tool_name.to_string(),
                blocked_by: None,
            };
        }

        // Check each active skill
        for (name, active_skill) in active.iter() {
            if !active_skill.skill.metadata.allows_tool(tool_name) {
                return ToolValidationResult {
                    allowed: false,
                    tool_name: tool_name.to_string(),
                    blocked_by: Some(name.clone()),
                };
            }
        }

        ToolValidationResult {
            allowed: true,
            tool_name: tool_name.to_string(),
            blocked_by: None,
        }
    }

    /// Increment usage count for all active skills
    ///
    /// Called after each query to track skill usage metrics.
    pub async fn increment_usage(&self) {
        let mut active = self.active_skills.write().await;

        for active_skill in active.values_mut() {
            active_skill.usage_count += 1;
        }
    }

    /// Get usage stats for an active skill
    ///
    /// Returns None if the skill is not active.
    pub async fn get_usage(&self, name: &str) -> Option<(Instant, u32)> {
        let active = self.active_skills.read().await;

        active.get(name).map(|s| (s.activated_at, s.usage_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_discovery_result() {
        let result = SkillDiscoveryResult {
            loaded: 4,
            failed: 1,
            errors: vec!["Error 1".to_string()],
        };

        assert_eq!(result.loaded, 4);
        assert_eq!(result.failed, 1);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_tool_validation_result() {
        let allowed = ToolValidationResult {
            allowed: true,
            tool_name: "bash".to_string(),
            blocked_by: None,
        };

        assert!(allowed.allowed);
        assert_eq!(allowed.tool_name, "bash");
        assert!(allowed.blocked_by.is_none());

        let blocked = ToolValidationResult {
            allowed: false,
            tool_name: "dangerous".to_string(),
            blocked_by: Some("pdf-skill".to_string()),
        };

        assert!(!blocked.allowed);
        assert_eq!(blocked.blocked_by, Some("pdf-skill".to_string()));
    }

    #[tokio::test]
    async fn test_active_skill_creation() {
        use std::fs;
        use tempfile::TempDir;

        // Create a temporary skill directory
        let temp_dir = TempDir::new().unwrap();
        let skill_path = temp_dir.path().join("test");
        fs::create_dir(&skill_path).unwrap();

        // Write a SKILL.md file
        let skill_file = skill_path.join("SKILL.md");
        let skill_content = r#"---
name: test
description: Test skill
---

Test content"#;
        fs::write(&skill_file, skill_content).unwrap();

        // Load the skill
        let skill = turboclaude_skills::Skill::from_file(&skill_file)
            .await
            .unwrap();

        let active = ActiveSkill {
            skill: skill.clone(),
            activated_at: Instant::now(),
            usage_count: 0,
        };

        assert_eq!(active.skill.metadata.name, "test");
        assert_eq!(active.usage_count, 0);
    }
}
