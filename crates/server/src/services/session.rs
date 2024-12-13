use std::sync::Arc;

use frog_core::entities::client::{ClientEntity, ClientId};
use frog_core::entities::session::{SessionEntity, SessionId, SessionStatus};
use frog_core::ports::session::SessionPort;
use phantom::crs::Crs;
use phantom::ops::Ops;
use phantom::param::Param;
use phantom::utils::fhe_function;
use phantom_zone_evaluator::boolean::fhew::prelude::FheU64;
use phantom_zone_evaluator::boolean::FheBool;
use tokio::sync::Mutex;

use crate::errors::AppError;
use crate::errors::AppError::SessionError;

pub struct SessionService {
    session: Arc<dyn SessionPort + Sync + Send>,
    phantom_param: Param,
    crs: Crs,
    participant_number: usize,

    mutex: Arc<Mutex<bool>>,
}

impl SessionService {
    pub fn new(
        session: Arc<dyn SessionPort + Sync + Send>,
        phantom_param: Param,
        crs: Crs,
        participant_number: usize,
    ) -> Self {
        Self {
            session,
            phantom_param,
            crs,
            participant_number,
            mutex: Default::default(),
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
        let guard = self.mutex.lock().await;
        let mut session_entity = self.session.get(session_id.clone()).await?;
        let client_entity = session_entity.client_info.get_mut(&client_id).unwrap();
        client_entity.bs_key_share = bs_key;
        let bs_key_shares = session_entity
            .client_info
            .values()
            .map(|client| client.bs_key_share.clone())
            .filter(|bs_key| !bs_key.is_empty())
            .collect::<Vec<_>>();
        if bs_key_shares.len() == self.participant_number {
            let phantom_server = session_entity.phantom_server.as_mut().unwrap();
            let mut bs_key_shares = bs_key_shares
                .iter()
                .map(|bytes| phantom_server.deserialize_bs_key_share(bytes).unwrap())
                .collect::<Vec<_>>();
            bs_key_shares.sort_by_key(|bs_key| bs_key.share_idx());
            phantom_server.aggregate_bs_key_shares(&bs_key_shares);
            session_entity.status = SessionStatus::WaitingForArgument;
        }
        self.session.update(session_id, session_entity).await?;
        let _ = guard;
        Ok(())
    }

    pub async fn add_data(
        &self,
        session_id: SessionId,
        client_id: ClientId,
        data: Vec<u8>,
    ) -> Result<(), AppError> {
        let guard = self.mutex.lock().await;
        let mut session_entity = self.session.get(session_id.clone()).await?;
        let client_entity = session_entity.client_info.get_mut(&client_id).unwrap();
        client_entity.encrypted_data = data;

        let params = session_entity
            .client_info
            .values()
            .map(|client| client.encrypted_data.clone())
            .filter(|data| !data.is_empty())
            .collect::<Vec<_>>();
        if params.len() == self.participant_number {
            let phantom_server = session_entity.phantom_server.as_mut().unwrap();
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
            let ct_out = fhe_function(cts.first().unwrap(), cts.get(1).unwrap());
            let ct_out = ct_out
                .cts()
                .into_iter()
                .map(|t| phantom_server.serialize_ct(t).unwrap())
                .collect::<Vec<_>>();
            session_entity.encrypted_result = ct_out;
            session_entity.status = SessionStatus::Done;
        }
        self.session.update(session_id, session_entity).await?;
        let _ = guard;
        Ok(())
    }
}
