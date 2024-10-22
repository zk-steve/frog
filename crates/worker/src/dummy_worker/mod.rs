use std::time::Duration;

use graphile_worker::{IntoTaskHandlerResult, TaskHandler, WorkerContext};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use tracing::info;

#[derive(Deserialize, Serialize)]
pub struct DummyWorker;

impl TaskHandler for DummyWorker {
    const IDENTIFIER: &'static str = "show_run_count";

    async fn run(self, _ctx: WorkerContext) -> impl IntoTaskHandlerResult {
        sleep(Duration::from_secs(5)).await;
        info!("Run count: 1");
    }
}
