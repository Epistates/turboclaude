//! Permission update types and definitions
//!
//! Provides types for dynamic permission changes during agent sessions.
//! Matches the Python SDK implementation (types.py:56-108).

use serde::{Deserialize, Serialize};

/// Permission behavior for rule-based updates
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PermissionBehavior {
    /// Allow the action
    Allow,

    /// Deny the action
    Deny,

    /// Ask for permission
    Ask,
}

/// Destination for permission updates
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PermissionUpdateDestination {
    /// Update user settings
    UserSettings,

    /// Update project settings
    ProjectSettings,

    /// Update local settings
    LocalSettings,

    /// Update session only (non-persistent)
    Session,
}

/// Permission rule value
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermissionRuleValue {
    /// Tool name for the rule
    #[serde(rename = "toolName")]
    pub tool_name: String,

    /// Optional rule content (regex pattern or other constraint)
    #[serde(rename = "ruleContent", skip_serializing_if = "Option::is_none")]
    pub rule_content: Option<String>,
}

impl PermissionRuleValue {
    /// Create a new permission rule
    pub fn new(tool_name: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            rule_content: None,
        }
    }

    /// Set the rule content
    pub fn with_rule_content(mut self, content: impl Into<String>) -> Self {
        self.rule_content = Some(content.into());
        self
    }
}

/// Add rules update
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AddRulesUpdate {
    /// Rules to add
    pub rules: Vec<PermissionRuleValue>,

    /// Behavior for the rules
    pub behavior: PermissionBehavior,

    /// Optional destination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<PermissionUpdateDestination>,
}

/// Replace rules update
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReplaceRulesUpdate {
    /// Rules to replace with
    pub rules: Vec<PermissionRuleValue>,

    /// Behavior for the rules
    pub behavior: PermissionBehavior,

    /// Optional destination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<PermissionUpdateDestination>,
}

/// Remove rules update
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RemoveRulesUpdate {
    /// Rules to remove
    pub rules: Vec<PermissionRuleValue>,

    /// Optional destination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<PermissionUpdateDestination>,
}

/// Set mode update
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetModeUpdate {
    /// Permission mode to set
    pub mode: crate::types::PermissionMode,

    /// Optional destination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<PermissionUpdateDestination>,
}

/// Add directories update
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AddDirectoriesUpdate {
    /// Directories to add
    pub directories: Vec<String>,

    /// Optional destination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<PermissionUpdateDestination>,
}

/// Remove directories update
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RemoveDirectoriesUpdate {
    /// Directories to remove
    pub directories: Vec<String>,

    /// Optional destination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<PermissionUpdateDestination>,
}

/// Permission update type
///
/// Represents dynamic permission changes during an agent session.
/// Matches Python SDK implementation (types.py:56-108).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PermissionUpdate {
    /// Add permission rules
    AddRules(AddRulesUpdate),

    /// Replace permission rules
    ReplaceRules(ReplaceRulesUpdate),

    /// Remove permission rules
    RemoveRules(RemoveRulesUpdate),

    /// Set permission mode
    SetMode(SetModeUpdate),

    /// Add allowed directories
    AddDirectories(AddDirectoriesUpdate),

    /// Remove allowed directories
    RemoveDirectories(RemoveDirectoriesUpdate),
}

impl PermissionUpdate {
    /// Create an add rules update
    pub fn add_rules(rules: Vec<PermissionRuleValue>, behavior: PermissionBehavior) -> Self {
        Self::AddRules(AddRulesUpdate {
            rules,
            behavior,
            destination: None,
        })
    }

    /// Create a replace rules update
    pub fn replace_rules(rules: Vec<PermissionRuleValue>, behavior: PermissionBehavior) -> Self {
        Self::ReplaceRules(ReplaceRulesUpdate {
            rules,
            behavior,
            destination: None,
        })
    }

    /// Create a remove rules update
    pub fn remove_rules(rules: Vec<PermissionRuleValue>) -> Self {
        Self::RemoveRules(RemoveRulesUpdate {
            rules,
            destination: None,
        })
    }

    /// Create a set mode update
    pub fn set_mode(mode: crate::types::PermissionMode) -> Self {
        Self::SetMode(SetModeUpdate {
            mode,
            destination: None,
        })
    }

    /// Create an add directories update
    pub fn add_directories(directories: Vec<String>) -> Self {
        Self::AddDirectories(AddDirectoriesUpdate {
            directories,
            destination: None,
        })
    }

    /// Create a remove directories update
    pub fn remove_directories(directories: Vec<String>) -> Self {
        Self::RemoveDirectories(RemoveDirectoriesUpdate {
            directories,
            destination: None,
        })
    }

    /// Set the destination for this update
    pub fn with_destination(mut self, destination: PermissionUpdateDestination) -> Self {
        match &mut self {
            Self::AddRules(u) => u.destination = Some(destination),
            Self::ReplaceRules(u) => u.destination = Some(destination),
            Self::RemoveRules(u) => u.destination = Some(destination),
            Self::SetMode(u) => u.destination = Some(destination),
            Self::AddDirectories(u) => u.destination = Some(destination),
            Self::RemoveDirectories(u) => u.destination = Some(destination),
        }
        self
    }

    /// Get the destination for this update
    pub fn destination(&self) -> Option<PermissionUpdateDestination> {
        match self {
            Self::AddRules(u) => u.destination,
            Self::ReplaceRules(u) => u.destination,
            Self::RemoveRules(u) => u.destination,
            Self::SetMode(u) => u.destination,
            Self::AddDirectories(u) => u.destination,
            Self::RemoveDirectories(u) => u.destination,
        }
    }

    /// Validate the update
    ///
    /// Returns `Ok(())` if the update is valid, or an error message if not.
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::AddRules(u) => {
                if u.rules.is_empty() {
                    return Err("AddRules update must have at least one rule".to_string());
                }
                for rule in &u.rules {
                    if rule.tool_name.is_empty() {
                        return Err("Rule tool_name cannot be empty".to_string());
                    }
                }
                Ok(())
            }
            Self::ReplaceRules(u) => {
                if u.rules.is_empty() {
                    return Err("ReplaceRules update must have at least one rule".to_string());
                }
                for rule in &u.rules {
                    if rule.tool_name.is_empty() {
                        return Err("Rule tool_name cannot be empty".to_string());
                    }
                }
                Ok(())
            }
            Self::RemoveRules(u) => {
                if u.rules.is_empty() {
                    return Err("RemoveRules update must have at least one rule".to_string());
                }
                for rule in &u.rules {
                    if rule.tool_name.is_empty() {
                        return Err("Rule tool_name cannot be empty".to_string());
                    }
                }
                Ok(())
            }
            Self::SetMode(_) => Ok(()),
            Self::AddDirectories(u) => {
                if u.directories.is_empty() {
                    return Err(
                        "AddDirectories update must have at least one directory".to_string()
                    );
                }
                for dir in &u.directories {
                    if dir.is_empty() {
                        return Err("Directory path cannot be empty".to_string());
                    }
                }
                Ok(())
            }
            Self::RemoveDirectories(u) => {
                if u.directories.is_empty() {
                    return Err(
                        "RemoveDirectories update must have at least one directory".to_string()
                    );
                }
                for dir in &u.directories {
                    if dir.is_empty() {
                        return Err("Directory path cannot be empty".to_string());
                    }
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PermissionMode;

    #[test]
    fn test_permission_rule_value() {
        let rule = PermissionRuleValue::new("bash").with_rule_content(".*");

        assert_eq!(rule.tool_name, "bash");
        assert_eq!(rule.rule_content, Some(".*".to_string()));
    }

    #[test]
    fn test_add_rules_update() {
        let rules = vec![
            PermissionRuleValue::new("bash"),
            PermissionRuleValue::new("file_editor"),
        ];

        let update = PermissionUpdate::add_rules(rules, PermissionBehavior::Allow)
            .with_destination(PermissionUpdateDestination::Session);

        assert!(matches!(update, PermissionUpdate::AddRules(_)));
        assert_eq!(
            update.destination(),
            Some(PermissionUpdateDestination::Session)
        );
        assert!(update.validate().is_ok());
    }

    #[test]
    fn test_set_mode_update() {
        let update = PermissionUpdate::set_mode(PermissionMode::BypassPermissions)
            .with_destination(PermissionUpdateDestination::ProjectSettings);

        assert!(matches!(update, PermissionUpdate::SetMode(_)));
        assert!(update.validate().is_ok());
    }

    #[test]
    fn test_add_directories_update() {
        let dirs = vec!["/home/user/project".to_string()];
        let update = PermissionUpdate::add_directories(dirs);

        assert!(matches!(update, PermissionUpdate::AddDirectories(_)));
        assert!(update.validate().is_ok());
    }

    #[test]
    fn test_validation_empty_rules() {
        let update = PermissionUpdate::add_rules(vec![], PermissionBehavior::Allow);
        assert!(update.validate().is_err());
    }

    #[test]
    fn test_validation_empty_directories() {
        let update = PermissionUpdate::add_directories(vec![]);
        assert!(update.validate().is_err());
    }

    #[test]
    fn test_validation_empty_tool_name() {
        let rules = vec![PermissionRuleValue::new("")];
        let update = PermissionUpdate::add_rules(rules, PermissionBehavior::Allow);
        assert!(update.validate().is_err());
    }

    #[test]
    fn test_serialization_add_rules() {
        let rules = vec![PermissionRuleValue::new("bash")];
        let update = PermissionUpdate::add_rules(rules, PermissionBehavior::Allow);

        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("addRules"));
        assert!(json.contains("bash"));

        let deserialized: PermissionUpdate = serde_json::from_str(&json).unwrap();
        assert_eq!(update, deserialized);
    }

    #[test]
    fn test_serialization_set_mode() {
        let update = PermissionUpdate::set_mode(PermissionMode::AcceptEdits);

        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("setMode"));

        let deserialized: PermissionUpdate = serde_json::from_str(&json).unwrap();
        assert_eq!(update, deserialized);
    }

    #[test]
    fn test_serialization_with_destination() {
        let rules = vec![PermissionRuleValue::new("bash")];
        let update = PermissionUpdate::add_rules(rules, PermissionBehavior::Allow)
            .with_destination(PermissionUpdateDestination::Session);

        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("destination"));
        assert!(json.contains("session"));

        let deserialized: PermissionUpdate = serde_json::from_str(&json).unwrap();
        assert_eq!(update, deserialized);
    }
}
