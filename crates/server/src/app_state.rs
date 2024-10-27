use std::sync::Arc;

use graphile_worker::WorkerUtils;

use crate::services::session::SessionService;

#[derive(Clone)]
pub struct AppState {
    pub worker_utils: Arc<WorkerUtils>,
    pub session_service: Arc<SessionService>,
}

impl AppState {
    pub fn new(worker_utils: Arc<WorkerUtils>, session_service: Arc<SessionService>) -> Self {
        Self {
            worker_utils,
            session_service,
        }
    }
}
