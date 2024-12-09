use async_trait::async_trait;

use crate::entities::client::{ClientEntity, ClientId};
use crate::entities::session::{SessionEntity, SessionId};
use crate::errors::CoreError;

#[async_trait]
pub trait SessionClientPort {
    async fn join_session(
        &self,
        session_id: SessionId,
        client_entity: ClientEntity,
    ) -> Result<(), CoreError>;

    async fn get_session(&self, session_id: SessionId) -> Result<SessionEntity, CoreError>;

    async fn bootstrap(
        &self,
        session_id: SessionId,
        client_id: ClientId,
        bs_key: Vec<u8>,
    ) -> Result<(), CoreError>;

    async fn send_data(
        &self,
        session_id: SessionId,
        client_id: ClientId,
        data: Vec<u8>,
    ) -> Result<(), CoreError>;
}
