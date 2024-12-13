use frog_common::workers::{Worker, COMPUTE_FUNCTION_WORKER_IDENTIFIER};
use frog_core::entities::session::SessionId;
use graphile_worker::{IntoTaskHandlerResult, TaskHandler, WorkerContext};
use serde::{Deserialize, Serialize};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::app_state::AppState;

#[derive(Deserialize, Serialize, Debug)]
pub struct ComputeFunctionWorker(Worker<SessionId>);

impl TaskHandler for ComputeFunctionWorker {
    const IDENTIFIER: &'static str = COMPUTE_FUNCTION_WORKER_IDENTIFIER;

    async fn run(self, ctx: WorkerContext) -> impl IntoTaskHandlerResult {
        let span = Span::current();
        let parent_cx = opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.extract(&self.0.tracing)
        });

        span.set_parent(parent_cx);

        let state = ctx.extensions().get::<AppState>().unwrap();

        state.session_service.compute_function(self.0.data).await
    }
}
