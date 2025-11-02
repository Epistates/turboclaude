//! Models API tests
//!
//! Testing model listing and retrieval endpoints

use turboclaude::Client;
use rstest::*;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

mod common;
use common::{model_list_response, model_get_response};

#[tokio::test]
async fn test_models_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(model_list_response()))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let result = client.models().list().await;

    assert!(result.is_ok());
    let models = result.unwrap();
    assert!(!models.is_empty());
    assert_eq!(models[0].id, "claude-3-5-sonnet-20241022");
}

#[tokio::test]
async fn test_models_get() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/models/claude-3-5-sonnet-20241022"))
        .respond_with(ResponseTemplate::new(200).set_body_json(model_get_response()))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let result = client.models().get("claude-3-5-sonnet-20241022").await;

    assert!(result.is_ok());
    let model = result.unwrap();
    assert_eq!(model.id, "claude-3-5-sonnet-20241022");
    assert_eq!(model.display_name, "Claude 3.5 Sonnet");
}

#[tokio::test]
async fn test_models_get_not_found() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/models/non-existent"))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(serde_json::json!({
                    "type": "error",
                    "error": {
                        "type": "not_found_error",
                        "message": "Model not found"
                    }
                }))
        )
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let result = client.models().get("non-existent").await;

    assert!(result.is_err());
}

/// Parametrized test for various model IDs
#[rstest]
#[case("claude-3-5-sonnet-20241022")]
#[case("claude-3-opus-20240229")]
#[case("claude-3-sonnet-20240229")]
#[case("claude-3-haiku-20240307")]
#[tokio::test]
async fn test_models_get_various_ids(#[case] model_id: &str) {
    let mock_server = MockServer::start().await;

    let response = serde_json::json!({
        "type": "model",
        "id": model_id,
        "display_name": "Test Model",
        "created_at": "2024-01-01T00:00:00Z"
    });

    Mock::given(method("GET"))
        .and(path(format!("/v1/models/{}", model_id)))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let result = client.models().get(model_id).await;

    assert!(result.is_ok());
    let model = result.unwrap();
    assert_eq!(model.id, model_id);
}
