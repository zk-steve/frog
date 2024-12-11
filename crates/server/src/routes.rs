use std::time::Duration;

use axum::error_handling::HandleErrorLayer;
use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{post, put};
use axum::{routing::get, Router};
use tower::{BoxError, ServiceBuilder};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::timeout::TimeoutLayer;

use crate::app_state::AppState;
use crate::controllers::session::{
    add_data, bootstrap_client, create_session, get_session, join_session,
};

pub fn routes(app_state: AppState) -> Router {
    let builder = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|error: BoxError| async move {
            if error.is::<tower::timeout::error::Elapsed>() {
                Ok(StatusCode::REQUEST_TIMEOUT)
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {error}"),
                ))
            }
        }))
        .timeout(Duration::from_secs(60));
    Router::new()
        .route("/", get(root))
        .nest(
            "/v1",
            Router::new()
                .nest(
                    "/sessions",
                    Router::new()
                        .route("/", post(create_session))
                        .route("/{id}", get(get_session).put(join_session))
                        .nest(
                            "/{id}",
                            Router::new().nest(
                                "/clients",
                                Router::new().nest(
                                    "/{client_id}",
                                    Router::new()
                                        .route("/bootstrap", put(bootstrap_client))
                                        .layer(builder)
                                        .route("/data", post(add_data))
                                        .layer(DefaultBodyLimit::disable())
                                        .layer(RequestBodyLimitLayer::new(100 * 1000 * 1000)),
                                ),
                            ),
                        ),
                )
                .with_state(app_state),
        )
        .route("/health", get(root))
        .layer(TimeoutLayer::new(Duration::from_secs(120)))
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
