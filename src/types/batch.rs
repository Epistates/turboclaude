//! Batch processing types

use serde::{Deserialize, Serialize};

/// A batch of message requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBatch {
    /// Unique identifier for the batch
    pub id: String,

    /// Type of the object (always "message_batch")
    #[serde(rename = "type")]
    pub batch_type: String,

    /// Processing status
    pub processing_status: ProcessingStatus,

    /// Request counts
    pub request_counts: RequestCounts,

    /// When the batch was created
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// When the batch expires
    pub expires_at: chrono::DateTime<chrono::Utc>,

    /// When processing started
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,

    /// When processing ended
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Results URL when complete
    pub results_url: Option<String>,
}

/// Processing status of a batch.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProcessingStatus {
    /// Batch is being processed
    InProgress,
    /// Batch has been cancelled
    Canceling,
    /// Batch processing ended
    Ended,
}

/// Request count statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestCounts {
    /// Total number of requests
    pub total: u32,
    /// Number of processing requests
    pub processing: u32,
    /// Number of succeeded requests
    pub succeeded: u32,
    /// Number of errored requests
    pub errored: u32,
    /// Number of cancelled requests
    pub canceled: u32,
    /// Number of expired requests
    pub expired: u32,
}
