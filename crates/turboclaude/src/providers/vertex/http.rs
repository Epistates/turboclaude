//! VertexHttpProvider implementation
//!
//! This module provides an HTTP provider for Google Cloud Vertex AI.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;
use url::Url;

use crate::{
    error::Result,
    http::{HttpProvider, Method, RequestBuilder, Response},
};

use super::{VERTEX_API_VERSION, error::VertexError};

/// HTTP provider for Google Vertex AI.
///
/// This provider implements the `HttpProvider` trait for Google Cloud's Vertex AI,
/// handling Google Cloud authentication, request/response translation, and streaming support.
///
/// # Architecture
///
/// - Uses `google-cloud-auth` for GCP authentication
/// - Constructs region-specific Vertex AI endpoints
/// - Injects `anthropic_version` into request body
/// - Model ID specified in URL path (not request body)
///
/// # Example
///
/// ```rust,no_run
/// use turboclaude::providers::vertex::VertexHttpProvider;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = Arc::new(VertexHttpProvider::builder()
///     .project_id("my-gcp-project")
///     .region("us-east5")
///     .build()
///     .await?);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct VertexHttpProvider {
    pub(crate) inner: Arc<ProviderInner>,
}

#[derive(Debug)]
pub(crate) struct ProviderInner {
    /// Google Cloud project ID
    pub(crate) project_id: String,
    /// GCP region (e.g., "us-east5", "europe-west1")
    pub(crate) region: String,
    /// Access token for authentication
    pub(crate) access_token: Option<String>,
    /// HTTP client for making requests
    pub(crate) client: reqwest::Client,
    /// Default timeout for requests
    #[allow(dead_code)]
    pub(crate) timeout: Duration,
}

impl VertexHttpProvider {
    /// Create a new builder for configuring the provider.
    pub fn builder() -> VertexHttpProviderBuilder {
        VertexHttpProviderBuilder::default()
    }

    /// Construct the Vertex AI endpoint URL for a given model and operation
    fn build_endpoint_url(&self, model: &str, streaming: bool) -> Result<String> {
        let operation = if streaming {
            "streamRawPredict"
        } else {
            "rawPredict"
        };

        Ok(format!(
            "https://{region}-aiplatform.googleapis.com/v1/projects/{project}/locations/{region}/publishers/anthropic/models/{model}:{operation}",
            region = self.inner.region,
            project = self.inner.project_id,
            model = model,
            operation = operation
        ))
    }

    /// Extract model from MessageRequest and inject anthropic_version
    fn prepare_request_body(
        &self,
        body: &(dyn erased_serde::Serialize + Send + Sync),
    ) -> Result<(String, serde_json::Value)> {
        // Serialize to JSON to manipulate
        let json_bytes = serde_json::to_vec(body).map_err(crate::error::Error::Serialization)?;
        let mut request_json: serde_json::Value =
            serde_json::from_slice(&json_bytes).map_err(crate::error::Error::Serialization)?;

        // Extract model from request
        let model = request_json
            .get("model")
            .and_then(|m| m.as_str())
            .ok_or_else(|| {
                crate::error::Error::InvalidRequest(
                    "Model field is required in request".to_string(),
                )
            })?
            .to_string();

        // Remove model from request body (goes in URL for Vertex)
        if let Some(obj) = request_json.as_object_mut() {
            obj.remove("model");
            // Inject anthropic_version into request body
            obj.insert(
                "anthropic_version".to_string(),
                serde_json::Value::String(VERTEX_API_VERSION.to_string()),
            );
        }

        Ok((model, request_json))
    }
}

#[async_trait]
impl HttpProvider for VertexHttpProvider {
    async fn request(
        &self,
        _method: Method,
        path: &str,
        body: Option<&(dyn erased_serde::Serialize + Send + Sync)>,
    ) -> Result<Response> {
        // Vertex only supports messages endpoint
        if !path.contains("/messages") {
            return Err(crate::error::Error::InvalidUrl(format!(
                "Vertex provider only supports /messages endpoint, got: {}",
                path
            )));
        }

        if let Some(body) = body {
            // Prepare request: extract model, inject anthropic_version
            let (model, request_body) = self.prepare_request_body(body)?;

            // Build endpoint URL
            let url = self.build_endpoint_url(&model, false)?;

            // Build HTTP request
            let mut request = self.inner.client.post(&url).json(&request_body);

            // Add authentication
            if let Some(token) = &self.inner.access_token {
                request = request.bearer_auth(token);
            }

            // Execute request
            let response = request
                .send()
                .await
                .map_err(|e| VertexError::Http(e.to_string()))?;

            // Check for errors
            let status = response.status();
            if !status.is_success() {
                let error_body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                return Err(VertexError::Api(format!("Status {}: {}", status, error_body)).into());
            }

            // Get response bytes
            let response_bytes = response
                .bytes()
                .await
                .map_err(|e| VertexError::Http(e.to_string()))?;

            Ok(Response::new(
                http::StatusCode::OK,
                Default::default(),
                response_bytes.to_vec(),
            ))
        } else {
            Err(crate::error::Error::InvalidRequest(
                "Request body is required for messages endpoint".to_string(),
            ))
        }
    }

    async fn request_streaming(
        &self,
        _method: Method,
        path: &str,
        body: Option<&(dyn erased_serde::Serialize + Send + Sync)>,
    ) -> Result<Box<dyn Stream<Item = Result<Bytes>> + Send + Unpin>> {
        // Vertex only supports messages endpoint
        if !path.contains("/messages") {
            return Err(crate::error::Error::InvalidUrl(format!(
                "Vertex provider only supports /messages endpoint, got: {}",
                path
            )));
        }

        if let Some(body) = body {
            // Prepare request: extract model, inject anthropic_version
            let (model, request_body) = self.prepare_request_body(body)?;

            // Build streaming endpoint URL
            let url = self.build_endpoint_url(&model, true)?;

            // Build HTTP request
            let mut request = self.inner.client.post(&url).json(&request_body);

            // Add authentication
            if let Some(token) = &self.inner.access_token {
                request = request.bearer_auth(token);
            }

            // Execute streaming request
            let response = request
                .send()
                .await
                .map_err(|e| VertexError::Http(e.to_string()))?;

            // Check for errors
            let status = response.status();
            if !status.is_success() {
                let error_body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                return Err(VertexError::Api(format!("Status {}: {}", status, error_body)).into());
            }

            // Convert response into byte stream
            let byte_stream = response.bytes_stream();
            let mapped_stream = futures::StreamExt::map(byte_stream, |result| {
                result.map_err(|e| VertexError::Http(e.to_string()).into())
            });

            Ok(Box::new(Box::pin(mapped_stream)))
        } else {
            Err(crate::error::Error::InvalidRequest(
                "Request body is required for messages endpoint".to_string(),
            ))
        }
    }

    fn create_request(&self, method: Method, _path: &str) -> Result<RequestBuilder> {
        // Vertex doesn't use traditional HTTP request building
        // This method is only used by Resources that build requests manually
        // For Vertex, we handle everything in request() and request_streaming()
        let url = Url::parse(&format!(
            "https://{}-aiplatform.googleapis.com/v1",
            self.inner.region
        ))
        .map_err(|e| crate::error::Error::InvalidUrl(e.to_string()))?;

        Ok(RequestBuilder::new(method, url))
    }

    fn provider_name(&self) -> &'static str {
        "vertex"
    }

    fn supports_beta(&self) -> bool {
        false // Vertex doesn't have beta endpoints
    }

    fn base_url(&self) -> &str {
        // Return a representative URL for debugging
        "https://aiplatform.googleapis.com"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Builder for creating a `VertexHttpProvider` with custom configuration.
///
/// # Example
///
/// ```rust,no_run
/// use turboclaude::providers::vertex::VertexHttpProvider;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = VertexHttpProvider::builder()
///     .project_id("my-gcp-project")
///     .region("us-east5")
///     .timeout(std::time::Duration::from_secs(120))
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct VertexHttpProviderBuilder {
    project_id: Option<String>,
    region: Option<String>,
    access_token: Option<String>,
    timeout: Option<Duration>,
}

impl VertexHttpProviderBuilder {
    /// Set the Google Cloud project ID.
    ///
    /// Required. Can also be inferred from GOOGLE_CLOUD_PROJECT environment variable.
    pub fn project_id(mut self, project_id: impl Into<String>) -> Self {
        self.project_id = Some(project_id.into());
        self
    }

    /// Set the GCP region (e.g., "us-east5", "europe-west1").
    ///
    /// Required. This determines which regional endpoint to use.
    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    /// Set an explicit access token for authentication.
    ///
    /// If not provided, will attempt to use Application Default Credentials.
    pub fn access_token(mut self, token: impl Into<String>) -> Self {
        self.access_token = Some(token.into());
        self
    }

    /// Set the request timeout.
    ///
    /// Defaults to 600 seconds (10 minutes).
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Build the provider with the configured settings.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Project ID is not provided
    /// - Region is not provided
    /// - Authentication fails
    pub async fn build(self) -> Result<VertexHttpProvider> {
        // Get project ID from config or environment
        let project_id = self
            .project_id
            .or_else(|| std::env::var("GOOGLE_CLOUD_PROJECT").ok())
            .ok_or_else(|| {
                VertexError::InvalidProjectId(
                    "Project ID must be provided or set in GOOGLE_CLOUD_PROJECT".to_string(),
                )
            })?;

        // Get region
        let region = self.region.ok_or_else(|| {
            VertexError::InvalidRegion("Region must be provided (e.g., 'us-east5')".to_string())
        })?;

        // Get or create access token using Application Default Credentials (ADC)
        let access_token = if let Some(token) = self.access_token {
            // Explicit token provided - use it
            Some(token)
        } else {
            // Attempt to use Application Default Credentials via google-cloud-auth 1.0 API
            tracing::info!("Attempting to load Application Default Credentials (ADC)");

            match google_cloud_auth::credentials::Builder::default().build() {
                Ok(credentials) => {
                    // Get authentication headers which contain the access token
                    match credentials.headers(http::Extensions::new()).await {
                        Ok(google_cloud_auth::credentials::CacheableResource::New {
                            data: headers,
                            ..
                        }) => {
                            // Extract Bearer token from Authorization header
                            if let Some(auth_value) = headers.get(http::header::AUTHORIZATION) {
                                if let Ok(auth_str) = auth_value.to_str() {
                                    if let Some(token) = auth_str.strip_prefix("Bearer ") {
                                        tracing::info!("Successfully loaded ADC token");
                                        Some(token.to_string())
                                    } else {
                                        tracing::warn!(
                                            "Authorization header does not contain Bearer token. Vertex requests may fail."
                                        );
                                        None
                                    }
                                } else {
                                    tracing::warn!(
                                        "Failed to parse Authorization header. Vertex requests may fail."
                                    );
                                    None
                                }
                            } else {
                                tracing::warn!(
                                    "No Authorization header in credentials. Vertex requests may fail."
                                );
                                None
                            }
                        }
                        Ok(google_cloud_auth::credentials::CacheableResource::NotModified) => {
                            tracing::warn!(
                                "Credentials not modified but no cached token available. Vertex requests may fail."
                            );
                            None
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to get credentials headers: {}. Vertex requests may fail without explicit token.",
                                e
                            );
                            None
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Application Default Credentials not available: {}. Provide explicit token via .access_token() or run: gcloud auth application-default login",
                        e
                    );
                    None
                }
            }
        };

        // Build HTTP client
        let client = reqwest::Client::builder()
            .timeout(self.timeout.unwrap_or(Duration::from_secs(600)))
            .build()
            .map_err(|e| VertexError::Http(e.to_string()))?;

        let inner = Arc::new(ProviderInner {
            project_id,
            region,
            access_token,
            client,
            timeout: self.timeout.unwrap_or(Duration::from_secs(600)),
        });

        Ok(VertexHttpProvider { inner })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_endpoint_url() {
        let provider = VertexHttpProvider {
            inner: Arc::new(ProviderInner {
                project_id: "test-project".to_string(),
                region: "us-east5".to_string(),
                access_token: None,
                client: reqwest::Client::new(),
                timeout: Duration::from_secs(600),
            }),
        };

        let url = provider
            .build_endpoint_url("claude-sonnet-4-5@20250929", false)
            .unwrap();
        assert!(url.contains("us-east5-aiplatform.googleapis.com"));
        assert!(url.contains("test-project"));
        assert!(url.contains("claude-sonnet-4-5@20250929"));
        assert!(url.contains(":rawPredict"));

        let streaming_url = provider
            .build_endpoint_url("claude-sonnet-4-5@20250929", true)
            .unwrap();
        assert!(streaming_url.contains(":streamRawPredict"));
    }

    #[test]
    fn test_provider_metadata() {
        let provider = VertexHttpProvider {
            inner: Arc::new(ProviderInner {
                project_id: "test-project".to_string(),
                region: "us-east5".to_string(),
                access_token: None,
                client: reqwest::Client::new(),
                timeout: Duration::from_secs(600),
            }),
        };

        assert_eq!(provider.provider_name(), "vertex");
        assert!(!provider.supports_beta());
    }

    #[tokio::test]
    async fn test_builder() {
        let result = VertexHttpProvider::builder()
            .project_id("test-project")
            .region("us-east5")
            .access_token("test-token")
            .build()
            .await;

        assert!(result.is_ok());
        let provider = result.unwrap();
        assert_eq!(provider.inner.project_id, "test-project");
        assert_eq!(provider.inner.region, "us-east5");
        assert_eq!(provider.inner.access_token, Some("test-token".to_string()));
    }
}
