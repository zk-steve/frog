use std::sync::Arc;

use graphile_worker::WorkerUtils;

/// Router for handling HTTP requests related to questions.
#[derive(Clone)]
pub struct AppState {
    pub worker_utils: Arc<WorkerUtils>,
}

impl AppState {
    pub fn new(worker_utils: Arc<WorkerUtils>) -> Self {
        AppState { worker_utils }
    }
}
