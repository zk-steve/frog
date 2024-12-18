use std::sync::Arc;

use frog_core::entities::session::{SessionId, SessionStatus};
use frog_core::ports::session::SessionPort;
use phantom::ops::Ops;
use phantom::utils::fhe_function;
use phantom_zone_evaluator::boolean::fhew::prelude::FheU64;
use phantom_zone_evaluator::boolean::FheBool;

use crate::errors::AppError;

/// Service for managing session-related operations.
pub struct SessionService {
    session: Arc<dyn SessionPort + Sync + Send>,
}

impl SessionService {
    /// Creates a new instance of `SessionService`.
    pub fn new(session: Arc<dyn SessionPort + Sync + Send>) -> Self {
        Self { session }
    }

    /// Aggregates bootstrapping key shares for a given session.
    ///
    /// # Arguments
    /// - `session_id`: The ID of the session to update.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    /// - `Err(AppError)` if an error occurs during the process.
    pub async fn aggregate_bs_key_shares(&self, session_id: SessionId) -> Result<(), AppError> {
        // Retrieve the session entity from the session port.
        let mut session_entity = self.session.get(session_id.clone()).await?;

        // Collect non-empty bootstrapping key shares from the client info.
        let bs_key_shares = session_entity
            .client_info
            .values()
            .filter_map(|client| {
                if !client.bs_key_share.is_empty() {
                    Some(client.bs_key_share.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // Ensure the session has a valid Phantom server instance.
        let phantom_server = session_entity
            .phantom_server
            .as_mut()
            .ok_or_else(|| AppError::UnexpectedError("Phantom server is missing".into()))?;

        // Deserialize and sort the key shares by index.
        let mut bs_key_shares = bs_key_shares
            .iter()
            .map(|bytes| {
                phantom_server
                    .deserialize_bs_key_share(bytes)
                    .map_err(|e| AppError::UnexpectedError(e.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        bs_key_shares.sort_by_key(|bs_key| bs_key.share_idx());

        // Aggregate the bootstrapping key shares.
        phantom_server.aggregate_bs_key_shares(&bs_key_shares);

        // Update the session status to indicate readiness for argument.
        session_entity.status = SessionStatus::WaitingForArgument;

        // Save the updated session entity.
        self.session.update(session_id, session_entity).await?;
        Ok(())
    }

    /// Computes the function using encrypted data for a given session.
    ///
    /// # Arguments
    /// - `session_id`: The ID of the session to compute for.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    /// - `Err(AppError)` if an error occurs during the computation.
    pub async fn compute_function(&self, session_id: SessionId) -> Result<(), AppError> {
        // Retrieve the session entity from the session port.
        let mut session_entity = self.session.get(session_id.clone()).await?;

        // Collect non-empty encrypted data from the client info.
        let params = session_entity
            .client_info
            .values()
            .filter_map(|client| {
                if !client.encrypted_data.is_empty() {
                    Some(client.encrypted_data.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // Ensure the session has a valid Phantom server instance.
        let phantom_server = session_entity
            .phantom_server
            .as_mut()
            .ok_or_else(|| AppError::UnexpectedError("Phantom server is missing".into()))?;

        // Deserialize, wrap, and convert the encrypted data into `FheU64`.
        let cts = params
            .iter()
            .map(|bytes| {
                let cts = phantom_server.deserialize_batched_ct(bytes).unwrap();
                let wrapped_cts = phantom_server.wrap_batched_ct(&cts);
                let wrapped_cts: [FheBool<_>; 64] =
                    <[FheBool<_>; 64]>::try_from(wrapped_cts).expect("Incorrect vector length");
                FheU64::new(wrapped_cts)
            })
            .collect::<Vec<_>>();

        // Compute the function on the first two ciphertexts (example use-case).
        let ct_out = fhe_function(
            cts.first()
                .ok_or_else(|| AppError::UnexpectedError("Missing first ciphertext".into()))?,
            cts.get(1)
                .ok_or_else(|| AppError::UnexpectedError("Missing second ciphertext".into()))?,
        );

        // Serialize the computed result back into bytes.
        let ct_out = ct_out
            .cts()
            .into_iter()
            .map(|t| phantom_server.serialize_ct(t).unwrap())
            .collect::<Vec<_>>();

        // Update the session entity with the result and mark it as done.
        session_entity.encrypted_result = ct_out;
        session_entity.status = SessionStatus::Done;

        // Save the updated session entity.
        self.session.update(session_id, session_entity).await?;
        Ok(())
    }
}
