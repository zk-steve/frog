use std::sync::Arc;

use frog_core::entities::client::{ClientEntity, ClientId};
use frog_core::entities::session::{SessionEntity, SessionId, SessionStatus};
use frog_core::ports::session::SessionPort;
use frog_core::ports::worker::WorkerPort;
use phantom::crs::Crs;
use phantom::ops::Ops;
use phantom::param::Param;
use tokio::sync::Mutex;

use crate::errors::AppError;
use crate::errors::AppError::{SessionError, UnexpectedError};

/// Service for managing session-related operations.
pub struct SessionService {
    /// Session repository interface for persistence operations.
    session: Arc<dyn SessionPort + Sync + Send>,
    /// Phantom protocol parameters.
    phantom_param: Param,
    /// Common reference string (CRS).
    crs: Crs,
    /// Number of participants required for a session.
    participant_number: usize,
    /// Interface to interact with worker tasks.
    worker_port: Arc<dyn WorkerPort + Send + Sync>,

    /// Mutex to ensure safe, synchronized bootstrap operations.
    bootstrap_mutex: Arc<Mutex<bool>>,
    /// Mutex to ensure safe, synchronized data addition operations.
    add_data_mutex: Arc<Mutex<bool>>,
}

impl SessionService {
    /// Constructs a new `SessionService` instance.
    pub fn new(
        session: Arc<dyn SessionPort + Sync + Send>,
        phantom_param: Param,
        crs: Crs,
        participant_number: usize,
        worker_port: Arc<dyn WorkerPort + Send + Sync>,
    ) -> Self {
        Self {
            session,
            phantom_param,
            crs,
            participant_number,
            worker_port,
            bootstrap_mutex: Default::default(),
            add_data_mutex: Default::default(),
        }
    }

    /// Creates a new session in the system.
    pub async fn create(&self) -> Result<(), AppError> {
        self.session
            .create(SessionEntity::new(
                SessionId::try_from("f8e774bd-2f9d-4502-92ca-ac8b9c25868e")?,
                self.phantom_param,
                self.crs,
            ))
            .await?;
        Ok(())
    }

    /// Deletes an existing session.
    pub async fn delete(&self) -> Result<(), AppError> {
        self.session
            .delete(SessionId::try_from("f8e774bd-2f9d-4502-92ca-ac8b9c25868e")?)
            .await?;
        Ok(())
    }

    /// Allows a client to join a session.
    ///
    /// # Arguments
    /// - `session_id`: The ID of the session to join.
    /// - `client_entity`: Information about the client attempting to join.
    ///
    /// # Returns
    /// - An error if the session is full or if any operations fail.
    pub async fn join(
        &self,
        session_id: SessionId,
        client_entity: ClientEntity,
    ) -> Result<(), AppError> {
        let mut session_entity = self.session.get(session_id.clone()).await?;

        // Check if the maximum number of participants is reached.
        if session_entity.client_info.len() >= self.participant_number {
            return Err(SessionError(format!(
                "Number of clients exceeded the limit: {}",
                self.participant_number
            )));
        }

        let phantom_server = session_entity
            .phantom_server
            .as_mut()
            .ok_or_else(|| UnexpectedError("Phantom server not initialized".to_string()))?;

        // Add client information to the session.
        session_entity
            .client_info
            .insert(client_entity.id.clone(), client_entity.clone());

        // If all participants have joined, finalize PK and RP key aggregation.
        if session_entity.client_info.len() == self.participant_number {
            let pk_shares = session_entity
                .client_info
                .values()
                .map(|client| client.pk_share.clone())
                .map(|bytes| phantom_server.deserialize_pk_share(&bytes).unwrap())
                .collect::<Vec<_>>();
            phantom_server.aggregate_pk_shares(&pk_shares);

            let rp_key_shares = session_entity
                .client_info
                .values()
                .map(|client| client.rp_key_share.clone())
                .map(|bytes| phantom_server.deserialize_rp_key_share(&bytes).unwrap())
                .collect::<Vec<_>>();
            phantom_server.aggregate_rp_key_shares(&rp_key_shares);

            session_entity.pk = phantom_server.serialize_pk()?;
            session_entity.status = SessionStatus::WaitingForBootstrap;
        }

        self.session.update(session_id, session_entity).await?;
        Ok(())
    }

    /// Retrieves session details by session ID.
    pub async fn get_session(&self, session_id: SessionId) -> Result<SessionEntity, AppError> {
        let session_entity = self.session.get(session_id).await?;
        Ok(session_entity)
    }

    /// Handles adding bootstrapping key from a client for a session.
    ///
    /// # Arguments
    /// - `session_id`: The ID of the session.
    /// - `client_id`: The ID of the client providing the bootstrap key.
    /// - `bs_key`: The bootstrap key provided by the client.
    pub async fn bootstrap(
        &self,
        session_id: SessionId,
        client_id: ClientId,
        bs_key: Vec<u8>,
    ) -> Result<(), AppError> {
        let _guard = self.bootstrap_mutex.lock().await;

        let mut session_entity = self.session.get(session_id.clone()).await?;
        let client_entity = session_entity
            .client_info
            .get_mut(&client_id)
            .ok_or_else(|| SessionError(format!("Client not found: {}", client_id)))?;

        client_entity.bs_key_share = bs_key;

        self.session
            .update(session_id.clone(), session_entity.clone())
            .await?;

        // Aggregate bootstrap keys if all participants have provided them.
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

        if bs_key_shares.len() == self.participant_number {
            self.worker_port.aggregate_bs_key_shares(session_id).await?;
        }

        Ok(())
    }

    /// Adds encrypted data from a client to the session.
    ///
    /// # Arguments
    /// - `session_id`: The session to which data is being added.
    /// - `client_id`: The ID of the client providing the data.
    /// - `data`: The encrypted data provided by the client.
    pub async fn add_data(
        &self,
        session_id: SessionId,
        client_id: ClientId,
        data: Vec<u8>,
    ) -> Result<(), AppError> {
        let _guard = self.add_data_mutex.lock().await;

        let mut session_entity = self.session.get(session_id.clone()).await?;
        let client_entity = session_entity
            .client_info
            .get_mut(&client_id)
            .ok_or_else(|| SessionError(format!("Client not found: {}", client_id)))?;

        client_entity.encrypted_data = data;

        self.session
            .update(session_id.clone(), session_entity.clone())
            .await?;

        // Trigger worker computation if all participants have provided data.
        let all_data = session_entity
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

        if all_data.len() == self.participant_number {
            self.worker_port.compute_function(session_id).await?;
        }

        Ok(())
    }
}
