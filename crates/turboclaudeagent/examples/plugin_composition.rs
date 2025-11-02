//! Plugin Composition and Dependency Resolution Example
//!
//! This example demonstrates how to:
//! - Define plugin manifests with versions and dependencies
//! - Resolve dependency graphs safely
//! - Detect version conflicts and incompatibilities
//! - Determine safe plugin load order
//! - Handle complex multi-plugin scenarios
//!
//! Run with: cargo run --example plugin_composition

use std::collections::HashMap;
use turboclaudeagent::{DependencyResolver, PluginManifest, Version};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Plugin Composition Example ===\n");

    // ============================================================================
    // SCENARIO 1: Simple Linear Dependency Chain
    // ============================================================================
    println!("📦 SCENARIO 1: Linear Dependency Chain");
    println!("   Plugin A (base) -> Plugin B (depends on A) -> Plugin C (depends on B)\n");

    let mut manifests_scenario1 = HashMap::new();

    // Plugin A: No dependencies (base plugin)
    manifests_scenario1.insert(
        "plugin-a".to_string(),
        PluginManifest {
            name: "plugin-a".to_string(),
            version: Version::parse("1.2.0").unwrap(),
            description: Some("Base plugin for core functionality".to_string()),
            requires: HashMap::new(),
            conflicts: vec![],
            priority: 100,
        },
    );

    // Plugin B: Depends on Plugin A
    let mut b_requires = HashMap::new();
    b_requires.insert("plugin-a".to_string(), "^1.0.0".to_string());

    manifests_scenario1.insert(
        "plugin-b".to_string(),
        PluginManifest {
            name: "plugin-b".to_string(),
            version: Version::parse("2.0.0").unwrap(),
            description: Some("Enhanced features built on plugin-a".to_string()),
            requires: b_requires,
            conflicts: vec![],
            priority: 50,
        },
    );

    // Plugin C: Depends on both A and B
    let mut c_requires = HashMap::new();
    c_requires.insert("plugin-a".to_string(), "^1.0.0".to_string());
    c_requires.insert("plugin-b".to_string(), "^2.0.0".to_string());

    manifests_scenario1.insert(
        "plugin-c".to_string(),
        PluginManifest {
            name: "plugin-c".to_string(),
            version: Version::parse("1.5.0").unwrap(),
            description: Some("Advanced features depending on A and B".to_string()),
            requires: c_requires,
            conflicts: vec![],
            priority: 25,
        },
    );

    let resolver1 = DependencyResolver::new(manifests_scenario1);
    match resolver1.resolve(&["plugin-c".to_string()]) {
        Ok(plan) => {
            println!("✅ Resolution successful!");
            println!("Load order: {}", plan.load_order.join(" -> "));
            println!(
                "  (This ensures all dependencies are available before dependent plugins load)\n"
            );
        }
        Err(e) => {
            println!("❌ Resolution failed: {}\n", e);
        }
    }

    // ============================================================================
    // SCENARIO 2: Semantic Version Constraint Matching
    // ============================================================================
    println!("📦 SCENARIO 2: Version Constraint Matching");
    println!("   Testing different version constraint patterns\n");

    let mut manifests_scenario2 = HashMap::new();

    // Plugin with version 1.5.3
    manifests_scenario2.insert(
        "database".to_string(),
        PluginManifest {
            name: "database".to_string(),
            version: Version::parse("1.5.3").unwrap(),
            description: Some("Database abstraction layer v1.5.3".to_string()),
            requires: HashMap::new(),
            conflicts: vec![],
            priority: 100,
        },
    );

    // Test different constraints
    let test_cases = vec![
        ("^1.0.0", true, "Caret: allows up to next major (1.x.x)"),
        ("^2.0.0", false, "Caret: requires major version 2"),
        ("~1.5.0", true, "Tilde: allows same major.minor (1.5.x)"),
        ("~1.4.0", false, "Tilde: requires minor version 4"),
        ("=1.5.3", true, "Exact: must match exactly"),
        ("1.5.3", true, "No prefix: defaults to exact match"),
    ];

    let db_version = Version::parse("1.5.3").unwrap();

    for (constraint, expected, description) in test_cases {
        let matches = db_version.matches(constraint);
        let status = if matches == expected { "✅" } else { "❌" };
        println!(
            "{} {} matches '{}': {}",
            status, description, constraint, matches
        );
    }
    println!();

    // ============================================================================
    // SCENARIO 3: Conflict Detection
    // ============================================================================
    println!("📦 SCENARIO 3: Conflict Detection");
    println!("   Two plugins that cannot coexist\n");

    let mut manifests_scenario3 = HashMap::new();

    // Auth method 1: JWT tokens
    manifests_scenario3.insert(
        "auth-jwt".to_string(),
        PluginManifest {
            name: "auth-jwt".to_string(),
            version: Version::parse("1.0.0").unwrap(),
            description: Some("JWT-based authentication".to_string()),
            requires: HashMap::new(),
            conflicts: vec!["auth-session".to_string()], // Conflicts with session auth
            priority: 100,
        },
    );

    // Auth method 2: Session cookies
    manifests_scenario3.insert(
        "auth-session".to_string(),
        PluginManifest {
            name: "auth-session".to_string(),
            version: Version::parse("2.0.0").unwrap(),
            description: Some("Session cookie authentication".to_string()),
            requires: HashMap::new(),
            conflicts: vec!["auth-jwt".to_string()], // Conflicts with JWT auth
            priority: 90,
        },
    );

    let resolver3 = DependencyResolver::new(manifests_scenario3);

    println!("Attempting to load both auth-jwt and auth-session together:");
    match resolver3.resolve(&["auth-jwt".to_string(), "auth-session".to_string()]) {
        Ok(_) => {
            println!("❌ Should have detected conflict!\n");
        }
        Err(e) => {
            println!("✅ Conflict detected correctly: {}\n", e);
        }
    }

    // ============================================================================
    // SCENARIO 4: Complex Multi-Plugin Ecosystem
    // ============================================================================
    println!("📦 SCENARIO 4: Complex Ecosystem (e-commerce platform)");
    println!("   Realistic scenario with 5+ plugins and cross-dependencies\n");

    let mut manifests_scenario4 = HashMap::new();

    // Core: Database abstraction
    manifests_scenario4.insert(
        "core-db".to_string(),
        PluginManifest {
            name: "core-db".to_string(),
            version: Version::parse("2.0.0").unwrap(),
            description: Some("Database abstraction layer".to_string()),
            requires: HashMap::new(),
            conflicts: vec![],
            priority: 100,
        },
    );

    // Users: Depends on database
    let mut users_requires = HashMap::new();
    users_requires.insert("core-db".to_string(), "^2.0.0".to_string());
    manifests_scenario4.insert(
        "users".to_string(),
        PluginManifest {
            name: "users".to_string(),
            version: Version::parse("1.5.0").unwrap(),
            description: Some("User management system".to_string()),
            requires: users_requires,
            conflicts: vec![],
            priority: 95,
        },
    );

    // Products: Depends on database
    let mut products_requires = HashMap::new();
    products_requires.insert("core-db".to_string(), "^2.0.0".to_string());
    manifests_scenario4.insert(
        "products".to_string(),
        PluginManifest {
            name: "products".to_string(),
            version: Version::parse("1.2.0").unwrap(),
            description: Some("Product catalog system".to_string()),
            requires: products_requires,
            conflicts: vec![],
            priority: 90,
        },
    );

    // Orders: Depends on users, products, and database
    let mut orders_requires = HashMap::new();
    orders_requires.insert("core-db".to_string(), "^2.0.0".to_string());
    orders_requires.insert("users".to_string(), "^1.0.0".to_string());
    orders_requires.insert("products".to_string(), "^1.0.0".to_string());
    manifests_scenario4.insert(
        "orders".to_string(),
        PluginManifest {
            name: "orders".to_string(),
            version: Version::parse("1.0.0").unwrap(),
            description: Some("Order management system".to_string()),
            requires: orders_requires,
            conflicts: vec![],
            priority: 85,
        },
    );

    // Payments: Depends on orders
    let mut payments_requires = HashMap::new();
    payments_requires.insert("orders".to_string(), "^1.0.0".to_string());
    manifests_scenario4.insert(
        "payments".to_string(),
        PluginManifest {
            name: "payments".to_string(),
            version: Version::parse("1.3.0").unwrap(),
            description: Some("Payment processing system".to_string()),
            requires: payments_requires,
            conflicts: vec![],
            priority: 80,
        },
    );

    let resolver4 = DependencyResolver::new(manifests_scenario4);

    println!("Loading complete e-commerce platform with payments plugin...");
    match resolver4.resolve(&["payments".to_string()]) {
        Ok(plan) => {
            println!("✅ Resolution successful!");
            println!("Load order:");
            for (i, plugin) in plan.load_order.iter().enumerate() {
                println!("  {}. {} (loads all dependencies first)", i + 1, plugin);
            }
            println!("\n💡 This load order ensures:");
            println!("   • Database is initialized first (required by all)");
            println!("   • Users and products load before orders (they depend on them)");
            println!("   • Payments loads last (depends on orders)\n");
        }
        Err(e) => {
            println!("❌ Resolution failed: {}\n", e);
        }
    }

    // ============================================================================
    // SCENARIO 5: Version Mismatch Detection
    // ============================================================================
    println!("📦 SCENARIO 5: Version Mismatch Detection");
    println!("   Catching incompatible versions early\n");

    let mut manifests_scenario5 = HashMap::new();

    // Old version of library
    manifests_scenario5.insert(
        "cache-lib".to_string(),
        PluginManifest {
            name: "cache-lib".to_string(),
            version: Version::parse("1.0.0").unwrap(),
            description: Some("Caching library v1.0".to_string()),
            requires: HashMap::new(),
            conflicts: vec![],
            priority: 100,
        },
    );

    // Plugin requires newer version
    let mut cache_plugin_requires = HashMap::new();
    cache_plugin_requires.insert("cache-lib".to_string(), "^2.0.0".to_string());
    manifests_scenario5.insert(
        "cache-plugin".to_string(),
        PluginManifest {
            name: "cache-plugin".to_string(),
            version: Version::parse("1.0.0").unwrap(),
            description: Some("Caching plugin requiring v2.0+".to_string()),
            requires: cache_plugin_requires,
            conflicts: vec![],
            priority: 90,
        },
    );

    let resolver5 = DependencyResolver::new(manifests_scenario5);

    println!("Attempting to use cache-plugin with outdated cache-lib v1.0.0:");
    match resolver5.resolve(&["cache-plugin".to_string()]) {
        Ok(_) => {
            println!("❌ Should have detected version mismatch!\n");
        }
        Err(e) => {
            println!("✅ Version mismatch detected: {}\n", e);
        }
    }

    // ============================================================================
    // SCENARIO 6: Selective Plugin Loading
    // ============================================================================
    println!("📦 SCENARIO 6: Selective Plugin Loading");
    println!("   Loading only required plugins (not all available ones)\n");

    let mut manifests_scenario6 = HashMap::new();

    // Available plugins
    for i in 1..=5 {
        manifests_scenario6.insert(
            format!("optional-feature-{}", i),
            PluginManifest {
                name: format!("optional-feature-{}", i),
                version: Version::parse("1.0.0").unwrap(),
                description: Some(format!("Optional feature {}", i)),
                requires: HashMap::new(),
                conflicts: vec![],
                priority: 50,
            },
        );
    }

    let resolver6 = DependencyResolver::new(manifests_scenario6);

    println!("Available plugins: optional-feature-1 through optional-feature-5");
    println!("User selected: optional-feature-2 and optional-feature-4\n");

    match resolver6.resolve(&[
        "optional-feature-2".to_string(),
        "optional-feature-4".to_string(),
    ]) {
        Ok(plan) => {
            println!("✅ Selective loading successful!");
            println!("Loading: {}", plan.load_order.join(", "));
            println!("  (Other optional plugins are not loaded)\n");
        }
        Err(e) => {
            println!("❌ Failed: {}\n", e);
        }
    }

    // ============================================================================
    // SUMMARY
    // ============================================================================
    println!("=== Key Takeaways ===");
    println!("✅ Version constraints (^, ~, =) enable safe upgrades");
    println!("✅ Transitive dependencies are collected automatically");
    println!("✅ Topological sorting ensures safe load order");
    println!("✅ Conflict detection prevents incompatible plugins");
    println!("✅ Version mismatches are caught before runtime");
    println!("✅ Complex ecosystems can be composed safely\n");

    println!("💡 Use DependencyResolver in production to:");
    println!("   • Validate plugin compatibility before loading");
    println!("   • Determine safe load order for initialization");
    println!("   • Detect dependency issues before they cause crashes");
    println!("   • Enable safe plugin ecosystem composition");

    Ok(())
}
