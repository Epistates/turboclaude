//! Common test utilities and helpers

use std::path::Path;

/// Load a response fixture
#[allow(dead_code)]
pub fn load_response_fixture(name: &str) -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(manifest_dir)
        .join("tests")
        .join("fixtures")
        .join("responses")
        .join(format!("{}.json", name));

    std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "Failed to load response fixture '{}' from {:?}: {}",
            name, path, e
        )
    })
}

/// Create a test API key
#[allow(dead_code)]
pub fn test_api_key() -> String {
    "sk-test-key-01234567890123456789012345678901234567890123456789".to_string()
}
