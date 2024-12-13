use frog_core::errors::CoreError;
use thiserror::Error;

// The kinds of errors we can hit in our application.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("core error")]
    CoreError(#[from] CoreError),
    #[error("Unexpected error {0}")]
    UnexpectedError(String),
}
