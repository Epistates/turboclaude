//! Files API resource for uploading and managing files
//!
//! This module provides access to the Files API which allows uploading files
//! for document analysis, image understanding, and other file-based features.

use std::path::Path;
use bytes::Bytes;
use crate::{Client, error::Result, http};
use crate::types::beta::{FileMetadata, FileListParams, FilePage};
use super::BETA_FILES_API;

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

        // Create multipart form with file
        let form = reqwest::multipart::Form::new()
            .file("file", file_path)
            .await
            .map_err(|e| crate::error::Error::Io(std::io::Error::other(e)))?;

        // Use client's multipart_request builder to respect all client configuration
        let response = self.client
            .multipart_request(http::Method::POST, "/v1/files")?
            .header("anthropic-beta", BETA_FILES_API)
            .multipart(form)
            .send()
            .await
            .map_err(|e| crate::error::Error::HttpClient(e.to_string()))?;

        // Use SDK's standard error handling for consistent error conversion
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            let error = crate::error::Error::from_response(status, &text, response.headers());
            return Err(error);
        }

        response.json().await
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
        // Use client's multipart_request to get consistent configuration (timeout, headers, etc.)
        let response = self.client
            .multipart_request(http::Method::GET, &format!("/v1/files/{}/content", file_id))?
            .header("anthropic-beta", BETA_FILES_API)
            .header("Accept", "application/binary")
            .send()
            .await
            .map_err(|e| crate::error::Error::HttpClient(e.to_string()))?;

        // Use SDK's standard error handling for consistent error conversion
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            let error = crate::error::Error::from_response(status, &text, response.headers());
            return Err(error);
        }

        response.bytes().await
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
        // Use client's multipart_request to get consistent configuration
        let response = self.client
            .multipart_request(http::Method::GET, "/v1/files")?
            .header("anthropic-beta", BETA_FILES_API)
            .query(&params)
            .send()
            .await
            .map_err(|e| crate::error::Error::HttpClient(e.to_string()))?;

        // Use SDK's standard error handling for consistent error conversion
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            let error = crate::error::Error::from_response(status, &text, response.headers());
            return Err(error);
        }

        response.json().await
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
                BETA_FILES_API
            )
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
                BETA_FILES_API
            )
            .send()
            .await?
            .parse_result()
            .map(|_: serde_json::Value| ())  // Discard response body
    }
}
