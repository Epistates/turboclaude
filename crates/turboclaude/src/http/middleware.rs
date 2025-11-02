//! HTTP middleware for request/response processing

use super::{RequestBuilder, Response};
use async_trait::async_trait;

/// Trait for HTTP middleware.
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Process a request before sending.
    async fn process_request(
        &self,
        request: RequestBuilder,
    ) -> Result<RequestBuilder, crate::error::Error> {
        Ok(request)
    }

    /// Process a response after receiving.
    async fn process_response(&self, response: Response) -> Result<Response, crate::error::Error> {
        Ok(response)
    }
}

/// Middleware that adds logging/tracing.
pub struct TracingMiddleware;

#[async_trait]
impl Middleware for TracingMiddleware {
    async fn process_request(
        &self,
        request: RequestBuilder,
    ) -> Result<RequestBuilder, crate::error::Error> {
        tracing::debug!("Sending {} request to {}", request.method(), request.url());
        Ok(request)
    }

    async fn process_response(&self, response: Response) -> Result<Response, crate::error::Error> {
        tracing::debug!("Received response with status: {}", response.status());
        Ok(response)
    }
}

/// Middleware that adds rate limiting.
pub struct RateLimitMiddleware {
    governor: std::sync::Arc<governor::DefaultDirectRateLimiter>,
}

impl RateLimitMiddleware {
    /// Create a new rate limit middleware.
    ///
    /// If `requests_per_second` is 0 or negative, defaults to 1 request per second.
    pub fn new(requests_per_second: f64) -> Self {
        use governor::{Quota, RateLimiter};
        use std::num::NonZeroU32;

        // Ensure we have a valid non-zero value
        let rate = if requests_per_second <= 0.0 {
            // SAFETY: NonZeroU32::new(1) is guaranteed to return Some(NonZeroU32(1)) because
            // 1 != 0 by mathematical definition. This is a compile-time invariant that cannot fail.
            NonZeroU32::new(1).expect("1 is non-zero")
        } else {
            NonZeroU32::new(requests_per_second as u32).unwrap_or_else(|| {
                // SAFETY: NonZeroU32::new(1) is guaranteed to return Some(NonZeroU32(1)) because
                // 1 != 0 by mathematical definition. This is a compile-time invariant that cannot fail.
                NonZeroU32::new(1).expect("1 is non-zero")
            })
        };

        let quota = Quota::per_second(rate);

        Self {
            governor: std::sync::Arc::new(RateLimiter::direct(quota)),
        }
    }
}

#[async_trait]
impl Middleware for RateLimitMiddleware {
    async fn process_request(
        &self,
        request: RequestBuilder,
    ) -> Result<RequestBuilder, crate::error::Error> {
        // Wait until we can proceed
        self.governor.until_ready().await;
        Ok(request)
    }
}

/// Composite middleware that chains multiple middleware.
pub struct MiddlewareStack {
    middlewares: Vec<Box<dyn Middleware>>,
}

impl Default for MiddlewareStack {
    fn default() -> Self {
        Self::new()
    }
}

impl MiddlewareStack {
    /// Create a new middleware stack.
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    /// Add a middleware to the stack.
    pub fn push(&mut self, middleware: Box<dyn Middleware>) {
        self.middlewares.push(middleware);
    }
}

#[async_trait]
impl Middleware for MiddlewareStack {
    async fn process_request(
        &self,
        mut request: RequestBuilder,
    ) -> Result<RequestBuilder, crate::error::Error> {
        for middleware in &self.middlewares {
            request = middleware.process_request(request).await?;
        }
        Ok(request)
    }

    async fn process_response(
        &self,
        mut response: Response,
    ) -> Result<Response, crate::error::Error> {
        // Process in reverse order for responses
        for middleware in self.middlewares.iter().rev() {
            response = middleware.process_response(response).await?;
        }
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_middleware_stack_creation() {
        let stack = MiddlewareStack::new();
        assert_eq!(stack.middlewares.len(), 0);

        // Test builder pattern
        let mut stack = MiddlewareStack::default();
        assert_eq!(stack.middlewares.len(), 0);

        stack.push(Box::new(TracingMiddleware));
        assert_eq!(stack.middlewares.len(), 1);
    }

    #[test]
    fn test_middleware_stack_empty() {
        let stack = MiddlewareStack::new();
        assert_eq!(stack.middlewares.len(), 0);
        assert!(stack.middlewares.is_empty());
    }

    #[test]
    fn test_tracing_middleware_enabled() {
        // Just verify the middleware can be created and implements the trait
        let _middleware = TracingMiddleware;

        // TracingMiddleware should always be Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TracingMiddleware>();
    }

    #[test]
    fn test_rate_limit_middleware_token_bucket() {
        // Create middleware with 10 requests per second
        let _middleware = RateLimitMiddleware::new(10.0);

        // Verify it was created (uses Governor token bucket internally)
        // The actual rate limiting is tested in integration tests
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RateLimitMiddleware>();
    }

    #[test]
    fn test_rate_limit_middleware_invalid_rate() {
        // Test with zero rate (should default to 1)
        let _middleware_zero = RateLimitMiddleware::new(0.0);
        // Should not panic, defaults to 1

        // Test with negative rate (should default to 1)
        let _middleware_neg = RateLimitMiddleware::new(-5.0);
        // Should not panic, defaults to 1
    }

    #[test]
    fn test_middleware_ordering() {
        let mut stack = MiddlewareStack::new();

        // Add middlewares in order
        stack.push(Box::new(TracingMiddleware));
        stack.push(Box::new(RateLimitMiddleware::new(10.0)));

        assert_eq!(stack.middlewares.len(), 2);

        // Verify we can add more
        stack.push(Box::new(TracingMiddleware));
        assert_eq!(stack.middlewares.len(), 3);
    }

    #[tokio::test]
    async fn test_middleware_stack_process_request() -> crate::Result<()> {
        use crate::Client;

        let client = Client::new("test-key");
        let request = client.request(http::Method::GET, "/test")?;

        let mut stack = MiddlewareStack::new();
        stack.push(Box::new(TracingMiddleware));

        // Process request through empty stack
        let result = stack.process_request(request).await;
        assert!(result.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_middleware_stack_multiple() -> crate::Result<()> {
        use crate::Client;

        let client = Client::new("test-key");
        let request = client.request(http::Method::POST, "/messages")?;

        let mut stack = MiddlewareStack::new();
        stack.push(Box::new(TracingMiddleware));
        stack.push(Box::new(RateLimitMiddleware::new(50.0)));

        // Process request through stack with multiple middlewares
        let result = stack.process_request(request).await;
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_rate_limit_creation() {
        // Test various valid rates
        let rates = vec![1.0, 10.0, 50.0, 100.0, 1000.0];

        for rate in rates {
            let middleware = RateLimitMiddleware::new(rate);
            // Should create successfully
            drop(middleware);
        }
    }
}
