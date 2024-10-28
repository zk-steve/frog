use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{post, put};
use axum::{routing::get, Router};

use crate::app_state::AppState;
use crate::controllers::dummy_job::add_job;
use crate::controllers::game::{create_game, get_game, join_game, player_move, player_view};

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .nest(
            "/v1",
            Router::new()
                .nest(
                    "/games",
                    Router::new()
                        .route("/", post(create_game))
                        .route("/{id}", get(get_game).put(join_game))
                        .nest(
                            "/{id}",
                            Router::new()
                                .route("/move", put(player_move))
                                .route("/view", get(player_view)),
                        ),
                )
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
