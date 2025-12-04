//! Multi-Claude Provider (MCP) Basic Example
//!
//! NOTE: This example is currently disabled as the MCP provider routing
//! functionality is not yet implemented. The providers module contains
//! bedrock, vertex, and foundry - but not a multi-provider routing layer.
//!
//! ## Future Implementation
//!
//! When implemented, this will demonstrate how to use the `McpHttpProvider` to create a
//! client that can route requests to multiple Claude providers (Anthropic,
//! Bedrock, Vertex) based on model availability and priority.
//!
//! ## Prerequisites
//!
//! 1. **Anthropic API Key**:
//!    - Environment variable: `ANTHROPIC_API_KEY`
//!
//! 2. **AWS Credentials** (for Bedrock):
//!    - Environment variables: `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`
//!    - AWS credentials file: `~/.aws/credentials`
//!    - IAM role (when running on AWS services)
//!
//! 3. **GCP Credentials** (for Vertex):
//!    - Environment variable: `GOOGLE_APPLICATION_CREDENTIALS` pointing to your
//!      service account JSON key file.
//!    - Or run `gcloud auth application-default login`

fn main() {
    println!("This example is not yet implemented.");
    println!("MCP (Multi-provider routing) will be available in a future release.");
}
