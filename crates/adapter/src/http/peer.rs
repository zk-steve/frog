use async_trait::async_trait;
use frog_core::errors::CoreError;
use frog_core::ports::peer::PeerPort;
use reqwest::Client;

pub struct PeerClient {
    client: Client,
}

impl PeerClient {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl PeerPort for PeerClient {
    async fn get_dec_share(&self, peer_endpoint: &str) -> Result<Vec<u8>, CoreError> {
        let response = self
            .client
            .get(format!("{}/decrypt_share", peer_endpoint))
            .send()
            .await
            .map_err(|e| CoreError::InternalError(e.into()))?;

        response
            .json()
            .await
            .map_err(|e| CoreError::ParseResponseError(e.into()))
    }
}