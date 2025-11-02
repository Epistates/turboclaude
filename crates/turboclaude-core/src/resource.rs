//! Universal resource lifecycle management.
//!
//! The `Resource` trait provides a consistent pattern for lazy initialization,
//! cleanup, and health checking across all SDK resources.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::OnceCell;

/// A resource that can be initialized, queried, and cleaned up.
///
/// # Type Parameters
/// - `Config`: Configuration needed to initialize the resource
/// - `Error`: Error type for initialization failures
///
/// # Examples
///
/// ```rust
/// use turboclaude_core::resource::{Resource, LazyResource};
/// use async_trait::async_trait;
///
/// struct DatabaseConnection {
///     url: String,
/// }
///
/// #[derive(Clone)]
/// struct DbConfig {
///     url: String,
/// }
///
/// #[async_trait]
/// impl Resource for DatabaseConnection {
///     type Config = DbConfig;
///     type Error = std::io::Error;
///
///     async fn initialize(config: Self::Config) -> Result<Self, Self::Error> {
///         Ok(Self { url: config.url })
///     }
///
///     async fn is_healthy(&self) -> bool {
///         true // Check connection
///     }
///
///     async fn cleanup(&mut self) -> Result<(), Self::Error> {
///         Ok(()) // Close connection
///     }
/// }
/// ```
#[async_trait]
pub trait Resource: Send + Sync + Sized {
    /// Configuration required to initialize this resource.
    type Config: Clone + Send + Sync;

    /// Error type for initialization failures.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Initialize the resource with the given configuration.
    ///
    /// This is called once when the resource is first accessed.
    async fn initialize(config: Self::Config) -> Result<Self, Self::Error>;

    /// Check if the resource is healthy and ready to use.
    ///
    /// Default implementation always returns `true`.
    async fn is_healthy(&self) -> bool {
        true
    }

    /// Clean up the resource before it's dropped.
    ///
    /// Default implementation does nothing.
    async fn cleanup(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// A lazily-initialized resource that implements `Resource`.
///
/// This wrapper handles initialization on first access and cleanup on drop.
///
/// # Examples
///
/// ```rust
/// use turboclaude_core::resource::{Resource, LazyResource};
/// use async_trait::async_trait;
///
/// # struct MyResource;
/// # #[derive(Clone)]
/// # struct MyConfig;
/// # #[async_trait]
/// # impl Resource for MyResource {
/// #     type Config = MyConfig;
/// #     type Error = std::io::Error;
/// #     async fn initialize(config: Self::Config) -> Result<Self, Self::Error> {
/// #         Ok(MyResource)
/// #     }
/// # }
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), std::io::Error> {
/// let resource: LazyResource<MyResource> = LazyResource::new(MyConfig);
/// // Resource is not initialized yet
///
/// let instance = resource.get().await?; // Initialized on first access
/// let same_instance = resource.get().await?; // Returns cached instance
/// # Ok(())
/// # }
/// ```
pub struct LazyResource<R: Resource> {
    inner: Arc<OnceCell<R>>,
    config: R::Config,
}

impl<R: Resource> LazyResource<R> {
    /// Create a new lazy resource with the given configuration.
    pub fn new(config: R::Config) -> Self {
        Self {
            inner: Arc::new(OnceCell::new()),
            config,
        }
    }

    /// Get or initialize the resource.
    ///
    /// The first call will initialize the resource. Subsequent calls return
    /// the cached instance.
    pub async fn get(&self) -> Result<&R, R::Error> {
        self.inner
            .get_or_try_init(|| R::initialize(self.config.clone()))
            .await
    }

    /// Check if the resource has been initialized.
    pub fn is_initialized(&self) -> bool {
        self.inner.get().is_some()
    }

    /// Check if the resource is healthy.
    ///
    /// Returns `None` if the resource hasn't been initialized yet.
    pub async fn is_healthy(&self) -> Option<bool> {
        match self.inner.get() {
            Some(resource) => Some(resource.is_healthy().await),
            None => None,
        }
    }
}

impl<R: Resource> Clone for LazyResource<R> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            config: self.config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct TestResource {
        value: i32,
        #[allow(dead_code)]
        init_count: Arc<AtomicU32>,
    }

    #[derive(Clone)]
    struct TestConfig {
        value: i32,
        init_count: Arc<AtomicU32>,
    }

    #[async_trait]
    impl Resource for TestResource {
        type Config = TestConfig;
        type Error = std::io::Error;

        async fn initialize(config: Self::Config) -> Result<Self, Self::Error> {
            config.init_count.fetch_add(1, Ordering::SeqCst);
            Ok(Self {
                value: config.value,
                init_count: config.init_count,
            })
        }

        async fn is_healthy(&self) -> bool {
            self.value > 0
        }

        async fn cleanup(&mut self) -> Result<(), Self::Error> {
            // Simulate cleanup
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_lazy_resource_initialization() {
        let init_count = Arc::new(AtomicU32::new(0));
        let resource = LazyResource::<TestResource>::new(TestConfig {
            value: 42,
            init_count: init_count.clone(),
        });

        assert!(!resource.is_initialized());
        assert_eq!(init_count.load(Ordering::SeqCst), 0);

        let instance = resource.get().await.unwrap();
        assert_eq!(instance.value, 42);
        assert!(resource.is_initialized());
        assert_eq!(init_count.load(Ordering::SeqCst), 1);

        // Second call returns same instance - init count should not increase
        let instance2 = resource.get().await.unwrap();
        assert_eq!(instance2.value, 42);
        assert_eq!(init_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_lazy_resource_cloning() {
        let init_count = Arc::new(AtomicU32::new(0));
        let resource = LazyResource::<TestResource>::new(TestConfig {
            value: 42,
            init_count: init_count.clone(),
        });

        // Clone before initialization
        let cloned = resource.clone();
        assert!(!cloned.is_initialized());

        // Initialize original
        let instance = resource.get().await.unwrap();
        assert_eq!(instance.value, 42);
        assert_eq!(init_count.load(Ordering::SeqCst), 1);

        // Clone shares the same initialized resource
        assert!(cloned.is_initialized());
        let cloned_instance = cloned.get().await.unwrap();
        assert_eq!(cloned_instance.value, 42);
        assert_eq!(init_count.load(Ordering::SeqCst), 1); // Still only initialized once
    }

    #[tokio::test]
    async fn test_resource_health_check() {
        let init_count = Arc::new(AtomicU32::new(0));
        let resource = LazyResource::<TestResource>::new(TestConfig {
            value: 100,
            init_count,
        });

        // Health check on uninitialized resource
        assert_eq!(resource.is_healthy().await, None);

        // Initialize and check health
        let _instance = resource.get().await.unwrap();
        assert_eq!(resource.is_healthy().await, Some(true));
    }

    #[tokio::test]
    async fn test_resource_health_check_unhealthy() {
        let init_count = Arc::new(AtomicU32::new(0));
        let resource = LazyResource::<TestResource>::new(TestConfig {
            value: -1, // Negative value will be unhealthy
            init_count,
        });

        let _instance = resource.get().await.unwrap();
        assert_eq!(resource.is_healthy().await, Some(false));
    }
}
