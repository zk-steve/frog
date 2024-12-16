use frog_common::workers::{WorkerPayload, BS_KEY_SHARES_AGGREGATOR_WORKER_IDENTIFIER};
use frog_core::entities::session::SessionId;
use graphile_worker::{IntoTaskHandlerResult, TaskHandler, WorkerContext};
use serde::{Deserialize, Serialize};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::app_state::AppState;

/// Worker responsible for aggregating bootstrapping key shares.
#[derive(Deserialize, Serialize, Debug)]
pub struct BsKeySharesWorker(WorkerPayload<SessionId>);

impl TaskHandler for BsKeySharesWorker {
    /// Unique identifier for the worker, used for task scheduling and execution.
    const IDENTIFIER: &'static str = BS_KEY_SHARES_AGGREGATOR_WORKER_IDENTIFIER;

    /// The main entry point for the worker to process tasks.
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

        state
            .session_service
            .aggregate_bs_key_shares(self.0.data)
            .await
    }
}
