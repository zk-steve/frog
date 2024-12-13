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
use crate::errors::AppError::SessionError;

pub struct SessionService {
    session: Arc<dyn SessionPort + Sync + Send>,
    phantom_param: Param,
    crs: Crs,
    participant_number: usize,
    worker_port: Arc<dyn WorkerPort + Send + Sync>,

    bootstrap_mutex: Arc<Mutex<bool>>,
    add_data_mutex: Arc<Mutex<bool>>,
}

impl SessionService {
    pub fn new(
        session: Arc<dyn SessionPort + Sync + Send>,
        phantom_param: Param,
        crs: Crs,
        participant_number: usize,
        worker_job: Arc<dyn WorkerPort + Send + Sync>,
    ) -> Self {
        Self {
            session,
            phantom_param,
            crs,
            participant_number,
            worker_port: worker_job,
            bootstrap_mutex: Default::default(),
            add_data_mutex: Default::default(),
        }
    }

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

    pub async fn delete(&self) -> Result<(), AppError> {
        self.session
            .delete(SessionId::try_from("f8e774bd-2f9d-4502-92ca-ac8b9c25868e")?)
            .await?;
        Ok(())
    }

    pub async fn join(
        &self,
        session_id: SessionId,
        client_entity: ClientEntity,
    ) -> Result<(), AppError> {
        let mut session_entity = self.session.get(session_id.clone()).await?;
        if session_entity.client_info.len() == self.participant_number {
            return Err(SessionError(format!(
                "number of clients exceeded {}",
                self.participant_number
            )));
        }

        let phantom_server = session_entity.phantom_server.as_mut().unwrap();

        session_entity
            .client_info
            .insert(client_entity.id.clone(), client_entity.clone());

        if session_entity.client_info.len() == self.participant_number {
            let pk_shares = session_entity
                .client_info
                .values()
                .map(|client_entity| client_entity.pk_share.clone())
                .map(|bytes| phantom_server.deserialize_pk_share(&bytes).unwrap())
                .collect::<Vec<_>>();
            phantom_server.aggregate_pk_shares(&pk_shares);

            phantom_server.aggregate_rp_key_shares(
                &session_entity
                    .client_info
                    .values()
                    .map(|client_entity| client_entity.rp_key_share.clone())
                    .map(|bytes| phantom_server.deserialize_rp_key_share(&bytes).unwrap())
                    .collect::<Vec<_>>(),
            );

            session_entity.pk = phantom_server.serialize_pk()?;
            session_entity.status = SessionStatus::WaitingForBootstrap;
        }

        self.session.update(session_id, session_entity).await?;

        Ok(())
    }

    pub async fn get_session(&self, session_id: SessionId) -> Result<SessionEntity, AppError> {
        let session_entity = self.session.get(session_id).await?;
        Ok(session_entity)
    }

    pub async fn bootstrap(
        &self,
        session_id: SessionId,
        client_id: ClientId,
        bs_key: Vec<u8>,
    ) -> Result<(), AppError> {
        let guard = self.bootstrap_mutex.lock().await;
        let mut session_entity = self.session.get(session_id.clone()).await?;
        let client_entity = session_entity.client_info.get_mut(&client_id).unwrap();
        client_entity.bs_key_share = bs_key;
        let bs_key_shares = session_entity
            .client_info
            .values()
            .map(|client| client.bs_key_share.clone())
            .filter(|bs_key| !bs_key.is_empty())
            .collect::<Vec<_>>();
        self.session
            .update(session_id.clone(), session_entity)
            .await?;

        if bs_key_shares.len() == self.participant_number {
            self.worker_port.aggregate_bs_key_shares(session_id).await?;
        }

        let _ = guard;
        Ok(())
    }

    pub async fn add_data(
        &self,
        session_id: SessionId,
        client_id: ClientId,
        data: Vec<u8>,
    ) -> Result<(), AppError> {
        let guard = self.add_data_mutex.lock().await;
        let mut session_entity = self.session.get(session_id.clone()).await?;
        let client_entity = session_entity.client_info.get_mut(&client_id).unwrap();
        client_entity.encrypted_data = data;

        let params = session_entity
            .client_info
            .values()
            .map(|client| client.encrypted_data.clone())
            .filter(|data| !data.is_empty())
            .collect::<Vec<_>>();

        self.session
            .update(session_id.clone(), session_entity)
            .await?;

        if params.len() == self.participant_number {
            self.worker_port.compute_function(session_id).await?;
        }

        let _ = guard;
        Ok(())
    }
}
