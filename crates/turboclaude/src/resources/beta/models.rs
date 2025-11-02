//! Models API resource for the Beta API
//!
//! List and retrieve model information including model IDs, display names, and metadata.

use super::Resource;
use crate::types::beta::{Model, ModelPage};
use crate::{Client, Error, error::Result};

/// Beta version for Models API
pub const BETA_MODELS_API: &str = "models-2024-04-01";

/// Models resource for the Beta API.
///
/// Provides methods for listing available models and retrieving specific model information.
///
/// # Example
///
/// ```rust,no_run
/// # use turboclaude::Client;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("sk-ant-...");
///
/// // List all models
/// let models = client.beta().models()
///     .list()
///     .limit(10)
///     .send()
///     .await?;
///
/// for model in &models.data {
///     println!("{}: {}", model.id, model.display_name);
/// }
///
/// // Retrieve a specific model
/// let model = client.beta().models()
///     .retrieve("claude-3-5-sonnet-20241022")
///     .await?;
///
/// println!("Model: {} ({})", model.display_name, model.id);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Models {
    client: Client,
}

impl Models {
    /// Create a new Models resource.
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// List all available models.
    ///
    /// Returns a builder for constructing the list request with
    /// pagination options.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    ///
    /// // List first page
    /// let page = client.beta().models()
    ///     .list()
    ///     .limit(20)
    ///     .send()
    ///     .await?;
    ///
    /// for model in &page.data {
    ///     println!("{}: {}", model.id, model.display_name);
    /// }
    ///
    /// // Get next page if available
    /// if page.has_more {
    ///     if let Some(last_id) = page.last_id {
    ///         let next_page = client.beta().models()
    ///             .list()
    ///             .after(&last_id)
    ///             .send()
    ///             .await?;
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn list(&self) -> ModelsListBuilder {
        ModelsListBuilder::new(self.client.clone())
    }

    /// Retrieve a specific model by ID or alias.
    ///
    /// # Arguments
    ///
    /// * `model_id` - The model identifier or alias (e.g., "claude-3-5-sonnet-20241022" or "claude-3-5-sonnet")
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The model_id is empty
    /// - The model does not exist
    /// - The API request fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    ///
    /// // Retrieve by full ID
    /// let model = client.beta().models()
    ///     .retrieve("claude-3-5-sonnet-20241022")
    ///     .await?;
    ///
    /// // Or by alias
    /// let model = client.beta().models()
    ///     .retrieve("claude-3-5-sonnet")
    ///     .await?;
    ///
    /// println!("Model: {} ({})", model.display_name, model.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn retrieve(&self, model_id: impl AsRef<str>) -> Result<Model> {
        let model_id = model_id.as_ref();
        if model_id.is_empty() {
            return Err(Error::InvalidRequest(
                "model_id cannot be empty".to_string(),
            ));
        }

        let base_url = self.client.base_url().trim_end_matches('/');
        let url = format!("{}/v1/models/{}?beta=true", base_url, model_id);

        let response = self
            .client
            .http_client()
            .get(&url)
            .header("anthropic-beta", BETA_MODELS_API)
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

impl Resource for Models {
    fn client(&self) -> &Client {
        &self.client
    }
}

/// Builder for listing models with pagination.
pub struct ModelsListBuilder {
    client: Client,
    limit: Option<u32>,
    before_id: Option<String>,
    after_id: Option<String>,
}

impl std::fmt::Debug for ModelsListBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelsListBuilder")
            .field("limit", &self.limit)
            .field("before_id", &self.before_id)
            .field("after_id", &self.after_id)
            .finish()
    }
}

impl ModelsListBuilder {
    fn new(client: Client) -> Self {
        Self {
            client,
            limit: None,
            before_id: None,
            after_id: None,
        }
    }

    /// Set the maximum number of models to return per page.
    ///
    /// Defaults to 20. Ranges from 1 to 1000.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    /// let models = client.beta().models().list().limit(50).send().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit.clamp(1, 1000));
        self
    }

    /// Set cursor to get models before this ID.
    ///
    /// When provided, returns the page of results immediately before this object.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    ///
    /// // Get previous page
    /// let page = client.beta().models()
    ///     .list()
    ///     .before("claude-3-opus-20240229")
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn before(mut self, id: impl Into<String>) -> Self {
        self.before_id = Some(id.into());
        self
    }

    /// Set cursor to get models after this ID.
    ///
    /// When provided, returns the page of results immediately after this object.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("sk-ant-...");
    ///
    /// // Get next page
    /// let page = client.beta().models()
    ///     .list()
    ///     .after("claude-3-5-sonnet-20241022")
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn after(mut self, id: impl Into<String>) -> Self {
        self.after_id = Some(id.into());
        self
    }

    /// Execute the list request.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    pub async fn send(self) -> Result<ModelPage> {
        let base_url = self.client.base_url().trim_end_matches('/');
        let mut url = format!("{}/v1/models?beta=true", base_url);
        let mut params = Vec::new();

        if let Some(limit) = self.limit {
            params.push(format!("limit={}", limit));
        }
        if let Some(before_id) = self.before_id {
            params.push(format!("before_id={}", before_id));
        }
        if let Some(after_id) = self.after_id {
            params.push(format!("after_id={}", after_id));
        }

        if !params.is_empty() {
            url.push('&');
            url.push_str(&params.join("&"));
        }

        let response = self
            .client
            .http_client()
            .get(&url)
            .header("anthropic-beta", BETA_MODELS_API)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_models_resource_creation() {
        let client = Client::new("test-key");
        let models = Models::new(client);

        // Test that we can create builders
        let _list_builder = models.list();
    }

    #[test]
    fn test_models_list_builder() {
        let client = Client::new("test-key");
        let builder = Models::new(client)
            .list()
            .limit(50)
            .before("model_before")
            .after("model_after");

        assert_eq!(builder.limit, Some(50));
        assert_eq!(builder.before_id, Some("model_before".to_string()));
        assert_eq!(builder.after_id, Some("model_after".to_string()));
    }

    #[test]
    fn test_models_list_builder_limit_clamping() {
        let client = Client::new("test-key");

        // Test lower bound
        let builder_low = Models::new(client.clone()).list().limit(0);
        assert_eq!(builder_low.limit, Some(1));

        // Test upper bound
        let builder_high = Models::new(client).list().limit(2000);
        assert_eq!(builder_high.limit, Some(1000));
    }

    #[test]
    fn test_models_list_builder_default() {
        let client = Client::new("test-key");
        let builder = Models::new(client).list();

        assert_eq!(builder.limit, None);
        assert_eq!(builder.before_id, None);
        assert_eq!(builder.after_id, None);
    }
}
