//! HTTP middleware for request/response processing

use async_trait::async_trait;
use super::{RequestBuilder, Response};

/// Trait for HTTP middleware.
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Process a request before sending.
    async fn process_request(&self, request: RequestBuilder) -> Result<RequestBuilder, crate::error::Error> {
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
    async fn process_request(&self, request: RequestBuilder) -> Result<RequestBuilder, crate::error::Error> {
        tracing::debug!(
            "Sending {} request to {}",
            request.method(),
            request.url()
        );
        Ok(request)
    }

    async fn process_response(&self, response: Response) -> Result<Response, crate::error::Error> {
        tracing::debug!(
            "Received response with status: {}",
            response.status()
        );
        Ok(response)
    }
}

/// Middleware that adds rate limiting.
pub struct RateLimitMiddleware {
    governor: std::sync::Arc<governor::DefaultDirectRateLimiter>,
}

impl RateLimitMiddleware {
    /// Create a new rate limit middleware.
    pub fn new(requests_per_second: f64) -> Self {
        use governor::{Quota, RateLimiter};
        use std::num::NonZeroU32;

        let quota = Quota::per_second(
            NonZeroU32::new(requests_per_second as u32).unwrap_or(NonZeroU32::new(1).unwrap())
        );

        Self {
            governor: std::sync::Arc::new(RateLimiter::direct(quota)),
        }
    }
}

#[async_trait]
impl Middleware for RateLimitMiddleware {
    async fn process_request(&self, request: RequestBuilder) -> Result<RequestBuilder, crate::error::Error> {
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
    async fn process_request(&self, mut request: RequestBuilder) -> Result<RequestBuilder, crate::error::Error> {
        for middleware in &self.middlewares {
            request = middleware.process_request(request).await?;
        }
        Ok(request)
    }

    async fn process_response(&self, mut response: Response) -> Result<Response, crate::error::Error> {
        // Process in reverse order for responses
        for middleware in self.middlewares.iter().rev() {
            response = middleware.process_response(response).await?;
        }
        Ok(response)
    }
}