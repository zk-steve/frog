use std::io;

use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use frog_core::errors::CoreError;
use thiserror::Error;

use crate::json_response::JsonResponse;

/// Represents the various errors that can occur in the application.
#[derive(Error, Debug)]
pub enum AppError {
    /// Error caused by invalid JSON in the request body.
    #[error("JSON error: {0}")]
    JsonRejection(JsonRejection),

    /// Error related to I/O operations.
    #[error("I/O error: {0}")]
    IOError(#[from] io::Error),

    /// Error originating from the application's core logic.
    #[error("Core error: {0}")]
    CoreError(#[from] CoreError),

    /// Error during serialization or deserialization with `bincode`.
    #[error("Bincode error: {0}")]
    BincodeError(#[from] bincode::Error),
}

/// Implements `IntoResponse` to convert `AppError` into an HTTP response.
///
/// This implementation centralizes error handling, providing:
/// - Detailed responses for specific error types.
/// - Logging for unhandled errors.
/// - Consistent JSON error response format for all errors.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        /// Structure for serializing error responses.
        #[derive(serde::Serialize)]
        struct ErrorResponse {
            message: String,
        }

        // Determine the HTTP status code and error message based on the error type.
        let (status, message) = match self {
            // Handle errors caused by invalid JSON in the request body.
            AppError::JsonRejection(rejection) => {
                // These errors are caused by client input, so we don't log them as server errors.
                (rejection.status(), rejection.body_text())
            }

            // Handle errors originating from the application's core logic.
            AppError::CoreError(error) => match error {
                CoreError::ParseIdError(_) | CoreError::ValidationFail(_) => {
                    // Client errors such as invalid input or validation failure.
                    (StatusCode::BAD_REQUEST, error.to_string())
                }
                CoreError::NotFound => {
                    // Specific case where the requested resource is not found.
                    (StatusCode::NOT_FOUND, error.to_string())
                }
                _ => {
                    // For all other core errors, return a 500 Internal Server Error.
                    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
                }
            },

            // Handle all other error types generically.
            error => {
                // Log unexpected errors for debugging and monitoring purposes.
                tracing::error!(%error, "Unhandled application error");

                // Return a generic error message to the client to avoid exposing sensitive details.
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong".to_string(),
                )
            }
        };

        // Serialize the error message into a JSON response.
        (status, JsonResponse(ErrorResponse { message })).into_response()
    }
}
