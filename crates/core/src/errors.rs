use anyhow::Error;
use thiserror::Error;

/// Represents the core error types for the application.
///
/// This enum consolidates various error cases, including external errors (e.g., `uuid::Error`,
/// `std::io::Error`) and custom application errors.
#[derive(Error, Debug)]
pub enum CoreError {
    /// Error when failing to parse an ID.
    ///
    /// This wraps `uuid::Error` for invalid or malformed UUID strings.
    #[error("Failed to parse ID: {0}")]
    ParseIdError(#[from] uuid::Error),

    /// Error indicating that a requested resource was not found.
    ///
    /// This can be used for missing entities or data lookups.
    #[error("Resource not found")]
    NotFound,

    /// Internal application error.
    ///
    /// This wraps `anyhow::Error` for general-purpose errors within the application.
    #[error("Internal error: {0}")]
    InternalError(#[from] Error),

    /// Error indicating an unexpected or invalid response.
    ///
    /// Useful when validating responses from external systems or peers.
    #[error("Unexpected response: {0}")]
    UnexpectedResponse(String),

    /// Error indicating an invalid request.
    ///
    /// Useful when validating requests from users.
    #[error("Validation fails: {0}")]
    ValidationFail(String),

    /// Error originating from worker nodes.
    ///
    /// This wraps `anyhow::Error` for errors reported by worker processes or tasks.
    #[error("Worker error: {0}")]
    WorkerError(Error),
}
