use async_trait::async_trait;

use crate::entities::session::SessionId;
use crate::errors::CoreError;

#[async_trait]
pub trait WorkerPort {
    async fn aggregate_bs_key_shares(&self, session_id: SessionId) -> Result<(), CoreError>;
    async fn compute_function(&self, session_id: SessionId) -> Result<(), CoreError>;
}
