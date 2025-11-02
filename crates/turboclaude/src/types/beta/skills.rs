//! Types for the Beta Skills API
//!
//! Skills enable agent capabilities with low-latency tool integration.
//! This module provides types for creating, managing, and versioning skills.

use serde::{Deserialize, Serialize};

/// A skill object returned by the API.
///
/// Skills are reusable capabilities that can be attached to messages
/// for agent functionality. Each skill has a unique ID and can have
/// multiple versions over time.
///
/// # Example Response
///
/// ```json
/// {
///   "id": "skill_01ABC",
///   "created_at": "2025-01-15T10:30:00Z",
///   "display_title": "Weather Lookup",
///   "latest_version": "1759178010641129",
///   "source": "custom",
///   "type": "skill",
///   "updated_at": "2025-01-15T10:30:00Z"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(Default))]
pub struct Skill {
    /// Unique identifier for the skill.
    ///
    /// The format and length of IDs may change over time.
    pub id: String,

    /// ISO 8601 timestamp of when the skill was created.
    pub created_at: String,

    /// Display title for the skill.
    ///
    /// This is a human-readable label that is not included in the
    /// prompt sent to the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_title: Option<String>,

    /// The latest version identifier for the skill.
    ///
    /// This represents the most recent version of the skill that has
    /// been created. Version identifiers are Unix epoch timestamps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_version: Option<String>,

    /// Source of the skill.
    ///
    /// Possible values:
    /// - `"custom"`: the skill was created by a user
    /// - `"anthropic"`: the skill was created by Anthropic
    pub source: String,

    /// Object type. For Skills, this is always `"skill"`.
    #[serde(rename = "type")]
    pub type_: String,

    /// ISO 8601 timestamp of when the skill was last updated.
    pub updated_at: String,
}

/// A skill version object.
///
/// Each skill can have multiple versions, allowing you to iterate on
/// functionality while maintaining stable references. Versions are
/// identified by Unix epoch timestamps.
///
/// # Example Response
///
/// ```json
/// {
///   "id": "skill_version_01XYZ",
///   "created_at": "2025-01-15T10:30:00Z",
///   "description": "Retrieves current weather for a location",
///   "directory": "weather_skill",
///   "name": "Weather Lookup",
///   "skill_id": "skill_01ABC",
///   "type": "skill_version",
///   "version": "1759178010641129"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(Default))]
pub struct SkillVersion {
    /// Unique identifier for the skill version.
    ///
    /// The format and length of IDs may change over time.
    pub id: String,

    /// ISO 8601 timestamp of when the skill version was created.
    pub created_at: String,

    /// Description of the skill version.
    ///
    /// This is extracted from the SKILL.md file in the skill upload.
    pub description: String,

    /// Directory name of the skill version.
    ///
    /// This is the top-level directory name that was extracted from
    /// the uploaded files.
    pub directory: String,

    /// Human-readable name of the skill version.
    ///
    /// This is extracted from the SKILL.md file in the skill upload.
    pub name: String,

    /// Identifier for the skill that this version belongs to.
    pub skill_id: String,

    /// Object type. For Skill Versions, this is always `"skill_version"`.
    #[serde(rename = "type")]
    pub type_: String,

    /// Version identifier for the skill.
    ///
    /// Each version is identified by a Unix epoch timestamp in
    /// microseconds (e.g., "1759178010641129").
    pub version: String,
}

/// Response from a delete operation.
///
/// Returned when successfully deleting a skill or skill version.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeletedObject {
    /// ID of the deleted object.
    pub id: String,

    /// Whether the deletion was successful.
    pub deleted: bool,

    /// Type of the deleted object (`"skill"` or `"skill_version"`).
    #[serde(rename = "type")]
    pub type_: String,
}

/// Source filter for listing skills.
///
/// Used to filter skills by who created them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillSource {
    /// Only return user-created skills.
    Custom,

    /// Only return Anthropic-created skills.
    Anthropic,
}

impl SkillSource {
    /// Convert to string for query parameters.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Custom => "custom",
            Self::Anthropic => "anthropic",
        }
    }
}

impl std::fmt::Display for SkillSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Paginated response for listing skills.
///
/// Contains a page of skills along with pagination information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillPage {
    /// List of skills in this page.
    pub data: Vec<Skill>,

    /// Whether there are more results available.
    pub has_more: bool,

    /// ID of the first skill in this page (for backwards pagination).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,

    /// ID of the last skill in this page (for forwards pagination).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,

    /// Next page token for cursor-based pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page: Option<String>,
}

impl SkillPage {
    /// Check if there are more pages available.
    pub fn has_next_page(&self) -> bool {
        self.has_more
    }

    /// Get the cursor for fetching the next page.
    pub fn next_cursor(&self) -> Option<&str> {
        self.next_page.as_deref()
    }
}

/// Paginated response for listing skill versions.
///
/// Contains a page of versions for a specific skill.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillVersionPage {
    /// List of skill versions in this page.
    pub data: Vec<SkillVersion>,

    /// Whether there are more results available.
    pub has_more: bool,

    /// ID of the first version in this page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,

    /// ID of the last version in this page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,

    /// Next page token for cursor-based pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page: Option<String>,
}

impl SkillVersionPage {
    /// Check if there are more pages available.
    pub fn has_next_page(&self) -> bool {
        self.has_more
    }

    /// Get the cursor for fetching the next page.
    pub fn next_cursor(&self) -> Option<&str> {
        self.next_page.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_deserialization() {
        let json = r#"{
            "id": "skill_01ABC",
            "created_at": "2025-01-15T10:30:00Z",
            "display_title": "Weather Lookup",
            "latest_version": "1759178010641129",
            "source": "custom",
            "type": "skill",
            "updated_at": "2025-01-15T10:30:00Z"
        }"#;

        let skill: Skill = serde_json::from_str(json).unwrap();
        assert_eq!(skill.id, "skill_01ABC");
        assert_eq!(skill.display_title, Some("Weather Lookup".to_string()));
        assert_eq!(skill.source, "custom");
        assert_eq!(skill.type_, "skill");
    }

    #[test]
    fn test_skill_version_deserialization() {
        let json = r#"{
            "id": "skill_version_01XYZ",
            "created_at": "2025-01-15T10:30:00Z",
            "description": "Retrieves current weather",
            "directory": "weather_skill",
            "name": "Weather Lookup",
            "skill_id": "skill_01ABC",
            "type": "skill_version",
            "version": "1759178010641129"
        }"#;

        let version: SkillVersion = serde_json::from_str(json).unwrap();
        assert_eq!(version.id, "skill_version_01XYZ");
        assert_eq!(version.skill_id, "skill_01ABC");
        assert_eq!(version.version, "1759178010641129");
        assert_eq!(version.type_, "skill_version");
    }

    #[test]
    fn test_deleted_object_deserialization() {
        let json = r#"{
            "id": "skill_01ABC",
            "deleted": true,
            "type": "skill"
        }"#;

        let deleted: DeletedObject = serde_json::from_str(json).unwrap();
        assert_eq!(deleted.id, "skill_01ABC");
        assert!(deleted.deleted);
        assert_eq!(deleted.type_, "skill");
    }

    #[test]
    fn test_skill_source_serialization() {
        assert_eq!(SkillSource::Custom.as_str(), "custom");
        assert_eq!(SkillSource::Anthropic.as_str(), "anthropic");

        assert_eq!(
            serde_json::to_string(&SkillSource::Custom).unwrap(),
            r#""custom""#
        );
        assert_eq!(
            serde_json::to_string(&SkillSource::Anthropic).unwrap(),
            r#""anthropic""#
        );
    }

    #[test]
    fn test_skill_source_display() {
        assert_eq!(format!("{}", SkillSource::Custom), "custom");
        assert_eq!(format!("{}", SkillSource::Anthropic), "anthropic");
    }

    #[test]
    fn test_skill_optional_fields() {
        let json = r#"{
            "id": "skill_01ABC",
            "created_at": "2025-01-15T10:30:00Z",
            "source": "custom",
            "type": "skill",
            "updated_at": "2025-01-15T10:30:00Z"
        }"#;

        let skill: Skill = serde_json::from_str(json).unwrap();
        assert_eq!(skill.display_title, None);
        assert_eq!(skill.latest_version, None);
    }
}
