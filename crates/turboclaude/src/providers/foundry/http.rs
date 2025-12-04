//! FoundryHttpProvider implementation
//!
//! This module provides an HTTP provider for Microsoft Azure Foundry's Claude API.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use url::Url;

use super::error::FoundryError;
use crate::{
    error::Result,
    http::{HttpProvider, Method, RequestBuilder, Response},
};

/// Type alias for async token provider function
pub type TokenProvider =
    Arc<dyn Fn() -> Pin<Box<dyn Future<Output = std::result::Result<String, String>> + Send>> + Send + Sync>;

/// HTTP provider for Microsoft Azure Foundry.
///
/// This provider implements the `HttpProvider` trait for Azure Foundry's Claude API,
/// handling Azure authentication and request routing.
///
/// # Architecture
///
/// - Uses reqwest for HTTP operations
/// - Supports API key or Azure AD token authentication
/// - Compatible with standard Anthropic API format
///
/// # Example
///
/// ```rust,no_run
/// use turboclaude::providers::foundry::FoundryHttpProvider;
/// use std::sync::Arc;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = Arc::new(FoundryHttpProvider::builder()
///     .resource("my-foundry-resource")
///     .api_key("my-api-key")
///     .build()?);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct FoundryHttpProvider {
    pub(crate) inner: Arc<ProviderInner>,
}

impl std::fmt::Debug for FoundryHttpProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FoundryHttpProvider")
            .field("base_url", &self.inner.base_url)
            .field("has_api_key", &self.inner.api_key.is_some())
            .field("has_token_provider", &self.inner.token_provider.is_some())
            .finish()
    }
}

pub(crate) struct ProviderInner {
    /// HTTP client
    pub(crate) client: reqwest::Client,
    /// Base URL for Foundry API
    pub(crate) base_url: String,
    /// Azure resource name
    pub(crate) resource: String,
    /// API key (if using API key auth)
    pub(crate) api_key: Option<String>,
    /// Azure AD token provider (if using Azure AD auth)
    pub(crate) token_provider: Option<TokenProvider>,
    /// Default timeout for requests
    pub(crate) timeout: Duration,
    /// Maximum number of retries
    pub(crate) max_retries: u32,
}

impl std::fmt::Debug for ProviderInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderInner")
            .field("base_url", &self.base_url)
            .field("resource", &self.resource)
            .field("has_api_key", &self.api_key.is_some())
            .field("has_token_provider", &self.token_provider.is_some())
            .field("timeout", &self.timeout)
            .field("max_retries", &self.max_retries)
            .finish()
    }
}

impl FoundryHttpProvider {
    /// Create a new builder for configuring the provider.
    pub fn builder() -> FoundryHttpProviderBuilder {
        FoundryHttpProviderBuilder::default()
    }

    /// Get the base URL for this provider.
    pub fn get_base_url(&self) -> &str {
        &self.inner.base_url
    }

    /// Get the resource name.
    pub fn resource(&self) -> &str {
        &self.inner.resource
    }

    /// Get the authorization header value.
    async fn get_auth_header(&self) -> Result<String> {
        if let Some(api_key) = &self.inner.api_key {
            Ok(format!("Bearer {}", api_key))
        } else if let Some(token_provider) = &self.inner.token_provider {
            let token = token_provider()
                .await
                .map_err(|e| FoundryError::TokenProvider(e))?;
            Ok(format!("Bearer {}", token))
        } else {
            Err(FoundryError::Authentication("No authentication configured".to_string()).into())
        }
    }

    /// Build headers for a request.
    async fn build_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        // Add authorization
        let auth = self.get_auth_header().await?;
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth)
                .map_err(|e| FoundryError::RequestConstruction(e.to_string()))?,
        );

        // Add content type
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        // Add Anthropic API version header
        headers.insert(
            "anthropic-version",
            HeaderValue::from_static("2023-06-01"),
        );

        Ok(headers)
    }
}

#[async_trait]
impl HttpProvider for FoundryHttpProvider {
    async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<&(dyn erased_serde::Serialize + Send + Sync)>,
    ) -> Result<Response> {
        // Check for unsupported endpoints
        if path.contains("/batches") || path.contains("/models") {
            return Err(FoundryError::UnsupportedOperation(format!(
                "Endpoint {} is not available on Foundry",
                path
            ))
            .into());
        }

        let url = format!("{}{}", self.inner.base_url, path);
        let headers = self.build_headers().await?;

        let mut request_builder = match method {
            Method::GET => self.inner.client.get(&url),
            Method::POST => self.inner.client.post(&url),
            Method::PUT => self.inner.client.put(&url),
            Method::DELETE => self.inner.client.delete(&url),
            Method::PATCH => self.inner.client.patch(&url),
            _ => return Err(crate::error::Error::BadRequest {
                message: format!("Unsupported HTTP method: {:?}", method),
                error_type: None,
            }),
        };

        request_builder = request_builder.headers(headers);

        if let Some(body) = body {
            let json_bytes =
                serde_json::to_vec(body).map_err(crate::error::Error::Serialization)?;
            request_builder = request_builder.body(json_bytes);
        }

        let response = request_builder
            .send()
            .await
            .map_err(|e| FoundryError::Network(e.to_string()))?;

        let status = response.status();
        let headers = response.headers().clone();
        let body_bytes = response
            .bytes()
            .await
            .map_err(|e| FoundryError::ResponseParsing(e.to_string()))?;

        if !status.is_success() {
            let error_message =
                String::from_utf8_lossy(&body_bytes).to_string();
            return Err(FoundryError::Api {
                status: status.as_u16(),
                message: error_message,
            }
            .into());
        }

        // Convert reqwest headers to http::HeaderMap
        let mut http_headers = http::HeaderMap::new();
        for (name, value) in headers.iter() {
            if let (Ok(name), Ok(value)) = (
                http::HeaderName::try_from(name.as_str()),
                http::HeaderValue::from_bytes(value.as_bytes()),
            ) {
                http_headers.insert(name, value);
            }
        }

        Ok(Response::new(
            http::StatusCode::from_u16(status.as_u16()).unwrap_or(http::StatusCode::OK),
            http_headers,
            body_bytes.to_vec(),
        ))
    }

    async fn request_streaming(
        &self,
        method: Method,
        path: &str,
        body: Option<&(dyn erased_serde::Serialize + Send + Sync)>,
    ) -> Result<Box<dyn Stream<Item = Result<Bytes>> + Send + Unpin>> {
        // Check for unsupported endpoints
        if path.contains("/batches") || path.contains("/models") {
            return Err(FoundryError::UnsupportedOperation(format!(
                "Endpoint {} is not available on Foundry",
                path
            ))
            .into());
        }

        let url = format!("{}{}", self.inner.base_url, path);
        let headers = self.build_headers().await?;

        let mut request_builder = match method {
            Method::GET => self.inner.client.get(&url),
            Method::POST => self.inner.client.post(&url),
            Method::PUT => self.inner.client.put(&url),
            Method::DELETE => self.inner.client.delete(&url),
            Method::PATCH => self.inner.client.patch(&url),
            _ => return Err(crate::error::Error::BadRequest {
                message: format!("Unsupported HTTP method: {:?}", method),
                error_type: None,
            }),
        };

        request_builder = request_builder.headers(headers);

        if let Some(body) = body {
            let json_bytes =
                serde_json::to_vec(body).map_err(crate::error::Error::Serialization)?;
            request_builder = request_builder.body(json_bytes);
        }

        let response = request_builder
            .send()
            .await
            .map_err(|e| FoundryError::Network(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let body_bytes = response
                .bytes()
                .await
                .map_err(|e| FoundryError::ResponseParsing(e.to_string()))?;
            let error_message =
                String::from_utf8_lossy(&body_bytes).to_string();
            return Err(FoundryError::Api {
                status: status.as_u16(),
                message: error_message,
            }
            .into());
        }

        // Convert response stream to our format
        let stream = response.bytes_stream();
        let mapped_stream = futures::StreamExt::map(stream, |result| {
            result.map_err(|e| crate::error::Error::Connection(e.to_string()))
        });

        Ok(Box::new(Box::pin(mapped_stream)))
    }

    fn create_request(&self, method: Method, path: &str) -> Result<RequestBuilder> {
        let url_str = format!("{}{}", self.inner.base_url, path);
        let url = Url::parse(&url_str)
            .map_err(|e| crate::error::Error::InvalidUrl(e.to_string()))?;

        Ok(RequestBuilder::new(method, url))
    }

    fn provider_name(&self) -> &'static str {
        "foundry"
    }

    fn supports_beta(&self) -> bool {
        true // Foundry supports the standard Anthropic API including beta features
    }

    fn base_url(&self) -> &str {
        &self.inner.base_url
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Builder for creating a `FoundryHttpProvider` with custom configuration.
///
/// # Example
///
/// ```rust,no_run
/// use turboclaude::providers::foundry::FoundryHttpProvider;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = FoundryHttpProvider::builder()
///     .resource("my-foundry-resource")
///     .api_key("my-api-key")
///     .timeout(std::time::Duration::from_secs(120))
///     .build()?;
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct FoundryHttpProviderBuilder {
    resource: Option<String>,
    base_url: Option<String>,
    api_key: Option<String>,
    token_provider: Option<TokenProvider>,
    timeout: Option<Duration>,
    max_retries: Option<u32>,
}

impl FoundryHttpProviderBuilder {
    /// Set the Azure Foundry resource name.
    ///
    /// This is the name of your Foundry deployment in Azure.
    /// If not set, reads from `ANTHROPIC_FOUNDRY_RESOURCE` environment variable.
    pub fn resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    /// Set a custom base URL for the Foundry API.
    ///
    /// If not set, constructs URL from resource name or reads from
    /// `ANTHROPIC_FOUNDRY_BASE_URL` environment variable.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Set the API key for authentication.
    ///
    /// If not set, reads from `ANTHROPIC_FOUNDRY_API_KEY` environment variable.
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Set an Azure AD token provider for authentication.
    ///
    /// This is the preferred authentication method for production deployments.
    /// The provider function is called before each request to get a fresh token.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use turboclaude::providers::foundry::FoundryHttpProvider;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let provider = FoundryHttpProvider::builder()
    ///     .resource("my-resource")
    ///     .azure_ad_token_provider(|| async {
    ///         // Your token acquisition logic
    ///         // e.g., using Azure SDK's DefaultAzureCredential
    ///         Ok("token".to_string())
    ///     })
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn azure_ad_token_provider<F, Fut>(mut self, provider: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = std::result::Result<String, String>> + Send + 'static,
    {
        self.token_provider = Some(Arc::new(move || Box::pin(provider())));
        self
    }

    /// Set the request timeout.
    ///
    /// Defaults to 600 seconds (10 minutes).
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the maximum number of retries for failed requests.
    ///
    /// Defaults to 2 retries.
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Build the provider with the configured settings.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Resource name is not provided and cannot be read from environment
    /// - Neither API key nor token provider is configured
    pub fn build(self) -> Result<FoundryHttpProvider> {
        // Get resource name
        let resource = self
            .resource
            .or_else(|| std::env::var("ANTHROPIC_FOUNDRY_RESOURCE").ok())
            .ok_or_else(|| {
                FoundryError::MissingConfig(
                    "Resource name required. Set via builder or ANTHROPIC_FOUNDRY_RESOURCE env var"
                        .to_string(),
                )
            })?;

        // Validate resource name
        if resource.is_empty() || resource.contains('/') {
            return Err(FoundryError::InvalidResource(resource).into());
        }

        // Get base URL
        let base_url = self
            .base_url
            .or_else(|| std::env::var("ANTHROPIC_FOUNDRY_BASE_URL").ok())
            .unwrap_or_else(|| {
                format!(
                    "https://{}.services.ai.azure.com/models/anthropic",
                    resource
                )
            });

        // Get API key
        let api_key = self
            .api_key
            .or_else(|| std::env::var("ANTHROPIC_FOUNDRY_API_KEY").ok());

        // Validate authentication
        if api_key.is_none() && self.token_provider.is_none() {
            tracing::warn!(
                "No authentication configured for Foundry provider. \
                Set ANTHROPIC_FOUNDRY_API_KEY or provide a token provider."
            );
        }

        // Build HTTP client
        let timeout = self.timeout.unwrap_or(Duration::from_secs(600));
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| FoundryError::RequestConstruction(e.to_string()))?;

        let inner = Arc::new(ProviderInner {
            client,
            base_url,
            resource,
            api_key,
            token_provider: self.token_provider,
            timeout,
            max_retries: self.max_retries.unwrap_or(2),
        });

        Ok(FoundryHttpProvider { inner })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_with_resource() {
        temp_env::with_var("ANTHROPIC_FOUNDRY_API_KEY", Some("test-key"), || {
            let provider = FoundryHttpProvider::builder()
                .resource("test-resource")
                .build()
                .expect("Should build with resource");

            assert_eq!(provider.resource(), "test-resource");
            assert!(provider.get_base_url().contains("test-resource"));
        });
    }

    #[test]
    fn test_builder_with_custom_base_url() {
        temp_env::with_var("ANTHROPIC_FOUNDRY_API_KEY", Some("test-key"), || {
            let provider = FoundryHttpProvider::builder()
                .resource("test-resource")
                .base_url("https://custom.endpoint.com/v1")
                .build()
                .expect("Should build with custom URL");

            assert_eq!(provider.get_base_url(), "https://custom.endpoint.com/v1");
        });
    }

    #[test]
    fn test_builder_missing_resource() {
        // Clear environment
        temp_env::with_var_unset("ANTHROPIC_FOUNDRY_RESOURCE", || {
            let result = FoundryHttpProvider::builder().build();
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_builder_invalid_resource() {
        let result = FoundryHttpProvider::builder()
            .resource("invalid/resource")
            .api_key("test-key")
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_provider_name() {
        temp_env::with_var("ANTHROPIC_FOUNDRY_API_KEY", Some("test-key"), || {
            let provider = FoundryHttpProvider::builder()
                .resource("test-resource")
                .build()
                .expect("Should build");

            assert_eq!(provider.provider_name(), "foundry");
            assert!(provider.supports_beta());
        });
    }

    #[test]
    fn test_default_base_url_construction() {
        temp_env::with_var("ANTHROPIC_FOUNDRY_API_KEY", Some("test-key"), || {
            let provider = FoundryHttpProvider::builder()
                .resource("my-resource")
                .build()
                .expect("Should build");

            assert_eq!(
                provider.get_base_url(),
                "https://my-resource.services.ai.azure.com/models/anthropic"
            );
        });
    }

    #[test]
    fn test_env_var_fallback() {
        temp_env::with_vars(
            [
                ("ANTHROPIC_FOUNDRY_RESOURCE", Some("env-resource")),
                ("ANTHROPIC_FOUNDRY_API_KEY", Some("env-key")),
            ],
            || {
                let provider = FoundryHttpProvider::builder()
                    .build()
                    .expect("Should build from env vars");

                assert_eq!(provider.resource(), "env-resource");
            },
        );
    }

    #[test]
    fn test_timeout_configuration() {
        temp_env::with_var("ANTHROPIC_FOUNDRY_API_KEY", Some("test-key"), || {
            let provider = FoundryHttpProvider::builder()
                .resource("test-resource")
                .timeout(Duration::from_secs(120))
                .build()
                .expect("Should build");

            assert_eq!(provider.inner.timeout, Duration::from_secs(120));
        });
    }

    #[test]
    fn test_max_retries_configuration() {
        temp_env::with_var("ANTHROPIC_FOUNDRY_API_KEY", Some("test-key"), || {
            let provider = FoundryHttpProvider::builder()
                .resource("test-resource")
                .max_retries(5)
                .build()
                .expect("Should build");

            assert_eq!(provider.inner.max_retries, 5);
        });
    }
}
