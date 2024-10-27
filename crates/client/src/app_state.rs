use std::sync::Arc;

use crate::services::session::SessionService;

#[derive(Clone)]
pub struct AppState {
    pub session_service: Arc<SessionService>,
}

impl AppState {
    pub fn new(session_service: Arc<SessionService>) -> Self {
        Self { session_service }
    }
}
