//! Google Vertex AI client implementation

use std::sync::{Arc, OnceLock};
use std::time::Duration;

use google_cloud_auth::token::TokenSource;
use reqwest::Client as HttpClient;
use secrecy::{ExposeSecret, SecretString};
use serde_json::Value as JsonValue;
use url::Url;

use crate::{
    client::Client,
    error::{Error, Result},
    http::{HttpProvider, RequestBuilder},
    resources::{Messages, Models},
    streaming::StreamEvent,
    types::{Message, MessageRequest},
    DEFAULT_API_VERSION,
};

use super::{error::VertexError, http::VertexHttpProvider, VERTEX_API_VERSION};

/// Google Vertex AI client for accessing Claude models
///
/// This client provides access to Claude models through Google Cloud Vertex AI.
/// It uses Google Cloud authentication (ADC, service accounts, etc.) and makes
/// direct REST API calls to Vertex AI endpoints.
///
/// # Examples
///
/// ```rust,no_run
/// use turboclaude::providers::vertex::VertexClient;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = VertexClient::builder()
///     .project_id("my-gcp-project")
///     .region("us-central1")
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct VertexClient {
    inner: Arc<VertexClientInner>,
}

struct VertexClientInner {
    /// HTTP client for making requests
    http_client: HttpClient,
    /// HTTP provider for making requests
    http_provider: Arc<dyn HttpProvider>,
    /// Google Cloud project ID
    project_id: String,
    /// Google Cloud region (e.g., "us-central1")
    region: String,
    /// Token source for authentication (optional, for ADC)
    token_source: Option<Box<dyn TokenSource>>,
    /// Explicit access token (alternative to token source)
    access_token: Option<SecretString>,
    /// Base URL for Vertex AI API
    base_url: Url,
    /// Default timeout for requests
    timeout: Duration,
    /// Maximum retries
    max_retries: u32,
    /// API version for anthropic_version header
    api_version: String,
    /// Lazy-initialized client using the HTTP provider
    client: OnceLock<Client>,
}

impl VertexClient {
    /// Create a new builder for configuring the Vertex client
    pub fn builder() -> VertexClientBuilder {
        VertexClientBuilder::default()
    }

    /// Get the Messages resource for this client
    pub fn messages(&self) -> &Messages {
        let client = self
            .inner
            .client
            .get_or_init(|| Client::from_provider(self.inner.http_provider.clone()));
        client.messages()
    }

    /// Get the Models resource for this client
    ///
    /// Note: Google Vertex AI does not provide a Models API endpoint.
    /// This returns an empty Models resource.
    pub fn models(&self) -> &Models {
        let client = self
            .inner
            .client
            .get_or_init(|| Client::from_provider(self.inner.http_provider.clone()));
        client.models()
    }

    /// Get the Google Cloud project ID being used
    pub fn project_id(&self) -> &str {
        &self.inner.project_id
    }

    /// Get the Google Cloud region being used
    pub fn region(&self) -> &str {
        &self.inner.region
    }

    /// Get the base URL for Vertex AI API
    pub fn base_url(&self) -> &Url {
        &self.inner.base_url
    }

    /// Get a valid access token for authentication
    ///
    /// This will either return the explicit access token or fetch a fresh token
    /// from the token source (ADC).
    async fn get_access_token(&self) -> Result<String> {
        if let Some(token) = &self.inner.access_token {
            return Ok(token.expose_secret().to_string());
        }

        if let Some(token_source) = &self.inner.token_source {
            let token = token_source
                .token()
                .await
                .map_err(|e| VertexError::Authentication(e.to_string()))?;

            Ok(token.access_token)
        } else {
            Err(VertexError::Authentication(
                "No access token or token source available".to_string(),
            )
            .into())
        }
    }

    /// Construct the URL for a Messages API request
    ///
    /// Format: https://{region}-aiplatform.googleapis.com/v1/projects/{project}/locations/{region}/publishers/anthropic/models/{model}:rawPredict
    fn messages_url(&self, model: &str, streaming: bool) -> String {
        let action = if streaming {
            "streamRawPredict"
        } else {
            "rawPredict"
        };

        format!(
            "{}/projects/{}/locations/{}/publishers/anthropic/models/{}:{}",
            self.inner.base_url, self.inner.project_id, self.inner.region, model, action
        )
    }

    /// Construct the URL for token counting API
    fn token_count_url(&self) -> String {
        format!(
            "{}/projects/{}/locations/{}/publishers/anthropic/models/count-tokens:rawPredict",
            self.inner.base_url, self.inner.project_id, self.inner.region
        )
    }

    /// Send a non-streaming Messages API request
    pub async fn send_message(&self, request: &MessageRequest) -> Result<Message> {
        let url = self.messages_url(&request.model, false);
        let token = self.get_access_token().await?;

        // Prepare request body
        let mut body = serde_json::to_value(request)
            .map_err(|e| Error::Other(format!("Failed to serialize request: {}", e)))?;

        // Add anthropic_version to body
        if let Some(obj) = body.as_object_mut() {
            obj.insert(
                "anthropic_version".to_string(),
                serde_json::Value::String(self.inner.api_version.clone()),
            );
        }

        // Send request
        let response = self
            .inner
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .timeout(self.inner.timeout)
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        // Check status
        let status = response.status();
        if !status.is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(VertexError::Api(format!("Status {}: {}", status, error_body)).into());
        }

        // Parse response
        let message = response
            .json::<Message>()
            .await
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to parse response: {}", e)))?;

        Ok(message)
    }

    /// Send a streaming Messages API request
    pub async fn send_message_stream(
        &self,
        request: &MessageRequest,
    ) -> Result<impl futures::Stream<Item = Result<StreamEvent>>> {
        let url = self.messages_url(&request.model, true);
        let token = self.get_access_token().await?;

        // Prepare request body
        let mut body = serde_json::to_value(request)
            .map_err(|e| Error::Other(format!("Failed to serialize request: {}", e)))?;

        // Add anthropic_version and stream flag to body
        if let Some(obj) = body.as_object_mut() {
            obj.insert(
                "anthropic_version".to_string(),
                serde_json::Value::String(self.inner.api_version.clone()),
            );
            obj.insert("stream".to_string(), serde_json::Value::Bool(true));
        }

        // Send request
        let response = self
            .inner
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .timeout(self.inner.timeout)
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        // Check status
        let status = response.status();
        if !status.is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(VertexError::Api(format!("Status {}: {}", status, error_body)).into());
        }

        // Parse SSE stream
        // Note: For v0.1.0, use VertexHttpProvider instead which has full streaming support
        Err(VertexError::Service(
            "Streaming is not supported in this legacy VertexClient implementation. \
             Please use VertexHttpProvider instead, which supports full streaming. \
             See: https://docs.anthropic.com/en/api/claude-on-vertex-ai"
                .to_string(),
        ).into())
    }
}

/// Builder for configuring a Vertex client
#[derive(Default)]
pub struct VertexClientBuilder {
    project_id: Option<String>,
    region: Option<String>,
    access_token: Option<String>,
    base_url: Option<String>,
    timeout: Option<Duration>,
    max_retries: Option<u32>,
    api_version: Option<String>,
}

impl VertexClientBuilder {
    /// Set the Google Cloud project ID
    ///
    /// Can also be set via ANTHROPIC_VERTEX_PROJECT_ID or GOOGLE_CLOUD_PROJECT environment variables.
    pub fn project_id(mut self, project_id: impl Into<String>) -> Self {
        self.project_id = Some(project_id.into());
        self
    }

    /// Set the Google Cloud region
    ///
    /// Can also be set via CLOUD_ML_REGION environment variable.
    /// Common regions: us-central1, us-east4, europe-west1, asia-southeast1
    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    /// Set an explicit access token (instead of using ADC)
    pub fn access_token(mut self, token: impl Into<String>) -> Self {
        self.access_token = Some(token.into());
        self
    }

    /// Set a custom Vertex AI endpoint URL
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Set request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set maximum number of retries
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = Some(retries);
        self
    }

    /// Set API version
    pub fn api_version(mut self, version: impl Into<String>) -> Self {
        self.api_version = Some(version.into());
        self
    }

    /// Build the Vertex client
    pub async fn build(self) -> Result<VertexClient> {
        // Get region (required)
        let region = self
            .region
            .or_else(|| std::env::var("CLOUD_ML_REGION").ok())
            .ok_or(VertexError::MissingRegion)?;

        // Get project ID
        let project_id = self
            .project_id
            .or_else(|| std::env::var("ANTHROPIC_VERTEX_PROJECT_ID").ok())
            .or_else(|| std::env::var("GOOGLE_CLOUD_PROJECT").ok())
            .or_else(|| std::env::var("GCP_PROJECT").ok())
            .ok_or(VertexError::MissingProjectId)?;

        // Construct base URL
        let base_url = if let Some(url) = self.base_url {
            Url::parse(&url).map_err(|e| Error::Other(format!("Invalid base URL: {}", e)))?
        } else {
            let url = std::env::var("ANTHROPIC_VERTEX_BASE_URL").unwrap_or_else(|_| {
                if region == "global" {
                    "https://aiplatform.googleapis.com/v1".to_string()
                } else {
                    format!("https://{}-aiplatform.googleapis.com/v1", region)
                }
            });

            Url::parse(&url).map_err(|e| Error::Other(format!("Invalid base URL: {}", e)))?
        };

        // Set up authentication
        let (token_source, access_token) = if let Some(token) = self.access_token {
            (None, Some(SecretString::new(token)))
        } else {
            // Use Application Default Credentials (ADC)
            let token_source_manager =
                google_cloud_auth::token::DefaultTokenSourceProvider::new(
                    google_cloud_auth::project::Config::default().with_project_id(Some(project_id.clone())),
                )
                .await
                .map_err(|e| VertexError::Authentication(e.to_string()))?;

            let token_source = token_source_manager
                .token_source()
                .await
                .map_err(|e| VertexError::Authentication(e.to_string()))?;

            (Some(token_source), None)
        };

        // Build HTTP client
        let timeout = self.timeout.unwrap_or(Duration::from_secs(600));
        let http_client = HttpClient::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| Error::Other(format!("Failed to build HTTP client: {}", e)))?;

        // Create VertexHttpProvider with the same configuration
        let mut http_provider_builder = VertexHttpProvider::builder()
            .project_id(&project_id)
            .region(&region)
            .timeout(timeout);

        // Apply access token if provided
        if let Some(token) = self.access_token.as_ref() {
            http_provider_builder = http_provider_builder.access_token(token);
        }

        let http_provider: Arc<dyn HttpProvider> = Arc::new(http_provider_builder.build().await?);

        Ok(VertexClient {
            inner: Arc::new(VertexClientInner {
                http_client,
                http_provider,
                project_id,
                region,
                token_source,
                access_token,
                base_url,
                timeout,
                max_retries: self.max_retries.unwrap_or(2),
                api_version: self
                    .api_version
                    .unwrap_or_else(|| VERTEX_API_VERSION.to_string()),
                client: OnceLock::new(),
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_messages_url() {
        let client = VertexClient::builder()
            .project_id("test-project")
            .region("us-central1")
            .access_token("test-token")
            .build()
            .await
            .unwrap();

        let url = client.messages_url("claude-3-5-sonnet-20241022", false);
        assert!(url.contains("test-project"));
        assert!(url.contains("us-central1"));
        assert!(url.contains("claude-3-5-sonnet-20241022"));
        assert!(url.contains("rawPredict"));
        assert!(!url.contains("streamRawPredict"));

        let stream_url = client.messages_url("claude-3-5-sonnet-20241022", true);
        assert!(stream_url.contains("streamRawPredict"));
    }

    #[tokio::test]
    async fn test_builder_with_explicit_token() {
        let client = VertexClient::builder()
            .project_id("test-project")
            .region("us-central1")
            .access_token("ya29.test_token")
            .build()
            .await;

        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.project_id(), "test-project");
        assert_eq!(client.region(), "us-central1");
    }

    #[tokio::test]
    async fn test_builder_missing_region() {
        let result = VertexClient::builder()
            .project_id("test-project")
            .access_token("test-token")
            .build()
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Region must be provided"));
    }

    #[tokio::test]
    async fn test_builder_missing_project_id() {
        let result = VertexClient::builder()
            .region("us-central1")
            .access_token("test-token")
            .build()
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Project ID not provided"));
    }
}
