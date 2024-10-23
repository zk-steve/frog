use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{routing::get, Router};

use crate::app_state::AppState;
use crate::controllers::dummy_job::add_job;

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .nest(
            "/v1",
            Router::new()
                .route("/add_job", post(add_job))
                .with_state(app_state),
        )
        .route("/health", get(root))
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
