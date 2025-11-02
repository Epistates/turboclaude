//! BedrockHttpProvider implementation
//!
//! This module provides an HTTP provider for AWS Bedrock's Converse API.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_bedrockruntime::{
    Client as BedrockRuntimeClient, config::Credentials as AwsCredentials,
};
use bytes::Bytes;
use futures::Stream;
use url::Url;

use crate::{
    error::Result,
    http::{HttpProvider, Method, RequestBuilder, Response},
};

/// HTTP provider for AWS Bedrock.
///
/// This provider implements the `HttpProvider` trait for AWS Bedrock's Converse API,
/// handling AWS authentication, request/response translation, and streaming support.
///
/// # Architecture
///
/// - Uses official AWS SDK for Rust (`aws-sdk-bedrockruntime`)
/// - Translates turboclaude types ↔ Bedrock types
/// - Supports streaming via ConverseStream
/// - Handles AWS credentials via standard credential chain
///
/// # Example
///
/// ```rust,no_run
/// use turboclaude::providers::bedrock::BedrockHttpProvider;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = Arc::new(BedrockHttpProvider::builder()
///     .region("us-east-1")
///     .build()
///     .await?);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct BedrockHttpProvider {
    pub(crate) inner: Arc<ProviderInner>,
}

#[derive(Debug)]
pub(crate) struct ProviderInner {
    /// AWS Bedrock Runtime client
    pub(crate) bedrock: BedrockRuntimeClient,
    /// AWS region
    pub(crate) region: String,
    /// Default timeout for requests (reserved for future timeout implementation)
    #[allow(dead_code)]
    pub(crate) timeout: Duration,
    /// Maximum number of retries (reserved for future retry implementation)
    #[allow(dead_code)]
    pub(crate) max_retries: u32,
}

impl BedrockHttpProvider {
    /// Create a new builder for configuring the provider.
    pub fn builder() -> BedrockHttpProviderBuilder {
        BedrockHttpProviderBuilder::default()
    }

    /// Transform a model ID to Bedrock format if needed
    ///
    /// Converts short model names like "claude-3-5-sonnet-20241022" to
    /// Bedrock format like "anthropic.claude-3-5-sonnet-20241022-v2:0"
    pub(crate) fn normalize_model_id(model: &str) -> String {
        if model.starts_with("anthropic.") {
            // Already in Bedrock format
            model.to_string()
        } else if model.contains(":") {
            // Has version suffix, just add anthropic prefix
            format!("anthropic.{}", model)
        } else {
            // Need to add prefix and infer version
            // This is a simplified heuristic - may need refinement
            if model.contains("3-5-sonnet-20241022") {
                format!("anthropic.{}-v2:0", model)
            } else {
                format!("anthropic.{}-v1:0", model)
            }
        }
    }
}

#[async_trait]
impl HttpProvider for BedrockHttpProvider {
    async fn request(
        &self,
        _method: Method,
        path: &str,
        body: Option<&(dyn erased_serde::Serialize + Send + Sync)>,
    ) -> Result<Response> {
        // Bedrock only supports messages endpoint via Converse API
        if !path.contains("/messages") {
            return Err(crate::error::Error::InvalidUrl(format!(
                "Bedrock provider only supports /messages endpoint, got: {}",
                path
            )));
        }

        if let Some(body) = body {
            // Deserialize the MessageRequest from the generic body
            let json_bytes =
                serde_json::to_vec(body).map_err(crate::error::Error::Serialization)?;
            let request: crate::types::MessageRequest =
                serde_json::from_slice(&json_bytes).map_err(crate::error::Error::Serialization)?;

            // Translate to Bedrock and execute
            let message =
                super::translate::converse_non_streaming(&self.inner.bedrock, &request).await?;

            // Serialize response as JSON and wrap in Response
            let response_json =
                serde_json::to_vec(&message).map_err(crate::error::Error::Serialization)?;

            Ok(Response::new(
                http::StatusCode::OK,
                Default::default(),
                response_json,
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
        // Bedrock only supports messages endpoint via ConverseStream
        if !path.contains("/messages") {
            return Err(crate::error::Error::InvalidUrl(format!(
                "Bedrock provider only supports /messages endpoint, got: {}",
                path
            )));
        }

        if let Some(body) = body {
            // Deserialize the MessageRequest from the generic body
            let json_bytes =
                serde_json::to_vec(body).map_err(crate::error::Error::Serialization)?;
            let request: crate::types::MessageRequest =
                serde_json::from_slice(&json_bytes).map_err(crate::error::Error::Serialization)?;

            // Get streaming response from Bedrock
            let stream =
                super::translate::converse_streaming(&self.inner.bedrock, &request).await?;

            Ok(stream)
        } else {
            Err(crate::error::Error::InvalidRequest(
                "Request body is required for messages endpoint".to_string(),
            ))
        }
    }

    fn create_request(&self, method: Method, _path: &str) -> Result<RequestBuilder> {
        // Bedrock doesn't use traditional HTTP requests, but we need to return something
        // This method is only used by Resources that build requests manually
        // For Bedrock, we handle everything in request() and request_streaming()
        let url = Url::parse(&format!(
            "https://bedrock-runtime.{}.amazonaws.com/model/converse",
            self.inner.region
        ))
        .map_err(|e| crate::error::Error::InvalidUrl(e.to_string()))?;

        Ok(RequestBuilder::new(method, url))
    }

    fn provider_name(&self) -> &'static str {
        "bedrock"
    }

    fn supports_beta(&self) -> bool {
        false // Bedrock doesn't have beta endpoints
    }

    fn base_url(&self) -> &str {
        // Return a representative URL for debugging
        "https://bedrock-runtime.amazonaws.com"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Builder for creating a `BedrockHttpProvider` with custom configuration.
///
/// # Example
///
/// ```rust,no_run
/// use turboclaude::providers::bedrock::BedrockHttpProvider;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = BedrockHttpProvider::builder()
///     .region("us-east-1")
///     .timeout(std::time::Duration::from_secs(120))
///     .max_retries(3)
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct BedrockHttpProviderBuilder {
    region: Option<String>,
    aws_access_key: Option<String>,
    aws_secret_key: Option<String>,
    aws_session_token: Option<String>,
    aws_profile: Option<String>,
    endpoint_url: Option<String>,
    timeout: Option<Duration>,
    max_retries: Option<u32>,
}

impl BedrockHttpProviderBuilder {
    /// Set the AWS region (e.g., "us-east-1").
    ///
    /// If not set, will attempt to infer from AWS_REGION environment variable.
    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    /// Set AWS access key ID for authentication.
    pub fn aws_access_key(mut self, key: impl Into<String>) -> Self {
        self.aws_access_key = Some(key.into());
        self
    }

    /// Set AWS secret access key for authentication.
    pub fn aws_secret_key(mut self, key: impl Into<String>) -> Self {
        self.aws_secret_key = Some(key.into());
        self
    }

    /// Set AWS session token (for temporary credentials).
    pub fn aws_session_token(mut self, token: impl Into<String>) -> Self {
        self.aws_session_token = Some(token.into());
        self
    }

    /// Set AWS profile name (from ~/.aws/credentials).
    pub fn aws_profile(mut self, profile: impl Into<String>) -> Self {
        self.aws_profile = Some(profile.into());
        self
    }

    /// Set a custom Bedrock endpoint URL (for testing or VPC endpoints).
    pub fn endpoint_url(mut self, url: impl Into<String>) -> Self {
        self.endpoint_url = Some(url.into());
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
    /// - AWS SDK configuration fails
    /// - Credentials cannot be loaded
    pub async fn build(self) -> Result<BedrockHttpProvider> {
        // Determine region
        let region = self
            .region
            .or_else(|| std::env::var("AWS_REGION").ok())
            .unwrap_or_else(|| {
                tracing::warn!("No AWS region specified, defaulting to us-east-1");
                "us-east-1".to_string()
            });

        // Load AWS configuration
        let mut config_loader =
            aws_config::defaults(BehaviorVersion::latest()).region(Region::new(region.clone()));

        // Set profile if specified
        if let Some(profile) = &self.aws_profile {
            config_loader = config_loader.profile_name(profile);
        }

        // Set explicit credentials if provided
        if let (Some(access_key), Some(secret_key)) = (&self.aws_access_key, &self.aws_secret_key) {
            let creds = if let Some(session_token) = &self.aws_session_token {
                AwsCredentials::new(
                    access_key,
                    secret_key,
                    Some(session_token.clone()),
                    None,
                    "explicit",
                )
            } else {
                AwsCredentials::new(access_key, secret_key, None, None, "explicit")
            };

            config_loader = config_loader.credentials_provider(creds);
        }

        let aws_config = config_loader.load().await;

        // Build Bedrock Runtime client
        let mut bedrock_config = aws_sdk_bedrockruntime::config::Builder::from(&aws_config);

        // Set custom endpoint if provided
        if let Some(endpoint) = &self.endpoint_url {
            bedrock_config = bedrock_config.endpoint_url(endpoint);
        }

        let bedrock = BedrockRuntimeClient::from_conf(bedrock_config.build());

        let inner = Arc::new(ProviderInner {
            bedrock,
            region,
            timeout: self.timeout.unwrap_or(Duration::from_secs(600)),
            max_retries: self.max_retries.unwrap_or(2),
        });

        Ok(BedrockHttpProvider { inner })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_model_id() {
        // Already in Bedrock format
        assert_eq!(
            BedrockHttpProvider::normalize_model_id("anthropic.claude-3-5-sonnet-20241022-v2:0"),
            "anthropic.claude-3-5-sonnet-20241022-v2:0"
        );

        // Short format with version
        assert_eq!(
            BedrockHttpProvider::normalize_model_id("claude-3-opus-20240229-v1:0"),
            "anthropic.claude-3-opus-20240229-v1:0"
        );

        // Short format without version (3.5 Sonnet latest)
        assert_eq!(
            BedrockHttpProvider::normalize_model_id("claude-3-5-sonnet-20241022"),
            "anthropic.claude-3-5-sonnet-20241022-v2:0"
        );

        // Short format without version (other models)
        assert_eq!(
            BedrockHttpProvider::normalize_model_id("claude-3-haiku-20240307"),
            "anthropic.claude-3-haiku-20240307-v1:0"
        );
    }

    #[tokio::test]
    async fn test_builder_with_explicit_credentials() {
        let builder = BedrockHttpProvider::builder()
            .region("us-west-2")
            .aws_access_key("AKIAIOSFODNN7EXAMPLE")
            .aws_secret_key("wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY");

        // Should not panic (though credentials are fake)
        assert!(builder.build().await.is_ok());
    }

    #[test]
    fn test_provider_name() {
        let provider = BedrockHttpProvider {
            inner: Arc::new(ProviderInner {
                bedrock: BedrockRuntimeClient::from_conf(
                    aws_sdk_bedrockruntime::Config::builder()
                        .behavior_version(aws_config::BehaviorVersion::latest())
                        .region(Region::new("us-east-1"))
                        .build(),
                ),
                region: "us-east-1".to_string(),
                timeout: Duration::from_secs(600),
                max_retries: 2,
            }),
        };

        assert_eq!(provider.provider_name(), "bedrock");
        assert!(!provider.supports_beta());
    }
}
