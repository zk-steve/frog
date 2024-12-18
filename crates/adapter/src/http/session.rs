use async_trait::async_trait;
use frog_core::entities::client::{ClientEntity, ClientId};
use frog_core::entities::session::{SessionEntity, SessionId};
use frog_core::errors::CoreError;
use frog_core::errors::CoreError::UnexpectedResponse;
use frog_core::ports::session_client::SessionClientPort;
use log::error;
use reqwest::{Client, StatusCode};

pub struct SessionClient {
    server_endpoint: String,
    client: Client,
}

impl SessionClient {
    pub fn new(server_endpoint: String, client: Client) -> Self {
        Self {
            server_endpoint,
            client,
        }
    }

    /// Helper function to handle HTTP responses.
    async fn handle_response(&self, response: reqwest::Response) -> Result<String, CoreError> {
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| CoreError::InternalError(e.into()))?;

        if status != StatusCode::OK {
            error!("Unexpected response with status {}: {}", status, body);
            return Err(UnexpectedResponse(body));
        }
        Ok(body)
    }
}

#[async_trait]
impl SessionClientPort for SessionClient {
    async fn join_session(
        &self,
        session_id: SessionId,
        client_entity: ClientEntity,
    ) -> Result<(), CoreError> {
        let response = self
            .client
            .put(format!(
                "{}/v1/sessions/{}",
                &self.server_endpoint, session_id
            ))
            .json(&client_entity)
            .send()
            .await
            .map_err(|e| CoreError::InternalError(e.into()))?;

        self.handle_response(response).await?;
        Ok(())
    }

    async fn get_session(&self, session_id: SessionId) -> Result<SessionEntity, CoreError> {
        let response = self
            .client
            .get(format!(
                "{}/v1/sessions/{}",
                &self.server_endpoint, session_id
            ))
            .send()
            .await
            .map_err(|e| CoreError::InternalError(e.into()))?;
        response
            .json::<SessionEntity>()
            .await
            .map_err(|e| CoreError::InternalError(e.into()))
    }

    async fn bootstrap(
        &self,
        session_id: SessionId,
        client_id: ClientId,
        bs_key: Vec<u8>,
    ) -> Result<(), CoreError> {
        let response = self
            .client
            .put(format!(
                "{}/v1/sessions/{}/clients/{}/bootstrap",
                &self.server_endpoint, session_id, client_id
            ))
            .json(&bs_key)
            .send()
            .await
            .map_err(|e| CoreError::InternalError(e.into()))?;

        self.handle_response(response).await?;
        Ok(())
    }

    async fn send_data(
        &self,
        session_id: SessionId,
        client_id: ClientId,
        data: Vec<u8>,
    ) -> Result<(), CoreError> {
        let response = self
            .client
            .post(format!(
                "{}/v1/sessions/{}/clients/{}/data",
                &self.server_endpoint, session_id, client_id
            ))
            .json(&data)
            .send()
            .await
            .map_err(|e| CoreError::InternalError(e.into()))?;

        self.handle_response(response).await?;
        Ok(())
    }
}
