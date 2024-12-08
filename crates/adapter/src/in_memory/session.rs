use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use frog_core::entities::session::{SessionEntity, SessionId};
use frog_core::errors::CoreError;
use frog_core::ports::session::SessionPort;

use crate::in_memory::state::InMemoryState;

pub struct SessionInMemoryRepository {
    pub inner_state: Arc<RwLock<InMemoryState>>,
}

impl SessionInMemoryRepository {
    pub fn new(state: Arc<RwLock<InMemoryState>>) -> Self {
        Self { inner_state: state }
    }
}

#[async_trait]
impl SessionPort for SessionInMemoryRepository {
    async fn create(&self, session_entity: SessionEntity) -> Result<SessionId, CoreError> {
        self.inner_state
            .write()
            .unwrap()
            .sessions
            .insert(session_entity.id.clone(), session_entity.clone());
        Ok(session_entity.id)
    }

    async fn get(&self, session_id: SessionId) -> Result<SessionEntity, CoreError> {
        let result = self
            .inner_state
            .read()
            .unwrap()
            .sessions
            .get(&session_id)
            .ok_or(CoreError::NotFound)?
            .clone();
        Ok(result)
    }

    async fn update(
        &self,
        session_id: SessionId,
        mut session_entity: SessionEntity,
    ) -> Result<SessionId, CoreError> {
        session_entity.id = session_id.clone();
        *(self
            .inner_state
            .write()
            .unwrap()
            .sessions
            .get_mut(&session_id)
            .ok_or(CoreError::NotFound)?) = session_entity.clone();
        Ok(session_id)
    }

    async fn delete(&self, session_id: SessionId) -> Result<SessionId, CoreError> {
        self.inner_state
            .write()
            .unwrap()
            .sessions
            .remove(&session_id)
            .ok_or(CoreError::NotFound)?;
        Ok(session_id)
    }
}
