use std::collections::HashMap;

use frog_core::entities::session::{SessionEntity, SessionId};

#[derive(Default)]
pub struct InMemoryState {
    pub sessions: HashMap<SessionId, SessionEntity>,
}
