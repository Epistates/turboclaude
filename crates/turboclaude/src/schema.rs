//! JSON Schema transformation utilities for structured outputs
//!
//! This module provides utilities for generating and transforming JSON schemas
//! from Rust types to be compatible with Claude's structured outputs API.

#[cfg(feature = "schema")]
use schemars::schema::RootSchema;
#[cfg(feature = "schema")]
use serde_json::{json, Value};

/// Generate a JSON schema compatible with Claude's structured outputs API.
///
/// This function takes a type that implements `JsonSchema` and generates
/// a schema that conforms to the API's expectations.
///
/// # Type Parameters
///
/// * `T` - The type to generate a schema for. Must implement `JsonSchema`.
///
/// # Example
///
/// ```rust,ignore
/// use schemars::JsonSchema;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize, JsonSchema)]
/// struct Order {
///     product_name: String,
///     price: f64,
///     quantity: u32,
/// }
///
/// let schema = generate_schema::<Order>();
/// ```
#[cfg(feature = "schema")]
pub fn generate_schema<T: schemars::JsonSchema>() -> Value {
    let root_schema = schemars::schema_for!(T);
    transform_root_schema(root_schema)
}

/// Transform a root schema to be compatible with Claude's structured outputs API.
///
/// The Claude API has specific requirements for JSON schemas:
/// - Must be a valid JSON Schema Draft 7
/// - Should have clear definitions for all referenced types
/// - Should use simple, clean schema structures
#[cfg(feature = "schema")]
fn transform_root_schema(root: RootSchema) -> Value {
    let mut schema_value = serde_json::to_value(&root).unwrap_or(json!({}));

    // Extract the main schema
    if let Some(obj) = schema_value.as_object_mut() {
        // Clean up schema metadata that Claude doesn't need
        obj.remove("$schema");

        // Ensure definitions are present if needed
        if !root.definitions.is_empty() {
            obj.insert("definitions".to_string(), serde_json::to_value(&root.definitions).unwrap());
        }
    }

    schema_value
}

#[cfg(test)]
#[cfg(feature = "schema")]
mod tests {
    use super::*;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, JsonSchema)]
    struct SimpleType {
        name: String,
        count: u32,
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    struct NestedType {
        simple: SimpleType,
        optional: Option<String>,
    }

    #[test]
    fn test_generate_schema_simple() {
        let schema = generate_schema::<SimpleType>();

        // Should be a valid JSON object
        assert!(schema.is_object());

        // Should have properties
        assert!(schema.get("properties").is_some());

        // Should NOT have $schema (we remove it)
        assert!(schema.get("$schema").is_none());
    }

    #[test]
    fn test_generate_schema_nested() {
        let schema = generate_schema::<NestedType>();

        assert!(schema.is_object());

        // Should have definitions for nested types
        let obj = schema.as_object().unwrap();
        let has_definitions_or_inline =
            obj.contains_key("definitions") ||
            obj.get("properties")
                .and_then(|p| p.get("simple"))
                .is_some();

        assert!(has_definitions_or_inline, "Schema should handle nested types");
    }

    #[test]
    fn test_transform_preserves_structure() {
        let schema = generate_schema::<SimpleType>();
        let obj = schema.as_object().unwrap();

        // Should preserve essential schema fields
        assert!(obj.contains_key("type") || obj.contains_key("properties"),
                "Schema should preserve type information");
    }
}
