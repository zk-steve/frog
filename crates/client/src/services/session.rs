use std::sync::Arc;
use std::time::Duration;

use frog_core::entities::client::{ClientEntity, ClientId};
use frog_core::entities::session::{SessionId, SessionStatus};
use frog_core::ports::session_client::SessionClientPort;
use phantom::client::Client;
use phantom::native_ops::NativeOps;
use phantom::ops::Ops;
use phantom::utils::{binary_to_u64, u64_to_binary};
use tokio::sync::RwLock;
use tokio::time;
use tracing::debug;

use crate::errors::AppError;

pub struct SessionService {
    client_id: ClientId,
    session_id: SessionId,
    dec_share: Arc<RwLock<Vec<u8>>>,
    encrypted_result: Arc<RwLock<Vec<Vec<u8>>>>,
    result: Arc<RwLock<Option<u64>>>,

    session_client: Arc<dyn SessionClientPort + Sync + Send>,
    phantom_client: Arc<RwLock<Client<NativeOps>>>,
}

impl SessionService {
    pub fn new(
        client_id: ClientId,
        session_id: SessionId,
        session_client: Arc<dyn SessionClientPort + Sync + Send>,
        phantom_client: Arc<RwLock<Client<NativeOps>>>,
    ) -> Self {
        Self {
            client_id,
            session_id,
            dec_share: Default::default(),
            encrypted_result: Default::default(),
            result: Default::default(),
            session_client,
            phantom_client,
        }
    }

    pub async fn create(&self) -> Result<(), AppError> {
        Ok(())
    }

    pub async fn join(&self) -> Result<(), AppError> {
        let client = self.phantom_client.read().await;

        let pk_share = client.serialize_pk_share(&client.pk_share_gen())?;

        let rp_key_share = client.serialize_rp_key_share(&client.rp_key_share_gen())?;

        self.session_client
            .join_session(
                self.session_id.clone(),
                ClientEntity::new(self.client_id.clone(), pk_share, rp_key_share),
            )
            .await?;
        Ok(())
    }

    pub async fn bootstrap(&self) -> Result<(), AppError> {
        let client = self.phantom_client.read().await;

        let bs_key = client.serialize_bs_key_share(&client.bs_key_share_gen())?;
        self.session_client
            .bootstrap(self.session_id.clone(), self.client_id.clone(), bs_key)
            .await?;
        Ok(())
    }

    pub async fn wait(&self, session_status: SessionStatus) -> Result<(), AppError> {
        loop {
            let session_entity = self
                .session_client
                .get_session(self.session_id.clone())
                .await?;
            debug!("Session status: {:?}", session_entity.status);
            if session_entity.status == session_status {
                break;
            }
            time::sleep(Duration::from_secs(1)).await;
        }

        Ok(())
    }

    pub async fn update_pk(&self) -> Result<(), AppError> {
        let session_entity = self
            .session_client
            .get_session(self.session_id.clone())
            .await?;
        if session_entity.status != SessionStatus::WaitingForBootstrap {
            panic!("TODO:");
        }

        let mut client = self.phantom_client.write().await;
        let pk = client.deserialize_pk(&session_entity.pk)?;
        client.with_pk(pk);
        Ok(())
    }

    pub async fn send_secret_data(&self) -> Result<(), AppError> {
        let data = 6_u64;
        let client = self.phantom_client.read().await;
        let input = u64_to_binary::<64>(data);
        let encrypted_data =
            client.serialize_batched_ct(&client.batched_pk_encrypt(input.into_iter()))?;
        self.session_client
            .send_data(
                self.session_id.clone(),
                self.client_id.clone(),
                encrypted_data,
            )
            .await?;
        Ok(())
    }

    pub async fn fetch_encrypted_result(&self) -> Result<(), AppError> {
        let session_entity = self
            .session_client
            .get_session(self.session_id.clone())
            .await?;
        let client = self.phantom_client.read().await;
        let dec_shares = session_entity
            .encrypted_result
            .iter()
            .map(|ct| client.decrypt_share(&client.deserialize_ct(ct).unwrap()))
            .collect::<Vec<_>>();
        *self.dec_share.write().await = client.serialize_dec_shares(&dec_shares)?;
        *self.encrypted_result.write().await = session_entity.encrypted_result;
        Ok(())
    }

    pub async fn get_share(&self) -> Result<Vec<u8>, AppError> {
        Ok(self.dec_share.read().await.clone())
    }

    pub async fn get_result(&self) -> Result<Option<u64>, AppError> {
        Ok(*self.result.read().await)
    }

    pub async fn decrypt_result(&self, mut dec_shares: Vec<Vec<u8>>) -> Result<u64, AppError> {
        let client = self.phantom_client.read().await;

        let ct_out = &self
            .encrypted_result
            .read()
            .await
            .iter()
            .map(|ct| client.deserialize_ct(ct).unwrap())
            .collect::<Vec<_>>();

        dec_shares.push(self.dec_share.read().await.clone());

        let ct_out_dec_shares = dec_shares
            .iter()
            .map(|bytes| client.deserialize_dec_shares(bytes).unwrap())
            .collect::<Vec<_>>();

        let result = (0..ct_out.len())
            .map(|idx| {
                client.aggregate_decryption_shares(
                    &ct_out[idx],
                    ct_out_dec_shares.iter().map(|dec_shares| &dec_shares[idx]),
                )
            })
            .collect::<Vec<_>>();
        let result = binary_to_u64(result);
        *self.result.write().await = Some(result);
        Ok(result)
    }
}
