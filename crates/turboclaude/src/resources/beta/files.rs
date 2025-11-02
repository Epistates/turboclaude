//! Files API resource for uploading and managing files
//!
//! This module provides access to the Files API which allows uploading files
//! for document analysis, image understanding, and other file-based features.

use super::{BETA_FILES_API, Resource};
use crate::types::beta::{FileListParams, FileMetadata, FilePage};
use crate::{Client, error::Result};
use bytes::Bytes;
use std::path::Path;

/// Files resource for the Beta API
///
/// Provides methods for uploading, downloading, listing, and managing files.
///
/// # Example
///
/// ```rust,no_run
/// # use turboclaude::Client;
/// # use turboclaude::types::beta::FileListParams;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("sk-ant-...");
///
/// // Upload a file
/// let file = client.beta().files().upload("data.csv").await?;
/// println!("Uploaded: {} ({})", file.filename, file.id);
///
/// // List files
/// let page = client.beta().files().list(FileListParams::new().limit(20)).await?;
/// for file in page.data {
///     println!("{}: {} bytes", file.filename, file.size_bytes);
/// }
///
/// // Download file
/// let content = client.beta().files().download(&file.id).await?;
///
/// // Delete file
/// client.beta().files().delete(&file.id).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Files {
    client: Client,
}

impl Files {
    /// Create a new Files resource
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Upload a file to the Anthropic API
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to upload
    ///
    /// # Returns
    ///
    /// File metadata including the file ID for future operations
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    /// let file = client.beta().files().upload("data.csv").await?;
    /// println!("Uploaded file ID: {}", file.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upload(&self, path: impl AsRef<Path>) -> Result<FileMetadata> {
        let file_path = path.as_ref();

        // Build URL
        let url = format!("{}/v1/files", self.client.base_url());

        // Create multipart form
        let form = reqwest::multipart::Form::new()
            .file("file", file_path)
            .await
            .map_err(|e| crate::error::Error::Io(std::io::Error::other(e)))?;

        // Use reqwest client directly for multipart
        let response = self
            .client
            .http_client()
            .post(&url)
            .header("anthropic-beta", BETA_FILES_API)
            .header("x-api-key", &self.client.api_key())
            .multipart(form)
            .send()
            .await
            .map_err(|e| crate::error::Error::HttpClient(e.to_string()))?;

        // Check for errors
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(crate::error::Error::ApiError {
                status,
                message: text,
                error_type: None,
                request_id: None,
            });
        }

        response
            .json()
            .await
            .map_err(|e| crate::error::Error::ResponseValidation(e.to_string()))
    }

    /// Download file content as bytes
    ///
    /// # Arguments
    ///
    /// * `file_id` - ID of the file to download
    ///
    /// # Returns
    ///
    /// Raw file content as bytes
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    /// let content = client.beta().files().download("file_abc123").await?;
    /// std::fs::write("downloaded.csv", content)?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn download(&self, file_id: &str) -> Result<Bytes> {
        let url = format!("{}/v1/files/{}/content", self.client.base_url(), file_id);

        let response = self
            .client
            .http_client()
            .get(&url)
            .header("anthropic-beta", BETA_FILES_API)
            .header("x-api-key", &self.client.api_key())
            .header("Accept", "application/binary")
            .send()
            .await
            .map_err(|e| crate::error::Error::HttpClient(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(crate::error::Error::ApiError {
                status,
                message: text,
                error_type: None,
                request_id: None,
            });
        }

        response
            .bytes()
            .await
            .map_err(|e| crate::error::Error::HttpClient(e.to_string()))
    }

    /// List files with optional pagination
    ///
    /// # Arguments
    ///
    /// * `params` - List parameters including limit and cursors
    ///
    /// # Returns
    ///
    /// Paginated list of file metadata
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # use turboclaude::types::beta::FileListParams;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    ///
    /// // First page
    /// let page = client.beta().files().list(FileListParams::new().limit(20)).await?;
    ///
    /// // Next page
    /// if let Some(cursor) = page.next_cursor() {
    ///     let next_page = client.beta().files()
    ///         .list(FileListParams::new().after(cursor))
    ///         .await?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self, params: FileListParams) -> Result<FilePage> {
        let url = format!("{}/v1/files", self.client.base_url());

        let response = self
            .client
            .http_client()
            .get(&url)
            .header("anthropic-beta", BETA_FILES_API)
            .header("x-api-key", &self.client.api_key())
            .query(&params)
            .send()
            .await
            .map_err(|e| crate::error::Error::HttpClient(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(crate::error::Error::ApiError {
                status,
                message: text,
                error_type: None,
                request_id: None,
            });
        }

        response
            .json()
            .await
            .map_err(|e| crate::error::Error::ResponseValidation(e.to_string()))
    }

    /// Get metadata for a specific file
    ///
    /// # Arguments
    ///
    /// * `file_id` - ID of the file
    ///
    /// # Returns
    ///
    /// File metadata
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    /// let metadata = client.beta().files().get("file_abc123").await?;
    /// println!("File: {} ({} bytes)", metadata.filename, metadata.size_bytes);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, file_id: &str) -> Result<FileMetadata> {
        self.client
            .beta_request(
                http::Method::GET,
                &format!("/v1/files/{}", file_id),
                BETA_FILES_API,
            )?
            .send()
            .await?
            .parse_result()
    }

    /// Delete a file
    ///
    /// # Arguments
    ///
    /// * `file_id` - ID of the file to delete
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    /// client.beta().files().delete("file_abc123").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, file_id: &str) -> Result<()> {
        self.client
            .beta_request(
                http::Method::DELETE,
                &format!("/v1/files/{}", file_id),
                BETA_FILES_API,
            )?
            .send()
            .await?
            .parse_result()
            .map(|_: serde_json::Value| ()) // Discard response body
    }
}

impl Resource for Files {
    fn client(&self) -> &Client {
        &self.client
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beta_files_resource_creation() {
        let client = Client::new("test-api-key");
        let files = client.beta().files();

        // Verify resource is created (client is cloned, so we can't use ptr::eq)
        // Just verify the resource has a client reference
        let _ = files.client();
    }

    #[test]
    fn test_file_list_params_builder() {
        let params = FileListParams::new().limit(50).after("file_abc123");

        assert_eq!(params.limit, Some(50));
        assert_eq!(params.after, Some("file_abc123".to_string()));
        assert!(params.before.is_none());

        // Test serialization
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["limit"], 50);
        assert_eq!(json["after"], "file_abc123");
    }

    #[test]
    fn test_file_metadata_structure() {
        use crate::types::beta::FileMetadata;
        use chrono::Utc;

        let metadata = FileMetadata {
            id: "file_123".to_string(),
            created_at: Utc::now(),
            filename: "test.csv".to_string(),
            mime_type: "text/csv".to_string(),
            size_bytes: 1024,
            object_type: "file".to_string(),
            downloadable: Some(true),
        };

        assert_eq!(metadata.id, "file_123");
        assert_eq!(metadata.filename, "test.csv");
        assert_eq!(metadata.mime_type, "text/csv");
        assert_eq!(metadata.size_bytes, 1024);
        assert_eq!(metadata.downloadable, Some(true));
    }
}
