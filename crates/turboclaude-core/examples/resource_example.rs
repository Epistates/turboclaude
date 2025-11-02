//! Example: Using the Resource trait for lazy initialization
//!
//! This example shows how to create a resource that is lazily initialized
//! and cached for subsequent accesses.

use async_trait::async_trait;
use turboclaude_core::resource::{LazyResource, Resource};

/// A database connection that is expensive to create
struct DatabaseConnection {
    connection_string: String,
    is_connected: bool,
}

#[derive(Clone)]
struct DbConfig {
    connection_string: String,
}

#[async_trait]
impl Resource for DatabaseConnection {
    type Config = DbConfig;
    type Error = std::io::Error;

    async fn initialize(config: Self::Config) -> Result<Self, Self::Error> {
        println!("Initializing database connection...");
        // Simulate expensive initialization
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        Ok(Self {
            connection_string: config.connection_string,
            is_connected: true,
        })
    }

    async fn is_healthy(&self) -> bool {
        self.is_connected
    }

    async fn cleanup(&mut self) -> Result<(), Self::Error> {
        println!("Closing database connection...");
        self.is_connected = false;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = LazyResource::<DatabaseConnection>::new(DbConfig {
        connection_string: "postgres://localhost/mydb".to_string(),
    });

    println!("Created LazyResource (not initialized yet)");
    println!("Is initialized: {}", db.is_initialized());

    // First access - will initialize
    println!("\nFirst access - initializing...");
    let conn = db.get().await?;
    println!("First access - initialized: {}", conn.connection_string);
    println!("Is healthy: {}", conn.is_healthy().await);

    // Second access - returns cached instance
    println!("\nSecond access - using cached instance...");
    let conn2 = db.get().await?;
    println!("Second access - same instance: {}", conn2.connection_string);

    // Clone the resource - shares the same initialized instance
    println!("\nCloning the resource...");
    let db_clone = db.clone();
    println!("Clone is initialized: {}", db_clone.is_initialized());
    let conn3 = db_clone.get().await?;
    println!("Clone access - same instance: {}", conn3.connection_string);

    // Check health through the wrapper
    println!("\nHealth check through wrapper:");
    println!("Is healthy: {:?}", db.is_healthy().await);

    Ok(())
}
