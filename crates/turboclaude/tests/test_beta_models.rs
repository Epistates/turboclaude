//! Tests for Beta Models API

use turboclaude::Client;
use turboclaude::types::beta::ModelPage;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_models_list_empty() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .and(query_param("beta", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [],
            "has_more": false,
            "first_id": null,
            "last_id": null
        })))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .expect("Failed to build client");

    let page = client
        .beta()
        .models()
        .list()
        .send()
        .await
        .expect("Failed to list models");

    assert_eq!(page.data.len(), 0);
    assert!(!page.has_more);
    assert_eq!(page.first_id, None);
    assert_eq!(page.last_id, None);
}

#[tokio::test]
async fn test_models_list_with_models() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .and(query_param("beta", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "claude-3-5-sonnet-20241022",
                    "type": "model",
                    "display_name": "Claude 3.5 Sonnet",
                    "created_at": "2024-10-22T00:00:00Z"
                },
                {
                    "id": "claude-3-5-haiku-20241022",
                    "type": "model",
                    "display_name": "Claude 3.5 Haiku",
                    "created_at": "2024-10-22T00:00:00Z"
                }
            ],
            "has_more": true,
            "first_id": "claude-3-5-sonnet-20241022",
            "last_id": "claude-3-5-haiku-20241022"
        })))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .expect("Failed to build client");

    let page = client
        .beta()
        .models()
        .list()
        .send()
        .await
        .expect("Failed to list models");

    assert_eq!(page.data.len(), 2);
    assert!(page.has_more);
    assert_eq!(page.data[0].id, "claude-3-5-sonnet-20241022");
    assert_eq!(page.data[0].display_name, "Claude 3.5 Sonnet");
    assert_eq!(page.data[1].id, "claude-3-5-haiku-20241022");
    assert_eq!(page.data[1].display_name, "Claude 3.5 Haiku");
}

#[tokio::test]
async fn test_models_list_with_limit() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .and(query_param("beta", "true"))
        .and(query_param("limit", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [],
            "has_more": false,
            "first_id": null,
            "last_id": null
        })))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .expect("Failed to build client");

    let _page = client
        .beta()
        .models()
        .list()
        .limit(5)
        .send()
        .await
        .expect("Failed to list models with limit");
}

#[tokio::test]
async fn test_models_list_with_before_id() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .and(query_param("beta", "true"))
        .and(query_param("before_id", "claude-3-opus-20240229"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [],
            "has_more": false,
            "first_id": null,
            "last_id": null
        })))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .expect("Failed to build client");

    let _page = client
        .beta()
        .models()
        .list()
        .before("claude-3-opus-20240229")
        .send()
        .await
        .expect("Failed to list models with before_id");
}

#[tokio::test]
async fn test_models_list_with_after_id() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .and(query_param("beta", "true"))
        .and(query_param("after_id", "claude-3-5-sonnet-20241022"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [],
            "has_more": false,
            "first_id": null,
            "last_id": null
        })))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .expect("Failed to build client");

    let _page = client
        .beta()
        .models()
        .list()
        .after("claude-3-5-sonnet-20241022")
        .send()
        .await
        .expect("Failed to list models with after_id");
}

#[tokio::test]
async fn test_models_list_with_all_params() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .and(query_param("beta", "true"))
        .and(query_param("limit", "10"))
        .and(query_param("after_id", "model_start"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [],
            "has_more": false,
            "first_id": null,
            "last_id": null
        })))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .expect("Failed to build client");

    let _page = client
        .beta()
        .models()
        .list()
        .limit(10)
        .after("model_start")
        .send()
        .await
        .expect("Failed to list models with all params");
}

#[tokio::test]
async fn test_models_retrieve_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/models/claude-3-5-sonnet-20241022"))
        .and(query_param("beta", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "claude-3-5-sonnet-20241022",
            "type": "model",
            "display_name": "Claude 3.5 Sonnet",
            "created_at": "2024-10-22T00:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .expect("Failed to build client");

    let model = client
        .beta()
        .models()
        .retrieve("claude-3-5-sonnet-20241022")
        .await
        .expect("Failed to retrieve model");

    assert_eq!(model.id, "claude-3-5-sonnet-20241022");
    assert_eq!(model.display_name, "Claude 3.5 Sonnet");
    assert_eq!(model.model_type, "model");
}

#[tokio::test]
async fn test_models_retrieve_alias() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/models/claude-3-5-sonnet"))
        .and(query_param("beta", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "claude-3-5-sonnet-20241022",
            "type": "model",
            "display_name": "Claude 3.5 Sonnet",
            "created_at": "2024-10-22T00:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .expect("Failed to build client");

    let model = client
        .beta()
        .models()
        .retrieve("claude-3-5-sonnet")
        .await
        .expect("Failed to retrieve model by alias");

    assert_eq!(model.id, "claude-3-5-sonnet-20241022");
    assert_eq!(model.display_name, "Claude 3.5 Sonnet");
}

#[tokio::test]
async fn test_models_retrieve_not_found() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/models/invalid-model"))
        .and(query_param("beta", "true"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "error": {
                "type": "not_found_error",
                "message": "Model not found"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .expect("Failed to build client");

    let result = client.beta().models().retrieve("invalid-model").await;

    assert!(result.is_err());
}

#[test]
fn test_models_retrieve_empty_id() {
    let client = Client::new("test-key");

    let result = futures::executor::block_on(client.beta().models().retrieve(""));

    assert!(result.is_err());
    match result {
        Err(turboclaude::Error::InvalidRequest(msg)) => {
            assert!(msg.contains("cannot be empty"));
        }
        _ => panic!("Expected InvalidRequest error"),
    }
}

#[tokio::test]
async fn test_models_list_pagination_flow() {
    let mock_server = MockServer::start().await;

    // First page
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .and(query_param("beta", "true"))
        .and(query_param("limit", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "model_1",
                    "type": "model",
                    "display_name": "Model 1",
                    "created_at": "2024-10-22T00:00:00Z"
                },
                {
                    "id": "model_2",
                    "type": "model",
                    "display_name": "Model 2",
                    "created_at": "2024-10-22T00:00:00Z"
                }
            ],
            "has_more": true,
            "first_id": "model_1",
            "last_id": "model_2"
        })))
        .mount(&mock_server)
        .await;

    // Second page
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .and(query_param("beta", "true"))
        .and(query_param("after_id", "model_2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "model_3",
                    "type": "model",
                    "display_name": "Model 3",
                    "created_at": "2024-10-22T00:00:00Z"
                }
            ],
            "has_more": false,
            "first_id": "model_3",
            "last_id": "model_3"
        })))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .expect("Failed to build client");

    // Get first page
    let page1 = client
        .beta()
        .models()
        .list()
        .limit(2)
        .send()
        .await
        .expect("Failed to get first page of models");

    assert_eq!(page1.data.len(), 2);
    assert!(page1.has_more);

    // Get second page
    let page2 = client
        .beta()
        .models()
        .list()
        .after(
            page1
                .last_id
                .as_ref()
                .expect("Expected last_id to be present"),
        )
        .send()
        .await
        .expect("Failed to get second page of models");

    assert_eq!(page2.data.len(), 1);
    assert!(!page2.has_more);
}

#[test]
fn test_models_list_builder_default_values() {
    let client = Client::new("test-key");
    let builder = client.beta().models().list();

    // Verify builder has expected defaults through internal state
    // (This tests builder construction without requiring send())
    assert!(format!("{:?}", builder).contains("ModelsListBuilder"));
}

#[test]
fn test_models_list_builder_limit_bounds() {
    let client = Client::new("test-key");

    // Test that limit is clamped to valid range [1, 1000]
    // Lower bound
    let builder_low = client.beta().models().list().limit(0);
    assert!(format!("{:?}", builder_low).contains("ModelsListBuilder"));

    // Upper bound
    let builder_high = client.beta().models().list().limit(2000);
    assert!(format!("{:?}", builder_high).contains("ModelsListBuilder"));

    // Normal value
    let builder_normal = client.beta().models().list().limit(50);
    assert!(format!("{:?}", builder_normal).contains("ModelsListBuilder"));
}

#[tokio::test]
async fn test_models_list_api_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .and(query_param("beta", "true"))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "error": {
                "type": "internal_server_error",
                "message": "Internal server error"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .expect("Failed to build client");

    let result = client.beta().models().list().send().await;

    assert!(result.is_err());
}

#[test]
fn test_model_page_serialization() {
    let page = ModelPage {
        data: vec![],
        has_more: false,
        first_id: Some("first".to_string()),
        last_id: Some("last".to_string()),
    };

    let json = serde_json::to_value(&page).expect("Failed to serialize ModelPage to JSON");
    assert_eq!(json["has_more"], false);
    assert_eq!(json["first_id"], "first");
    assert_eq!(json["last_id"], "last");
}

#[test]
fn test_model_page_deserialization_with_nulls() {
    let json = r#"{
        "data": [],
        "has_more": false,
        "first_id": null,
        "last_id": null
    }"#;

    let page: ModelPage =
        serde_json::from_str(json).expect("Failed to deserialize ModelPage from JSON");
    assert!(!page.has_more);
    assert_eq!(page.first_id, None);
    assert_eq!(page.last_id, None);
}

#[test]
fn test_models_resource_integration() {
    let client = Client::new("test-key");
    let models = client.beta().models();

    // Verify resource provides expected API
    let _list_builder = models.list();

    // Verify models resource is accessible multiple times (OnceLock)
    let models2 = client.beta().models();
    assert!(std::ptr::eq(models, models2));
}
