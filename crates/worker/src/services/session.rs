use std::sync::Arc;

use frog_core::entities::session::{SessionId, SessionStatus};
use frog_core::ports::session::SessionPort;
use phantom::ops::Ops;
use phantom::utils::fhe_function;
use phantom_zone_evaluator::boolean::fhew::prelude::FheU64;
use phantom_zone_evaluator::boolean::FheBool;

use crate::errors::AppError;

pub struct SessionService {
    session: Arc<dyn SessionPort + Sync + Send>,
}

impl SessionService {
    pub fn new(session: Arc<dyn SessionPort + Sync + Send>) -> Self {
        Self { session }
    }

    pub async fn aggregate_bs_key_shares(&self, session_id: SessionId) -> Result<(), AppError> {
        let mut session_entity = self.session.get(session_id.clone()).await?;
        let bs_key_shares = session_entity
            .client_info
            .values()
            .map(|client| client.bs_key_share.clone())
            .filter(|bs_key| !bs_key.is_empty())
            .collect::<Vec<_>>();

        let phantom_server = session_entity.phantom_server.as_mut().unwrap();
        let mut bs_key_shares = bs_key_shares
            .iter()
            .map(|bytes| phantom_server.deserialize_bs_key_share(bytes).unwrap())
            .collect::<Vec<_>>();
        bs_key_shares.sort_by_key(|bs_key| bs_key.share_idx());
        phantom_server.aggregate_bs_key_shares(&bs_key_shares);
        session_entity.status = SessionStatus::WaitingForArgument;
        self.session.update(session_id, session_entity).await?;
        Ok(())
    }

    pub async fn compute_function(&self, session_id: SessionId) -> Result<(), AppError> {
        let mut session_entity = self.session.get(session_id.clone()).await?;

        let params = session_entity
            .client_info
            .values()
            .map(|client| client.encrypted_data.clone())
            .filter(|data| !data.is_empty())
            .collect::<Vec<_>>();

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
        self.session.update(session_id, session_entity).await?;
        Ok(())
    }
}
