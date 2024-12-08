use axum::extract::{Path, State};
use axum::Json;
use frog_core::entities::client::{ClientEntity, ClientId};
use frog_core::entities::session::{SessionEntity, SessionId};
use tracing::instrument;

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::json_response::JsonResponse;

#[instrument(level = "info", skip(app_state))]
pub async fn create_session(
    State(app_state): State<AppState>,
    Json(_input): Json<()>,
) -> Result<JsonResponse<()>, AppError> {
    app_state.session_service.create().await?;
    Ok(JsonResponse(()))
}

#[instrument(level = "info", skip(app_state))]
pub async fn get_session(
    Path(id): Path<String>,
    State(app_state): State<AppState>,
) -> Result<JsonResponse<SessionEntity>, AppError> {
    let session_entity = app_state.session_service.get_session(SessionId(id)).await?;
    Ok(JsonResponse(session_entity))
}

#[instrument(level = "info", skip(app_state, input))]
pub async fn join_session(
    Path(id): Path<String>,
    State(app_state): State<AppState>,
    Json(input): Json<ClientEntity>,
) -> Result<JsonResponse<()>, AppError> {
    app_state.session_service.join(SessionId(id), input).await?;
    Ok(JsonResponse(()))
}

#[instrument(level = "info", skip(app_state, bs_key))]
pub async fn bootstrap_client(
    Path((id, client_id)): Path<(String, usize)>,
    State(app_state): State<AppState>,
    Json(bs_key): Json<Vec<u8>>,
) -> Result<JsonResponse<()>, AppError> {
    app_state
        .session_service
        .bootstrap(SessionId(id), ClientId(client_id), bs_key)
        .await?;
    Ok(JsonResponse(()))
}

#[instrument(level = "info", skip(app_state, data))]
pub async fn add_data(
    Path((id, client_id)): Path<(String, usize)>,
    State(app_state): State<AppState>,
    Json(data): Json<Vec<u8>>,
) -> Result<JsonResponse<()>, AppError> {
    app_state
        .session_service
        .add_data(SessionId(id), ClientId(client_id), data)
        .await?;
    Ok(JsonResponse(()))
}
