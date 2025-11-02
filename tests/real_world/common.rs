/// Real-world testing utilities
///
/// These tests use the actual Anthropic Claude API and require:
/// - ANTHROPIC_API_KEY environment variable
/// - `--ignored` flag to run: cargo test --ignored real_world_
///
/// Run with: cargo test --ignored real_world_ -- --nocapture

use std::time::Instant;

/// Test environment configuration
pub struct TestConfig {
    pub api_key: String,
}

impl TestConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| {
                "ANTHROPIC_API_KEY not set. Real-world tests require a valid API key.\n\
                 Run with: ANTHROPIC_API_KEY=your-key cargo test --ignored real_world_"
            })?;

        Ok(Self { api_key })
    }
}

/// Metrics collector for test runs
#[derive(Debug, Default)]
pub struct TestMetrics {
    pub start_time: Option<Instant>,
    pub end_time: Option<Instant>,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_creation_tokens: u32,
    pub cache_read_tokens: u32,
    pub event_count: usize,
}

impl TestMetrics {
    pub fn new() -> Self {
        Self {
            start_time: Some(Instant::now()),
            ..Default::default()
        }
    }

    pub fn finish(&mut self) {
        self.end_time = Some(Instant::now());
    }

    pub fn elapsed(&self) -> std::time::Duration {
        if let (Some(start), Some(end)) = (self.start_time, self.end_time) {
            end - start
        } else if let Some(start) = self.start_time {
            start.elapsed()
        } else {
            std::time::Duration::default()
        }
    }

    pub fn print_summary(&self) {
        println!("\n=== Test Metrics ===");
        println!("Duration: {:?}", self.elapsed());
        if self.input_tokens > 0 {
            println!("Input tokens: {}", self.input_tokens);
        }
        if self.output_tokens > 0 {
            println!("Output tokens: {}", self.output_tokens);
        }
        if self.cache_creation_tokens > 0 {
            println!("Cache creation tokens: {}", self.cache_creation_tokens);
        }
        if self.cache_read_tokens > 0 {
            println!("Cache read tokens: {}", self.cache_read_tokens);
        }
        if self.event_count > 0 {
            println!("Stream events: {}", self.event_count);
        }
        println!("====================\n");
    }
}
