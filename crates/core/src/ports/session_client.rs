use async_trait::async_trait;

use crate::entities::client::{ClientEntity, ClientId};
use crate::entities::session::{SessionEntity, SessionId};
use crate::errors::CoreError;

/// Defines an asynchronous interface for client interactions within a session on the server.
#[async_trait]
pub trait SessionClientPort {
    /// Allows a client to join an existing session.
    ///
    /// # Parameters
    /// - `session_id`: The unique identifier of the session the client wants to join.
    /// - `client_entity`: The client entity containing the client's information.
    ///
    /// # Returns
    /// - `Ok(())`: If the client successfully joins the session.
    /// - `Err(CoreError)`: If the join operation fails.
    async fn join_session(
        &self,
        session_id: SessionId,
        client_entity: ClientEntity,
    ) -> Result<(), CoreError>;

    /// Retrieves the current state of a session.
    ///
    /// # Parameters
    /// - `session_id`: The unique identifier of the session to retrieve.
    ///
    /// # Returns
    /// - `Ok(SessionEntity)`: Returns the session entity containing its current state and metadata.
    /// - `Err(CoreError)`: If the session does not exist or retrieval fails.
    async fn get_session(&self, session_id: SessionId) -> Result<SessionEntity, CoreError>;

    /// Sends the client's bootstrap key to the server within a session.
    ///
    /// # Parameters
    /// - `session_id`: The unique identifier of the session.
    /// - `client_id`: The unique identifier of the client performing the bootstrap.
    /// - `bs_key`: A vector of bytes representing the client's bootstrap key.
    ///
    /// # Returns
    /// - `Ok(())`: If the bootstrap operation succeeds and the key is stored.
    /// - `Err(CoreError)`: If the bootstrap operation fails (e.g., session not found).
    async fn bootstrap(
        &self,
        session_id: SessionId,
        client_id: ClientId,
        bs_key: Vec<u8>,
    ) -> Result<(), CoreError>;

    /// Allows a client to send encrypted data to the server within a session.
    ///
    /// # Parameters
    /// - `session_id`: The unique identifier of the session to send data to.
    /// - `client_id`: The unique identifier of the client sending the data.
    /// - `data`: A vector of bytes containing the encrypted data.
    ///
    /// # Returns
    /// - `Ok(())`: If the data is successfully sent and stored on the server.
    /// - `Err(CoreError)`: If sending or storing the data fails (e.g., session or client errors).
    async fn send_data(
        &self,
        session_id: SessionId,
        client_id: ClientId,
        data: Vec<u8>,
    ) -> Result<(), CoreError>;
}
