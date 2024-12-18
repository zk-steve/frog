use frog_common::workers::{WorkerPayload, COMPUTE_FUNCTION_WORKER_IDENTIFIER};
use frog_core::entities::session::SessionId;
use graphile_worker::{IntoTaskHandlerResult, TaskHandler, WorkerContext};
use serde::{Deserialize, Serialize};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::app_state::AppState;

/// Worker responsible for executing a computation function on encrypted data.
#[derive(Deserialize, Serialize, Debug)]
pub struct ComputeFunctionWorker(WorkerPayload<SessionId>);

impl TaskHandler for ComputeFunctionWorker {
    /// Unique identifier for the worker, used for task scheduling and execution.
    const IDENTIFIER: &'static str = COMPUTE_FUNCTION_WORKER_IDENTIFIER;

    /// Executes the task to compute a function on encrypted data.
    ///
    /// # Arguments
    /// - `ctx`: The worker context, providing access to extensions and other runtime parameters.
    ///
    /// # Returns
    /// - `impl IntoTaskHandlerResult`: Result of the task execution, adhering to the worker interface.
    async fn run(self, ctx: WorkerContext) -> impl IntoTaskHandlerResult {
        // Extract the current tracing span to ensure tracing context is carried across task boundaries.
        let span = Span::current();
        let parent_cx = opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.extract(&self.0.tracing)
        });

        // Set the parent span for proper telemetry tracking.
        span.set_parent(parent_cx);

        // Access shared application state via the worker's context.
        let state = ctx
            .extensions()
            .get::<AppState>()
            .expect("AppState must be added to the worker context");

        // Delegate the task to the session service and handle errors if any.
        state.session_service.compute_function(self.0.data).await
    }
}
