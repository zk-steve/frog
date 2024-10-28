use axum::extract::{Path, State};
use axum::Json;
use tracing::instrument;

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::json_response::JsonResponse;

/// create_game act as a reset game function as of now
#[instrument(level = "info", skip(app_state))]
pub async fn create_game(
    State(app_state): State<AppState>,
    Json(_input): Json<()>,
) -> Result<JsonResponse<()>, AppError> {
    app_state.game_service.create().await?;
    Ok(JsonResponse(()))
}

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
    Path(id): Path<String>,
    State(app_state): State<AppState>,
    Json(input): Json<()>,
) -> Result<JsonResponse<()>, AppError> {
    Ok(JsonResponse(()))
}

#[instrument(level = "info", skip(app_state))]
pub async fn player_view(
    Path(id): Path<String>,
    State(app_state): State<AppState>,
    Json(input): Json<()>,
) -> Result<JsonResponse<()>, AppError> {
    Ok(JsonResponse(()))
}
