use frog_core::errors::CoreError;
use thiserror::Error;

/// Represents application-level errors that can occur during execution.
#[derive(Error, Debug)]
pub enum AppError {
    /// Error originating from the core application logic.
    #[error("Core error: {0}")]
    CoreError(#[from] CoreError),

    /// A generic unexpected error with a custom message.
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}
