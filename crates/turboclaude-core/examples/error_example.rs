use std::io;
use turboclaude_core::error_boundary;

/// Custom application error type with multiple variants.
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
enum AppError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("JSON parsing error: {0}")]
    Json(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

// Define error boundaries - these enable automatic conversion with `?` operator

// Boundary 1: Convert std::io::Error to AppError
error_boundary!(io::Error => AppError, |e| {
    AppError::Io(format!("{} (kind: {:?})", e, e.kind()))
});

// Boundary 2: Convert serde_json::Error to AppError
error_boundary!(serde_json::Error => AppError, |e| {
    AppError::Json(format!("at line {}, column {}: {}", e.line(), e.column(), e))
});

// Boundary 3: Convert toml::de::Error to AppError
error_boundary!(toml::de::Error => AppError, |e| {
    AppError::Config(e.to_string())
});

/// Example 1: Simple file reading with automatic error conversion.
#[allow(dead_code)]
fn read_config_file(path: &str) -> Result<String, AppError> {
    // io::Error automatically converts to AppError::Io via our boundary
    let content = std::fs::read_to_string(path)?;
    Ok(content)
}

/// Example 2: JSON parsing with automatic error conversion.
#[allow(dead_code)]
fn parse_json_config(json_str: &str) -> Result<serde_json::Value, AppError> {
    // serde_json::Error automatically converts to AppError::Json
    let config: serde_json::Value = serde_json::from_str(json_str)?;
    Ok(config)
}

/// Example 3: TOML parsing with automatic error conversion.
#[allow(dead_code)]
fn parse_toml_config(toml_str: &str) -> Result<toml::Value, AppError> {
    // toml::de::Error automatically converts to AppError::Config
    let config: toml::Value = toml::from_str(toml_str)?;
    Ok(config)
}

/// Example 4: Chaining multiple operations with different error types.
#[allow(dead_code)]
fn load_and_parse_json(path: &str) -> Result<serde_json::Value, AppError> {
    // First operation: io::Error -> AppError::Io
    let content = std::fs::read_to_string(path)?;

    // Second operation: serde_json::Error -> AppError::Json
    let parsed: serde_json::Value = serde_json::from_str(&content)?;

    // Validation (manual error)
    if !parsed.is_object() {
        return Err(AppError::Validation(
            "Expected JSON object at root".to_string(),
        ));
    }

    Ok(parsed)
}

/// Example 5: Complex operation with multiple error boundaries.
#[allow(dead_code)]
fn process_configuration(json_path: &str, toml_path: &str) -> Result<String, AppError> {
    println!("Processing configuration files...");

    // Load JSON config (io::Error and serde_json::Error auto-convert)
    let json_config = load_and_parse_json(json_path)?;
    println!("JSON config loaded: {}", json_config);

    // Load TOML config (io::Error and toml::de::Error auto-convert)
    let toml_content = std::fs::read_to_string(toml_path)?;
    let toml_config: toml::Value = toml::from_str(&toml_content)?;
    println!("TOML config loaded: {}", toml_config);

    Ok(format!(
        "Successfully processed {} and {}",
        json_path, toml_path
    ))
}

fn main() {
    println!("=== TurboClaude Core: Error Boundary Examples ===\n");

    // Example 1: Reading a non-existent file
    println!("Example 1: File reading error");
    match read_config_file("/nonexistent/file.txt") {
        Ok(_) => println!("  Success (unexpected)"),
        Err(e) => {
            println!("  Error: {}", e);
            println!("  Type: AppError::Io");
        }
    }
    println!();

    // Example 2: Parsing invalid JSON
    println!("Example 2: JSON parsing error");
    let invalid_json = r#"{ "key": invalid }"#;
    match parse_json_config(invalid_json) {
        Ok(_) => println!("  Success (unexpected)"),
        Err(e) => {
            println!("  Error: {}", e);
            println!("  Type: AppError::Json");
        }
    }
    println!();

    // Example 3: Parsing invalid TOML
    println!("Example 3: TOML parsing error");
    let invalid_toml = r#"
[section
key = "missing bracket"
"#;
    match parse_toml_config(invalid_toml) {
        Ok(_) => println!("  Success (unexpected)"),
        Err(e) => {
            println!("  Error: {}", e);
            println!("  Type: AppError::Config");
        }
    }
    println!();

    // Example 4: Valid JSON parsing
    println!("Example 4: Valid JSON parsing");
    let valid_json = r#"{"name": "turboclaude", "version": "1.0.0"}"#;
    match parse_json_config(valid_json) {
        Ok(config) => {
            println!("  Success!");
            println!("  Config: {}", config);
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // Example 5: Valid TOML parsing
    println!("Example 5: Valid TOML parsing");
    let valid_toml = r#"
[package]
name = "turboclaude"
version = "1.0.0"

[dependencies]
serde = "1.0"
"#;
    match parse_toml_config(valid_toml) {
        Ok(config) => {
            println!("  Success!");
            println!("  Config: {}", config);
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // Example 6: Demonstrating error context preservation
    println!("Example 6: Error context preservation");
    let permission_error = io::Error::new(
        io::ErrorKind::PermissionDenied,
        "access denied to /etc/shadow",
    );
    let app_error: AppError = permission_error.into();
    println!("  Original: io::Error (PermissionDenied)");
    println!("  Converted: {}", app_error);
    println!("  Context preserved: error kind and message included");
    println!();

    // Summary
    println!("=== Summary ===");
    println!("Error boundaries eliminate boilerplate map_err() calls.");
    println!("The ? operator now works seamlessly across error types.");
    println!("Error context is preserved during conversion.");
    println!("Multiple boundaries can be defined for the same target error.");
}
