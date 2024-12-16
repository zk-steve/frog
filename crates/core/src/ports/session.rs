use async_trait::async_trait;

use crate::entities::session::{SessionEntity, SessionId};
use crate::errors::CoreError;

/// Defines an asynchronous interface for managing session entities.
///
/// This trait abstracts the operations required to create, retrieve,
/// update, and delete sessions. It can be implemented by different storage backends,
/// such as databases or in-memory stores.
#[async_trait]
pub trait SessionPort {
    /// Creates a new session and persists it.
    ///
    /// # Parameters
    /// - `session_entity`: The session entity to be created.
    ///
    /// # Returns
    /// - `Ok(SessionId)`: Returns the unique identifier of the created session.
    /// - `Err(CoreError)`: Returns an error if the creation fails.
    async fn create(&self, session_entity: SessionEntity) -> Result<SessionId, CoreError>;

    /// Retrieves a session by its unique identifier.
    ///
    /// # Parameters
    /// - `session_id`: The unique identifier of the session to retrieve.
    ///
    /// # Returns
    /// - `Ok(SessionEntity)`: Returns the requested session entity.
    /// - `Err(CoreError)`: Returns an error if the session does not exist or retrieval fails.
    async fn get(&self, session_id: SessionId) -> Result<SessionEntity, CoreError>;

    /// Updates an existing session with new data.
    ///
    /// # Parameters
    /// - `session_id`: The unique identifier of the session to update.
    /// - `session_entity`: The updated session entity data.
    ///
    /// # Returns
    /// - `Ok(SessionId)`: Returns the unique identifier of the updated session.
    /// - `Err(CoreError)`: Returns an error if the update fails.
    async fn update(
        &self,
        session_id: SessionId,
        session_entity: SessionEntity,
    ) -> Result<SessionId, CoreError>;

    /// Deletes a session by its unique identifier.
    ///
    /// # Parameters
    /// - `session_id`: The unique identifier of the session to delete.
    ///
    /// # Returns
    /// - `Ok(())`: Indicates the session was successfully deleted.
    /// - `Err(CoreError)`: Returns an error if the deletion fails.
    async fn delete(&self, session_id: SessionId) -> Result<(), CoreError>;
}
