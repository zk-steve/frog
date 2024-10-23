use std::collections::HashMap;
use std::time::Duration;

use common::workers::dummy_worker::{DummyWorkerData, DUMMY_WORKER_IDENTIFIER};
use graphile_worker::{IntoTaskHandlerResult, TaskHandler, WorkerContext};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use tracing::{info, instrument, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

#[derive(Deserialize, Serialize)]
pub struct DummyWorker {
    pub data: DummyWorkerData,
    pub tracing: HashMap<String, String>,
}

impl TaskHandler for DummyWorker {
    const IDENTIFIER: &'static str = DUMMY_WORKER_IDENTIFIER;

    #[instrument(level = "info", skip(self, _ctx))]
    async fn run(self, _ctx: WorkerContext) -> impl IntoTaskHandlerResult {
        let span = Span::current();
        let parent_cx = opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.extract(&self.tracing)
        });

        span.set_parent(parent_cx);

        sleep(Duration::from_secs(5)).await;
        info!("data: {:?}", self.data.internal_id);
    }
}
