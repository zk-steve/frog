use async_trait::async_trait;

use crate::entities::session::{SessionEntity, SessionId};
use crate::errors::CoreError;

#[async_trait]
pub trait SessionPort {
    async fn create(&self, session_entity: SessionEntity) -> Result<SessionId, CoreError>;
    async fn get(&self, session_id: SessionId) -> Result<SessionEntity, CoreError>;
    async fn update(
        &self,
        session_id: SessionId,
        session_entity: SessionEntity,
    ) -> Result<SessionId, CoreError>;
    async fn delete(&self, session_id: SessionId) -> Result<(), CoreError>;
}
