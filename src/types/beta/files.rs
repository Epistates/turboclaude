//! Files API types for beta features
//!
//! Types for uploading, downloading, and managing files with the Anthropic API.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// File metadata returned by the Files API
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileMetadata {
    /// Unique file identifier
    pub id: String,

    /// When the file was created
    pub created_at: DateTime<Utc>,

    /// Original filename
    pub filename: String,

    /// MIME type of the file
    pub mime_type: String,

    /// Size in bytes
    pub size_bytes: u64,

    /// Object type (always "file")
    #[serde(rename = "type")]
    pub object_type: String,

    /// Whether the file is downloadable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloadable: Option<bool>,
}

/// Purpose of file upload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilePurpose {
    /// User-uploaded file
    UserUpload,
    /// Other purposes (extensible)
    #[serde(other)]
    Other,
}

/// Parameters for listing files
#[derive(Debug, Clone, Serialize, Default)]
pub struct FileListParams {
    /// Maximum number of files to return (default: 20, max: 100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Cursor for pagination (after this ID)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,

    /// Cursor for pagination (before this ID)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
}

impl FileListParams {
    /// Create new list params with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the limit
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the after cursor
    pub fn after(mut self, cursor: impl Into<String>) -> Self {
        self.after = Some(cursor.into());
        self
    }

    /// Set the before cursor
    pub fn before(mut self, cursor: impl Into<String>) -> Self {
        self.before = Some(cursor.into());
        self
    }
}

/// Paginated list of files
#[derive(Debug, Clone, Deserialize)]
pub struct FilePage {
    /// List of file metadata
    pub data: Vec<FileMetadata>,

    /// Whether there are more results
    pub has_more: bool,

    /// ID of the first item in this page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,

    /// ID of the last item in this page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
}

impl FilePage {
    /// Check if there are more pages
    pub fn has_next_page(&self) -> bool {
        self.has_more
    }

    /// Get cursor for next page
    pub fn next_cursor(&self) -> Option<&str> {
        self.last_id.as_deref()
    }

    /// Get cursor for previous page
    pub fn prev_cursor(&self) -> Option<&str> {
        self.first_id.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_list_params_builder() {
        let params = FileListParams::new()
            .limit(50)
            .after("file_abc123");

        assert_eq!(params.limit, Some(50));
        assert_eq!(params.after, Some("file_abc123".to_string()));
        assert_eq!(params.before, None);
    }

    #[test]
    fn test_file_list_params_serialization() {
        let params = FileListParams::new().limit(25).after("cursor123");

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"limit\":25"));
        assert!(json.contains("\"after\":\"cursor123\""));
    }

    #[test]
    fn test_file_page_cursors() {
        let page = FilePage {
            data: vec![],
            has_more: true,
            first_id: Some("first123".to_string()),
            last_id: Some("last456".to_string()),
        };

        assert!(page.has_next_page());
        assert_eq!(page.next_cursor(), Some("last456"));
        assert_eq!(page.prev_cursor(), Some("first123"));
    }

    #[test]
    fn test_file_metadata_deserialization() {
        let json = r#"{
            "id": "file_abc123",
            "created_at": "2025-01-15T10:30:00Z",
            "filename": "data.csv",
            "mime_type": "text/csv",
            "size_bytes": 1024,
            "type": "file",
            "downloadable": true
        }"#;

        let metadata: FileMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.id, "file_abc123");
        assert_eq!(metadata.filename, "data.csv");
        assert_eq!(metadata.mime_type, "text/csv");
        assert_eq!(metadata.size_bytes, 1024);
        assert_eq!(metadata.object_type, "file");
        assert_eq!(metadata.downloadable, Some(true));
    }

    #[test]
    fn test_file_purpose_serialization() {
        let purpose = FilePurpose::UserUpload;
        let json = serde_json::to_string(&purpose).unwrap();
        assert_eq!(json, "\"user_upload\"");
    }
}
