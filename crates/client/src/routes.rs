use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{routing::get, Router};

use crate::app_state::AppState;
use crate::controllers::session::get_decryption_share;

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(root))
        .route("/decrypt_share", get(get_decryption_share))
        .with_state(app_state)
        .fallback(handler_404)
}

async fn root() -> &'static str {
    "Server is running!"
}

async fn handler_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        "The requested resource was not found",
    )
}
