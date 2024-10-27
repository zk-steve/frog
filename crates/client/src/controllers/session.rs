use axum::extract::State;
use tracing::instrument;

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::json_response::JsonResponse;

#[instrument(level = "info", skip(app_state))]
pub async fn get_decryption_share(
    State(app_state): State<AppState>,
) -> Result<JsonResponse<Vec<u8>>, AppError> {
    let result = app_state.session_service.get_share().await?;
    Ok(JsonResponse(result))
}
