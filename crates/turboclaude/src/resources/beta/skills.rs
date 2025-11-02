//! Skills API resource for managing agent capabilities
//!
//! Skills enable reusable agent capabilities with low-latency tool integration.
//! This module provides methods for creating, managing, and versioning skills.

use super::{BETA_SKILLS_API, Resource};
use crate::types::beta::{DeletedObject, Skill, SkillSource, SkillVersion};
use crate::{Client, Error, error::Result};
use std::path::Path;

/// Skills resource for the Beta API.
///
/// Provides methods for creating, listing, retrieving, and deleting skills,
/// as well as managing skill versions.
///
/// # Example
///
/// ```rust,no_run
/// # use turboclaude::Client;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("sk-ant-...");
///
/// // Create a skill with markdown content
/// let skill_content = b"# Weather Lookup Skill\n\nProvides weather information.".to_vec();
/// let skill = client.beta().skills()
///     .create()
///     .file("SKILL.md", skill_content)
///     .display_title("My Skill")
///     .send()
///     .await?;
///
/// // List skills
/// let skills = client.beta().skills()
///     .list()
///     .limit(20)
///     .send()
///     .await?;
///
/// // Retrieve a skill
/// let skill = client.beta().skills()
///     .retrieve(&skill.id)
///     .await?;
///
/// // Delete a skill
/// client.beta().skills()
///     .delete(&skill.id)
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Skills {
    client: Client,
}

impl Skills {
    /// Create a new Skills resource.
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Create a new skill.
    ///
    /// Returns a builder for constructing the skill creation request
    /// with multipart file uploads.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    ///
    /// let skill_content = b"# Weather Lookup\n\nProvides weather data.".to_vec();
    /// let skill = client.beta().skills()
    ///     .create()
    ///     .file("SKILL.md", skill_content)
    ///     .display_title("Weather Lookup")
    ///     .send()
    ///     .await?;
    ///
    /// println!("Created skill: {}", skill.id);
    /// # Ok(())
    /// # }
    /// ```
    pub fn create(&self) -> SkillCreateBuilder {
        SkillCreateBuilder::new(self.client.clone())
    }

    /// List all skills with optional filtering and pagination.
    ///
    /// Returns a builder for constructing the list request with
    /// pagination and filtering options.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    ///
    /// // List first page
    /// let page = client.beta().skills()
    ///     .list()
    ///     .limit(20)
    ///     .send()
    ///     .await?;
    ///
    /// for skill in &page.data {
    ///     println!("{}: {}", skill.id, skill.display_title.as_deref().unwrap_or("Untitled"));
    /// }
    ///
    /// // Get next page if available
    /// if let Some(next_page_token) = page.next_page {
    ///     let next_page = client.beta().skills()
    ///         .list()
    ///         .page(&next_page_token)
    ///         .send()
    ///         .await?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn list(&self) -> SkillListBuilder {
        SkillListBuilder::new(self.client.clone())
    }

    /// Retrieve a specific skill by ID.
    ///
    /// # Arguments
    ///
    /// * `skill_id` - The unique identifier for the skill
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The skill_id is empty
    /// - The skill does not exist
    /// - The API request fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    /// let skill = client.beta().skills().retrieve("skill_01ABC").await?;
    /// println!("Skill: {:?}", skill);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn retrieve(&self, skill_id: &str) -> Result<Skill> {
        if skill_id.is_empty() {
            return Err(Error::InvalidRequest(
                "skill_id cannot be empty".to_string(),
            ));
        }

        let url = format!(
            "{}/v1/skills/{}?beta=true",
            self.client.base_url(),
            skill_id
        );

        let response = self
            .client
            .http_client()
            .get(&url)
            .header("anthropic-beta", BETA_SKILLS_API)
            .header("x-api-key", &self.client.api_key())
            .send()
            .await
            .map_err(|e| Error::HttpClient(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::ApiError {
                status,
                message: text,
                error_type: None,
                request_id: None,
            });
        }

        response
            .json()
            .await
            .map_err(|e| Error::ResponseValidation(e.to_string()))
    }

    /// Delete a skill by ID.
    ///
    /// # Arguments
    ///
    /// * `skill_id` - The unique identifier for the skill to delete
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The skill_id is empty
    /// - The skill does not exist
    /// - The API request fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    /// let deleted = client.beta().skills().delete("skill_01ABC").await?;
    /// assert!(deleted.deleted);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, skill_id: &str) -> Result<DeletedObject> {
        if skill_id.is_empty() {
            return Err(Error::InvalidRequest(
                "skill_id cannot be empty".to_string(),
            ));
        }

        let url = format!(
            "{}/v1/skills/{}?beta=true",
            self.client.base_url(),
            skill_id
        );

        let response = self
            .client
            .http_client()
            .delete(&url)
            .header("anthropic-beta", BETA_SKILLS_API)
            .header("x-api-key", &self.client.api_key())
            .send()
            .await
            .map_err(|e| Error::HttpClient(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::ApiError {
                status,
                message: text,
                error_type: None,
                request_id: None,
            });
        }

        response
            .json()
            .await
            .map_err(|e| Error::ResponseValidation(e.to_string()))
    }

    /// Access the versions sub-resource for a specific skill.
    ///
    /// # Arguments
    ///
    /// * `skill_id` - The unique identifier for the skill
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    ///
    /// // Access versions for a skill
    /// let versions = client.beta().skills()
    ///     .versions("skill_01ABC")
    ///     .list()
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn versions(&self, skill_id: impl Into<String>) -> SkillVersions {
        SkillVersions::new(self.client.clone(), skill_id.into())
    }
}

impl Resource for Skills {
    fn client(&self) -> &Client {
        &self.client
    }
}

/// Builder for creating a new skill with multipart file upload.
///
/// Files must include a SKILL.md at the root of the upload directory.
/// All files should be in the same top-level directory.
pub struct SkillCreateBuilder {
    client: Client,
    files: Vec<(String, Vec<u8>)>,
    display_title: Option<String>,
}

impl SkillCreateBuilder {
    fn new(client: Client) -> Self {
        Self {
            client,
            files: Vec::new(),
            display_title: None,
        }
    }

    /// Add a file to the skill upload.
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path of the file (e.g., "SKILL.md", "lib/utils.py")
    /// * `content` - File content as bytes
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    ///
    /// let skill_md = b"# My Skill\n\nSkill description.".to_vec();
    /// let tool_py = b"def get_weather(): pass".to_vec();
    /// let skill = client.beta().skills()
    ///     .create()
    ///     .file("SKILL.md", skill_md)
    ///     .file("lib/tool.py", tool_py)
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn file(mut self, path: impl Into<String>, content: Vec<u8>) -> Self {
        self.files.push((path.into(), content));
        self
    }

    /// Add a file from a filesystem path.
    ///
    /// # Arguments
    ///
    /// * `name` - Name to use for the file in the skill (e.g., "SKILL.md")
    /// * `path` - Path to the file on the filesystem
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    pub async fn file_from_path(
        mut self,
        name: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Self> {
        let content = tokio::fs::read(path.as_ref()).await.map_err(Error::Io)?;
        self.files.push((name.into(), content));
        Ok(self)
    }

    /// Set the display title for the skill.
    ///
    /// This is a human-readable label that is not included in the
    /// prompt sent to the model.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    ///
    /// let skill = client.beta().skills()
    ///     .create()
    ///     .file("SKILL.md", b"# My Skill\nDescription".to_vec())
    ///     .display_title("Weather Lookup")
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn display_title(mut self, title: impl Into<String>) -> Self {
        self.display_title = Some(title.into());
        self
    }

    /// Execute the skill creation request.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No files have been added
    /// - The multipart upload fails
    /// - The API request fails
    pub async fn send(self) -> Result<Skill> {
        if self.files.is_empty() {
            return Err(Error::InvalidRequest(
                "At least one file is required to create a skill".to_string(),
            ));
        }

        let url = format!("{}/v1/skills?beta=true", self.client.base_url());

        // Build multipart form
        let mut form = reqwest::multipart::Form::new();

        // Add files
        for (path, content) in self.files {
            let filename = path.clone();
            let part = reqwest::multipart::Part::bytes(content)
                .file_name(filename.clone())
                .mime_str("application/octet-stream")
                .map_err(|e| Error::InvalidRequest(format!("Invalid MIME type: {}", e)))?;
            form = form.part("files", part);
        }

        // Add display_title if provided
        if let Some(title) = self.display_title {
            form = form.text("display_title", title);
        }

        // Send request
        let response = self
            .client
            .http_client()
            .post(&url)
            .header("anthropic-beta", BETA_SKILLS_API)
            .header("x-api-key", &self.client.api_key())
            .multipart(form)
            .send()
            .await
            .map_err(|e| Error::HttpClient(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::ApiError {
                status,
                message: text,
                error_type: None,
                request_id: None,
            });
        }

        response
            .json()
            .await
            .map_err(|e| Error::ResponseValidation(e.to_string()))
    }
}

/// Builder for listing skills with pagination and filtering.
pub struct SkillListBuilder {
    client: Client,
    limit: Option<u32>,
    page: Option<String>,
    source: Option<SkillSource>,
}

impl SkillListBuilder {
    fn new(client: Client) -> Self {
        Self {
            client,
            limit: None,
            page: None,
            source: None,
        }
    }

    /// Set the maximum number of skills to return per page.
    ///
    /// Maximum value is 100. Defaults to 20.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    /// let skills = client.beta().skills().list().limit(50).send().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit.min(100));
        self
    }

    /// Set the pagination cursor.
    ///
    /// Pass the value from a previous response's `next_page` field
    /// to get the next page of results.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    ///
    /// let first_page = client.beta().skills().list().send().await?;
    /// if let Some(next_token) = first_page.next_page {
    ///     let next_page = client.beta().skills()
    ///         .list()
    ///         .page(&next_token)
    ///         .send()
    ///         .await?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn page(mut self, token: impl Into<String>) -> Self {
        self.page = Some(token.into());
        self
    }

    /// Filter skills by source.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # use turboclaude::types::beta::SkillSource;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    ///
    /// // Get only user-created skills
    /// let custom_skills = client.beta().skills()
    ///     .list()
    ///     .source(SkillSource::Custom)
    ///     .send()
    ///     .await?;
    ///
    /// // Get only Anthropic-created skills
    /// let anthropic_skills = client.beta().skills()
    ///     .list()
    ///     .source(SkillSource::Anthropic)
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn source(mut self, source: SkillSource) -> Self {
        self.source = Some(source);
        self
    }

    /// Execute the list request.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    pub async fn send(self) -> Result<SkillPage> {
        let mut url = format!("{}/v1/skills?beta=true", self.client.base_url());
        let mut params = Vec::new();

        if let Some(limit) = self.limit {
            params.push(format!("limit={}", limit));
        }
        if let Some(page) = self.page {
            params.push(format!("page={}", page));
        }
        if let Some(source) = self.source {
            params.push(format!("source={}", source.as_str()));
        }

        if !params.is_empty() {
            url.push('&');
            url.push_str(&params.join("&"));
        }

        let response = self
            .client
            .http_client()
            .get(&url)
            .header("anthropic-beta", BETA_SKILLS_API)
            .header("x-api-key", &self.client.api_key())
            .send()
            .await
            .map_err(|e| Error::HttpClient(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::ApiError {
                status,
                message: text,
                error_type: None,
                request_id: None,
            });
        }

        response
            .json()
            .await
            .map_err(|e| Error::ResponseValidation(e.to_string()))
    }
}

/// Paginated response for skill listing.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillPage {
    /// List of skills in this page.
    pub data: Vec<Skill>,

    /// Token for fetching the next page, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page: Option<String>,

    /// Whether there are more results available.
    pub has_more: bool,

    /// ID of the first skill in this page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,

    /// ID of the last skill in this page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
}

/// Versions sub-resource for managing skill versions.
///
/// Access this through `skills().versions(skill_id)`.
pub struct SkillVersions {
    client: Client,
    skill_id: String,
}

impl SkillVersions {
    fn new(client: Client, skill_id: String) -> Self {
        Self { client, skill_id }
    }

    /// Create a new version for this skill.
    ///
    /// Returns a builder for constructing the version creation request.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    ///
    /// let version = client.beta().skills()
    ///     .versions("skill_01ABC")
    ///     .create()
    ///     .file("SKILL.md", b"# Updated skill".to_vec())
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn create(&self) -> VersionCreateBuilder {
        VersionCreateBuilder::new(self.client.clone(), self.skill_id.clone())
    }

    /// List all versions for this skill.
    ///
    /// Returns a builder for constructing the list request.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    ///
    /// let versions = client.beta().skills()
    ///     .versions("skill_01ABC")
    ///     .list()
    ///     .limit(10)
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn list(&self) -> VersionListBuilder {
        VersionListBuilder::new(self.client.clone(), self.skill_id.clone())
    }

    /// Retrieve a specific version.
    ///
    /// # Arguments
    ///
    /// * `version` - Version identifier (Unix epoch timestamp)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The version string is empty
    /// - The version does not exist
    /// - The API request fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    /// let version = client.beta().skills()
    ///     .versions("skill_01ABC")
    ///     .retrieve("1759178010641129")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn retrieve(&self, version: &str) -> Result<SkillVersion> {
        if version.is_empty() {
            return Err(Error::InvalidRequest("version cannot be empty".to_string()));
        }

        let url = format!(
            "{}/v1/skills/{}/versions/{}?beta=true",
            self.client.base_url(),
            self.skill_id,
            version
        );

        let response = self
            .client
            .http_client()
            .get(&url)
            .header("anthropic-beta", BETA_SKILLS_API)
            .header("x-api-key", &self.client.api_key())
            .send()
            .await
            .map_err(|e| Error::HttpClient(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::ApiError {
                status,
                message: text,
                error_type: None,
                request_id: None,
            });
        }

        response
            .json()
            .await
            .map_err(|e| Error::ResponseValidation(e.to_string()))
    }

    /// Delete a specific version.
    ///
    /// # Arguments
    ///
    /// * `version` - Version identifier to delete
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The version string is empty
    /// - The version does not exist
    /// - The API request fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    /// let deleted = client.beta().skills()
    ///     .versions("skill_01ABC")
    ///     .delete("1759178010641129")
    ///     .await?;
    /// assert!(deleted.deleted);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, version: &str) -> Result<DeletedObject> {
        if version.is_empty() {
            return Err(Error::InvalidRequest("version cannot be empty".to_string()));
        }

        let url = format!(
            "{}/v1/skills/{}/versions/{}?beta=true",
            self.client.base_url(),
            self.skill_id,
            version
        );

        let response = self
            .client
            .http_client()
            .delete(&url)
            .header("anthropic-beta", BETA_SKILLS_API)
            .header("x-api-key", &self.client.api_key())
            .send()
            .await
            .map_err(|e| Error::HttpClient(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::ApiError {
                status,
                message: text,
                error_type: None,
                request_id: None,
            });
        }

        response
            .json()
            .await
            .map_err(|e| Error::ResponseValidation(e.to_string()))
    }
}

/// Builder for creating a new skill version.
pub struct VersionCreateBuilder {
    client: Client,
    skill_id: String,
    files: Vec<(String, Vec<u8>)>,
}

impl VersionCreateBuilder {
    fn new(client: Client, skill_id: String) -> Self {
        Self {
            client,
            skill_id,
            files: Vec::new(),
        }
    }

    /// Add a file to the version upload.
    ///
    /// Files must include a SKILL.md at the root.
    pub fn file(mut self, path: impl Into<String>, content: Vec<u8>) -> Self {
        self.files.push((path.into(), content));
        self
    }

    /// Add a file from a filesystem path.
    pub async fn file_from_path(
        mut self,
        name: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Self> {
        let content = tokio::fs::read(path.as_ref()).await.map_err(Error::Io)?;
        self.files.push((name.into(), content));
        Ok(self)
    }

    /// Execute the version creation request.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No files have been added
    /// - The multipart upload fails
    /// - The API request fails
    pub async fn send(self) -> Result<SkillVersion> {
        if self.files.is_empty() {
            return Err(Error::InvalidRequest(
                "At least one file is required to create a version".to_string(),
            ));
        }

        let url = format!(
            "{}/v1/skills/{}/versions?beta=true",
            self.client.base_url(),
            self.skill_id
        );

        // Build multipart form
        let mut form = reqwest::multipart::Form::new();

        // Add files
        for (path, content) in self.files {
            let filename = path.clone();
            let part = reqwest::multipart::Part::bytes(content)
                .file_name(filename.clone())
                .mime_str("application/octet-stream")
                .map_err(|e| Error::InvalidRequest(format!("Invalid MIME type: {}", e)))?;
            form = form.part("files", part);
        }

        // Send request
        let response = self
            .client
            .http_client()
            .post(&url)
            .header("anthropic-beta", BETA_SKILLS_API)
            .header("x-api-key", &self.client.api_key())
            .multipart(form)
            .send()
            .await
            .map_err(|e| Error::HttpClient(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::ApiError {
                status,
                message: text,
                error_type: None,
                request_id: None,
            });
        }

        response
            .json()
            .await
            .map_err(|e| Error::ResponseValidation(e.to_string()))
    }
}

/// Builder for listing skill versions.
pub struct VersionListBuilder {
    client: Client,
    skill_id: String,
    limit: Option<u32>,
    page: Option<String>,
}

impl VersionListBuilder {
    fn new(client: Client, skill_id: String) -> Self {
        Self {
            client,
            skill_id,
            limit: None,
            page: None,
        }
    }

    /// Set the maximum number of versions to return per page.
    ///
    /// Defaults to 20. Ranges from 1 to 1000.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit.clamp(1, 1000));
        self
    }

    /// Set the pagination cursor.
    pub fn page(mut self, token: impl Into<String>) -> Self {
        self.page = Some(token.into());
        self
    }

    /// Execute the list request.
    pub async fn send(self) -> Result<VersionPage> {
        let mut url = format!(
            "{}/v1/skills/{}/versions?beta=true",
            self.client.base_url(),
            self.skill_id
        );
        let mut params = Vec::new();

        if let Some(limit) = self.limit {
            params.push(format!("limit={}", limit));
        }
        if let Some(page) = self.page {
            params.push(format!("page={}", page));
        }

        if !params.is_empty() {
            url.push('&');
            url.push_str(&params.join("&"));
        }

        let response = self
            .client
            .http_client()
            .get(&url)
            .header("anthropic-beta", BETA_SKILLS_API)
            .header("x-api-key", &self.client.api_key())
            .send()
            .await
            .map_err(|e| Error::HttpClient(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::ApiError {
                status,
                message: text,
                error_type: None,
                request_id: None,
            });
        }

        response
            .json()
            .await
            .map_err(|e| Error::ResponseValidation(e.to_string()))
    }
}

/// Paginated response for version listing.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VersionPage {
    /// List of versions in this page.
    pub data: Vec<SkillVersion>,

    /// Token for fetching the next page, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page: Option<String>,

    /// Whether there are more results available.
    pub has_more: bool,

    /// ID of the first version in this page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,

    /// ID of the last version in this page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skills_resource_creation() {
        let client = Client::new("test-key");
        let skills = Skills::new(client);

        // Test that we can create builders
        let _create_builder = skills.create();
        let _list_builder = skills.list();
        let _versions = skills.versions("skill_123");
    }

    #[test]
    fn test_skill_create_builder() {
        let client = Client::new("test-key");
        let builder = Skills::new(client)
            .create()
            .file("SKILL.md", b"# My Skill".to_vec())
            .display_title("Test Skill");

        // Builder should have been constructed successfully
        assert_eq!(builder.files.len(), 1);
        assert_eq!(builder.display_title, Some("Test Skill".to_string()));
    }

    #[test]
    fn test_skill_list_builder() {
        let client = Client::new("test-key");
        let builder = Skills::new(client)
            .list()
            .limit(50)
            .page("token123")
            .source(SkillSource::Custom);

        assert_eq!(builder.limit, Some(50));
        assert_eq!(builder.page, Some("token123".to_string()));
        assert_eq!(builder.source, Some(SkillSource::Custom));
    }

    #[test]
    fn test_skill_list_builder_limit_clamping() {
        let client = Client::new("test-key");
        let builder = Skills::new(client).list().limit(200);

        // Should be clamped to 100
        assert_eq!(builder.limit, Some(100));
    }

    #[test]
    fn test_version_list_builder_limit_clamping() {
        let client = Client::new("test-key");
        let builder = SkillVersions::new(client, "skill_123".to_string())
            .list()
            .limit(2000);

        // Should be clamped to 1000
        assert_eq!(builder.limit, Some(1000));
    }
}
