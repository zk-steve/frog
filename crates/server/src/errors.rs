use std::io;

use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use frog_core::errors::CoreError;
use graphile_worker::errors::GraphileWorkerError;
use thiserror::Error;

use crate::json_response::JsonResponse;

/// Application-wide error types.
///
/// This enum represents all the errors that can occur within the application.
/// Each variant corresponds to a specific type of error that may arise during
/// request handling, database operations, or worker interactions.
#[derive(Error, Debug)]
pub enum AppError {
    /// The request body contained invalid JSON.
    #[error("Invalid JSON in the request body: {0}")]
    JsonRejection(JsonRejection),

    /// Input/output operation errors.
    #[error("I/O error occurred: {0}")]
    IOError(#[from] io::Error),

    /// Errors from the core logic of the application.
    #[error("Core error occurred: {0}")]
    CoreError(#[from] CoreError),

    /// Errors related to the Graphile worker.
    #[error("Graphile worker error occurred: {0}")]
    GraphileWorkerError(#[from] GraphileWorkerError),

    /// Session-specific errors, represented as a custom message.
    #[error("Session error: {0}")]
    SessionError(String),

    /// Errors arising from serialization or deserialization via `bincode`.
    #[error("Bincode serialization error occurred: {0}")]
    BincodeError(#[from] bincode::Error),

    /// Error during serialization or deserialization with `bincode`.
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

/// Convert `AppError` into an HTTP response.
///
/// This implementation defines how `AppError` is converted into an HTTP response,
/// allowing it to serve as the central error handler for Axum-based routes. Errors
/// are logged as necessary, and appropriate HTTP status codes and messages are returned.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Internal struct to format error responses as JSON.
        #[derive(serde::Serialize)]
        struct ErrorResponse {
            message: String,
        }

        // Map errors to appropriate HTTP status codes and messages.
        let (status, message) = match self {
            AppError::JsonRejection(rejection) => {
                // Errors caused by invalid JSON input are client errors.
                (rejection.status(), rejection.body_text())
            }
            AppError::GraphileWorkerError(error) => {
                // Graphile worker errors are internal server errors.
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
            }
            AppError::SessionError(error) => {
                // Session-related errors are internal server errors.
                (StatusCode::INTERNAL_SERVER_ERROR, error)
            }
            AppError::CoreError(error) => {
                // Core application logic errors are internal server errors.
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
            }
            e => {
                // Log unexpected errors.
                tracing::error!(%e, "Unhandled application error");

                // Do not expose details of unexpected errors to the client.
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An unexpected error occurred".to_owned(),
                )
            }
        };

        // Convert the error into an HTTP response with a JSON body.
        (status, JsonResponse(ErrorResponse { message })).into_response()
    }
}
