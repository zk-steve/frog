use axum::extract::{Path, State};
use axum::Json;
use tracing::instrument;

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::json_response::JsonResponse;

#[instrument(level = "info", skip(app_state))]
pub async fn get_game(
    Path(id): Path<String>,
    State(app_state): State<AppState>,
) -> Result<JsonResponse<()>, AppError> {
    Ok(JsonResponse(()))
}

#[instrument(level = "info", skip(app_state))]
pub async fn join_game(
    Path(id): Path<String>,
    State(app_state): State<AppState>,
    Json(input): Json<()>,
) -> Result<JsonResponse<()>, AppError> {
    Ok(JsonResponse(()))
}

#[instrument(level = "info", skip(app_state))]
pub async fn player_move(
    State(app_state): State<AppState>,
    Json(input): Json<()>,
) -> Result<JsonResponse<()>, AppError> {
    Ok(JsonResponse(()))
}

#[instrument(level = "info", skip(app_state))]
pub async fn player_view(State(app_state): State<AppState>) -> Result<JsonResponse<()>, AppError> {
    Ok(JsonResponse(()))
}
