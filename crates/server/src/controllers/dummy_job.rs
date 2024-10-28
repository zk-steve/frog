use std::collections::HashMap;
use std::str;

use axum::body::Bytes;
use axum::extract::State;
use frog_common::workers::dummy_worker::{DummyWorkerData, DUMMY_WORKER_IDENTIFIER};
use tracing::instrument;
use tracing::log::info;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::json_response::JsonResponse;

#[instrument(level = "info", skip(app_state))]
pub async fn add_job(
    State(app_state): State<AppState>,
    body: Bytes,
) -> Result<JsonResponse<()>, AppError> {
    let data = str::from_utf8(&body).unwrap();
    info!("{}", data);

    // retrieve the current span
    let span = tracing::Span::current();
    // retrieve the current context
    let cx = span.context();
    // inject the current context through the amqp headers
    let mut tracing_info = HashMap::new();
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx, &mut tracing_info)
    });

    let dummy = DummyWorkerData {
        internal_id: data.to_string(),
    };

    app_state
        .worker_utils
        .add_raw_job(
            DUMMY_WORKER_IDENTIFIER,
            serde_json::json!({
                "data": dummy,
                "tracing": tracing_info,
            }),
            Default::default(),
        )
        .await?;
    Ok(JsonResponse(()))
}
