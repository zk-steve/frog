use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use crate::services::session::SessionService;

#[derive(Clone)]
pub struct AppState {
    pub session_service: Arc<SessionService>,
}

impl Debug for AppState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState").finish()
    }
}

impl AppState {
    pub fn new(session_service: Arc<SessionService>) -> Self {
        Self { session_service }
    }
}
