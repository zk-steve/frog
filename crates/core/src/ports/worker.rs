use async_trait::async_trait;

use crate::entities::session::SessionId;
use crate::errors::CoreError;

/// Defines an asynchronous interface for sending commands to worker processes.
///
/// This trait allows for the coordination of tasks such as aggregating bootstrap key shares
/// and triggering computations within a specific session.
#[async_trait]
pub trait WorkerPort {
    /// Sends a command to workers to aggregate bootstrap key shares for the specified session.
    ///
    /// This operation combines bootstrap key shares contributed by clients into an aggregated key
    /// that will be used for further computations.
    ///
    /// # Parameters
    /// - `session_id`: The unique identifier of the session where the aggregation will occur.
    ///
    /// # Returns
    /// - `Ok(())`: If the command to aggregate key shares is successfully sent.
    /// - `Err(CoreError)`: If sending the command fails or the session is invalid.
    async fn aggregate_bs_key_shares(&self, session_id: SessionId) -> Result<(), CoreError>;

    /// Sends a command to workers to compute a specific function for the specified session.
    ///
    /// This operation initiates the computation logic for the session, using the aggregated key
    /// shares and data contributed by clients.
    ///
    /// # Parameters
    /// - `session_id`: The unique identifier of the session where the computation will occur.
    ///
    /// # Returns
    /// - `Ok(())`: If the command to compute the function is successfully sent.
    /// - `Err(CoreError)`: If sending the command fails or the session is invalid.
    async fn compute_function(&self, session_id: SessionId) -> Result<(), CoreError>;
}
