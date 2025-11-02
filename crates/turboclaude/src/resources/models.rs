//! Models API endpoint

use super::Resource;
use crate::{client::Client, error::Result, http::RawResponse, types::Model};

/// Models API resource.
///
/// This endpoint allows you to list available models.
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
    pub async fn list(&self) -> Result<Vec<Model>> {
        #[derive(serde::Deserialize)]
        struct ModelList {
            data: Vec<Model>,
        }

        let list: ModelList = self
            .client
            .request(http::Method::GET, "/v1/models")?
            .send()
            .await?
            .parse_result()?;

        Ok(list.data)
    }

    /// Get information about a specific model.
    pub async fn get(&self, model_id: &str) -> Result<Model> {
        let response = self
            .client
            .request(http::Method::GET, &format!("/v1/models/{}", model_id))?
            .send()
            .await?;

        response.parse_result()
    }

    /// Enable raw response mode for the next request.
    ///
    /// Returns a wrapper that provides access to response headers,
    /// status codes, and other HTTP metadata along with the parsed body.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use turboclaude::Client;
    /// # async fn example(client: Client) -> Result<(), Box<dyn std::error::Error>> {
    /// // Get raw response with headers
    /// let raw = client.models()
    ///     .with_raw_response()
    ///     .get("claude-3-5-sonnet-20241022")
    ///     .await?;
    ///
    /// // Access headers
    /// if let Some(request_id) = raw.request_id() {
    ///     println!("Request ID: {}", request_id);
    /// }
    ///
    /// // Access parsed response
    /// println!("Model: {}", raw.parsed().id);
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_raw_response(&self) -> ModelsRaw {
        ModelsRaw {
            client: self.client.clone(),
        }
    }
}

impl Resource for Models {
    fn client(&self) -> &Client {
        &self.client
    }
}

/// Models resource in raw response mode.
///
/// This wrapper provides the same methods as `Models`, but returns
/// `RawResponse<T>` instead of `T`, giving access to HTTP headers and metadata.
#[derive(Clone)]
pub struct ModelsRaw {
    client: Client,
}

impl ModelsRaw {
    /// List all available models and return raw response with headers.
    pub async fn list(&self) -> Result<RawResponse<Vec<Model>>> {
        #[derive(serde::Deserialize)]
        struct ModelList {
            data: Vec<Model>,
        }

        let raw: RawResponse<ModelList> = self
            .client
            .request(http::Method::GET, "/v1/models")?
            .send()
            .await?
            .into_parsed_raw()?;

        // Extract the data field while preserving raw response metadata
        let (status, headers) = (raw.status(), raw.headers().clone());
        let data = raw.into_parsed().data;

        Ok(RawResponse::new(data, status, headers))
    }

    /// Get information about a specific model and return raw response with headers.
    pub async fn get(&self, model_id: &str) -> Result<RawResponse<Model>> {
        let response = self
            .client
            .request(http::Method::GET, &format!("/v1/models/{}", model_id))?
            .send()
            .await?;

        response.into_parsed_raw()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_models_resource_creation() {
        let client = Client::new("test-api-key");
        let models = client.models();

        // Verify resource is created (client is cloned, so we can't use ptr::eq)
        // Just verify the resource has a client reference
        let _ = models.client();
    }

    #[test]
    fn test_models_with_raw_response() {
        let client = Client::new("test-api-key");
        let _models_raw = client.models().with_raw_response();

        // Verify we can create raw response wrapper
        // (actual HTTP calls tested in integration tests)
    }
}
