use std::io;

use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use frog_core::errors::CoreError;
use graphile_worker::errors::GraphileWorkerError;
use thiserror::Error;

use crate::json_response::JsonResponse;

// The kinds of errors we can hit in our application.
#[derive(Error, Debug)]
pub enum AppError {
    // The request body contained invalid JSON
    #[error("json error")]
    JsonRejection(JsonRejection),
    #[error("io error")]
    IOError(#[from] io::Error),
    #[error("core error")]
    CoreError(#[from] CoreError),
    #[error("graphile worker error")]
    GraphileWorkerError(#[from] GraphileWorkerError),
    #[error("session error {0}")]
    SessionError(String),
    #[error("bincode error")]
    BincodeError(#[from] bincode::Error),
}

// Tell axum how `AppError` should be converted into a response.
//
// This is also a convenient place to log errors.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // How we want errors responses to be serialized
        #[derive(serde::Serialize)]
        struct ErrorResponse {
            message: String,
        }

        let (status, message) = match self {
            AppError::JsonRejection(rejection) => {
                // This error is caused by bad user input so don't log it
                (rejection.status(), rejection.body_text())
            }
            AppError::GraphileWorkerError(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
            }
            AppError::SessionError(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
            e => {
                // Because `TraceLayer` wraps each request in a span that contains the request
                // method, uri, etc we don't need to include those details here
                tracing::error!(%e, "unknown error");

                // Don't expose any details about the error to the client
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong".to_owned(),
                )
            }
        };
        (status, JsonResponse(ErrorResponse { message })).into_response()
    }
}
