use async_trait::async_trait;

use crate::errors::CoreError;

#[async_trait]
pub trait PeerPort {
    async fn get_dec_share(&self, peer_endpoint: &str) -> Result<Vec<u8>, CoreError>;
}
