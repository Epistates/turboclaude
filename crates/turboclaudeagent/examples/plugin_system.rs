//! Example demonstrating the plugin system
//!
//! This example shows how to:
//! 1. Configure and validate plugin paths
//! 2. Load plugins from the filesystem
//! 3. Access plugin metadata and commands
//! 4. Work with multiple plugins

use turboclaudeagent::plugins::{PluginLoader, PluginMetadata, SdkPluginConfig};

fn main() {
    println!("=== TurboClaude Plugin System ===\n");

    // 1. Creating plugin configurations
    println!("1. Creating plugin configurations:");
    let plugin1_config = SdkPluginConfig::local("./plugins/demo-plugin");
    let plugin2_config = SdkPluginConfig::local("/opt/plugins/advanced");

    println!(
        "✓ Plugin 1: {} ({})",
        plugin1_config.path, plugin1_config.plugin_type
    );
    println!(
        "✓ Plugin 2: {} ({})\n",
        plugin2_config.path, plugin2_config.plugin_type
    );

    // 2. Validating plugin configurations
    println!("2. Validating plugin configurations:");
    match plugin1_config.validate() {
        Ok(_) => println!("✓ Plugin 1 configuration is valid"),
        Err(e) => println!("✗ Plugin 1 validation failed: {}", e),
    }

    match plugin2_config.validate() {
        Ok(_) => println!("✓ Plugin 2 configuration is valid"),
        Err(e) => println!("✗ Plugin 2 validation failed: {}", e),
    }
    println!();

    // 3. Creating and using a plugin loader
    println!("3. Creating and using plugin loaders:");
    let loader1 = PluginLoader::new(plugin1_config.clone());
    println!("✓ Created loader for plugin: {}", loader1.config().path);

    match loader1.load() {
        Ok(plugin) => {
            println!("✓ Successfully loaded plugin: {}", plugin.metadata.name);
            println!("  - Path: {:?}", plugin.path);
            println!("  - Commands: {:?}", plugin.commands);
        }
        Err(e) => {
            println!("ℹ Plugin not found (expected in example): {}", e);
        }
    }
    println!();

    // 4. Plugin metadata
    println!("4. Plugin metadata structure:");
    let metadata = PluginMetadata {
        name: "example-plugin".to_string(),
        description: Some("An example plugin for demonstration".to_string()),
        version: Some("1.0.0".to_string()),
        author: Some("TurboClaude Team".to_string()),
    };
    println!("✓ Plugin name: {}", metadata.name);
    println!("✓ Description: {:?}", metadata.description);
    println!("✓ Version: {:?}", metadata.version);
    println!("✓ Author: {:?}", metadata.author);
    println!();

    // 5. Serialization examples
    println!("5. Plugin configuration serialization:");
    let config = SdkPluginConfig::local("./my-plugin");
    match serde_json::to_string_pretty(&config) {
        Ok(json) => {
            println!("✓ Configuration as JSON:");
            println!("{}", json);
        }
        Err(e) => println!("✗ Serialization failed: {}", e),
    }
    println!();

    // 6. Plugin discovery patterns
    println!("6. Plugin discovery patterns:");
    println!();
    println!("   Pattern 1: Load from local directory");
    println!("   let config = SdkPluginConfig::local(\"./plugins/my-plugin\");");
    println!();

    println!("   Pattern 2: Load from absolute path");
    println!("   let config = SdkPluginConfig::local(\"/opt/plugins/advanced\");");
    println!();

    println!("   Pattern 3: Create loader and load");
    println!("   let loader = PluginLoader::new(config);");
    println!("   let plugin = loader.load()?;");
    println!();

    println!("   Pattern 4: Access plugin information");
    println!("   println!(\"Name: {{}}\", plugin.metadata.name);");
    println!("   println!(\"Commands: {{:?}}\", plugin.commands);");
    println!();

    // 7. Expected plugin directory structure
    println!("7. Expected plugin directory structure:");
    println!();
    println!("   my-plugin/");
    println!("   ├── .claude-plugin/");
    println!("   │   └── plugin.json         # Plugin metadata");
    println!("   ├── commands/");
    println!("   │   ├── greet.md            # Command markdown files");
    println!("   │   ├── help.md");
    println!("   │   └── status.md");
    println!("   └── assets/                 # Optional: resources");
    println!("       └── config.json");
    println!();

    // 8. Example plugin.json structure
    println!("8. Example .claude-plugin/plugin.json:");
    println!();
    let example_metadata = serde_json::json!({
        "name": "demo-plugin",
        "description": "A demonstration plugin",
        "version": "1.0.0",
        "author": "Your Organization"
    });
    if let Ok(json) = serde_json::to_string_pretty(&example_metadata) {
        println!("{}", json);
    }
    println!();

    // 9. Multiple plugin scenarios
    println!("9. Managing multiple plugins:");
    let configs = vec![
        SdkPluginConfig::local("./plugins/basic"),
        SdkPluginConfig::local("./plugins/advanced"),
        SdkPluginConfig::local("./plugins/utils"),
    ];

    println!("✓ Registered {} plugins:", configs.len());
    for (i, config) in configs.iter().enumerate() {
        println!("  {}. {} ({})", i + 1, config.path, config.plugin_type);
    }
    println!();

    println!("=== Plugin System Example Complete ===");
}
