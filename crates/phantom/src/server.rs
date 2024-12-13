use std::ops::Deref;

use itertools::Itertools;
use phantom_zone_evaluator::boolean::fhew::prelude::{
    aggregate_bs_key_shares, aggregate_pk_shares, aggregate_rp_key_shares, prepare_bs_key,
    prepare_rp_key, Elem, FhewBoolKeyOwned, FhewBoolMpiKeyShareOwned, FhewBoolMpiParam, RingOps,
    RingPackingKeyOwned, RingPackingKeyShareOwned, RlwePublicKey, RlwePublicKeyOwned,
    SeededRlwePublicKeyOwned,
};
use phantom_zone_evaluator::boolean::fhew::{
    FhewBoolBatchedCiphertextOwned, FhewBoolCiphertextOwned, FhewBoolEvaluator,
    FhewBoolPackedCiphertextOwned,
};
use phantom_zone_evaluator::boolean::FheBool;
use serde::{Deserialize, Serialize};

use crate::crs::Crs;
use crate::ops::Ops;
use crate::param::Param;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhantomServer<O: Ops> {
    param: Param,
    crs: Crs,
    ops: O,
    pk: Option<RlwePublicKeyOwned<Elem<O::Ring>>>,
    rp_key: Option<RingPackingKeyOwned<Elem<O::PackingRing>>>,
    rp_key_prep: Option<RingPackingKeyOwned<<O::PackingRing as RingOps>::EvalPrep>>,
    bs_key: Option<FhewBoolKeyOwned<Elem<O::Ring>, Elem<O::KeySwitchMod>>>,
    evaluator: Option<FhewBoolEvaluator<O::EvaluationRing, O::KeySwitchMod>>,
}

impl<O: Ops> Deref for PhantomServer<O> {
    type Target = O;

    fn deref(&self) -> &Self::Target {
        &self.ops
    }
}

impl<O: Ops> PhantomServer<O> {
    pub fn new(
        param: Param,
        crs: Crs,
        pk_bytes: Option<&[u8]>,
        rp_key_bytes: Option<&[u8]>,
        bs_key_bytes: Option<&[u8]>,
    ) -> bincode::Result<Self> {
        let mut server = Self {
            param,
            crs,
            ops: O::new(param),
            pk: None,
            rp_key: None,
            rp_key_prep: None,
            bs_key: None,
            evaluator: None,
        };
        if let Some(pk_bytes) = pk_bytes {
            server.with_pk(server.deserialize_pk(pk_bytes)?);
        }
        if let Some(rp_key_bytes) = rp_key_bytes {
            server.with_rp_key(server.deserialize_rp_key(rp_key_bytes)?);
        }
        if let Some(bs_key_bytes) = bs_key_bytes {
            server.with_bs_key(server.deserialize_bs_key(bs_key_bytes)?);
        }
        Ok(server)
    }

    pub fn param(&self) -> &FhewBoolMpiParam {
        &self.param
    }

    pub fn crs(&self) -> &Crs {
        &self.crs
    }

    pub fn pk(&self) -> &RlwePublicKeyOwned<Elem<O::Ring>> {
        self.pk.as_ref().unwrap()
    }

    pub fn rp_key(&self) -> &RingPackingKeyOwned<Elem<O::PackingRing>> {
        self.rp_key.as_ref().unwrap()
    }

    fn rp_key_prep(&self) -> &RingPackingKeyOwned<<O::PackingRing as RingOps>::EvalPrep> {
        self.rp_key_prep.as_ref().unwrap()
    }

    pub fn evaluator(&self) -> &FhewBoolEvaluator<O::EvaluationRing, O::KeySwitchMod> {
        self.evaluator.as_ref().unwrap()
    }

    pub fn bs_key(&self) -> &FhewBoolKeyOwned<Elem<O::EvaluationRing>, Elem<O::KeySwitchMod>> {
        self.bs_key.as_ref().unwrap()
    }

    pub fn with_pk(&mut self, pk: RlwePublicKeyOwned<Elem<O::Ring>>) {
        self.pk = Some(pk)
    }

    pub fn with_rp_key(&mut self, rp_key: RingPackingKeyOwned<Elem<O::PackingRing>>) {
        let mut rp_key_prep = RingPackingKeyOwned::allocate_eval(
            *self.ring_packing_param(),
            self.ring_rp().eval_size(),
        );
        prepare_rp_key(self.ring_rp(), &mut rp_key_prep, &rp_key);
        self.rp_key = Some(rp_key);
        self.rp_key_prep = Some(rp_key_prep);
    }

    pub fn with_bs_key(
        &mut self,
        bs_key: FhewBoolKeyOwned<Elem<O::EvaluationRing>, Elem<O::KeySwitchMod>>,
    ) {
        let bs_key_prep = {
            let ring: O::EvaluationRing = RingOps::new(self.param.modulus, self.param.ring_size);
            let mut bs_key_prep = FhewBoolKeyOwned::allocate_eval(**self.param, ring.eval_size());
            prepare_bs_key(&ring, &mut bs_key_prep, &bs_key);
            bs_key_prep
        };
        self.bs_key = Some(bs_key);
        self.evaluator = Some(FhewBoolEvaluator::new(bs_key_prep));
    }

    pub fn aggregate_pk_shares(&mut self, pk_shares: &[SeededRlwePublicKeyOwned<Elem<O::Ring>>]) {
        let crs = self.crs.fhew();
        let mut pk = RlwePublicKey::allocate(self.param.ring_size);
        aggregate_pk_shares(self.ring(), &mut pk, &crs, pk_shares);
        self.with_pk(pk);
    }

    pub fn aggregate_rp_key_shares(
        &mut self,
        rp_key_shares: &[RingPackingKeyShareOwned<Elem<O::PackingRing>>],
    ) {
        let crs = self.crs.ring_packing();
        let mut rp_key = RingPackingKeyOwned::allocate(*self.ring_packing_param());
        aggregate_rp_key_shares(self.ring_rp(), &mut rp_key, &crs, rp_key_shares);
        self.with_rp_key(rp_key);
    }

    pub fn aggregate_bs_key_shares(
        &mut self,
        bs_key_shares: &[FhewBoolMpiKeyShareOwned<Elem<O::Ring>, Elem<O::KeySwitchMod>>],
    ) {
        let crs = self.crs.fhew();
        let bs_key = {
            let mut bs_key = FhewBoolKeyOwned::allocate(**self.param);
            aggregate_bs_key_shares(self.ring(), self.mod_ks(), &mut bs_key, &crs, bs_key_shares);
            bs_key
        };
        self.with_bs_key(bs_key);
    }

    pub fn wrap_batched_ct(
        &self,
        ct: &FhewBoolBatchedCiphertextOwned<Elem<O::Ring>>,
    ) -> Vec<FheBool<&FhewBoolEvaluator<O::EvaluationRing, O::KeySwitchMod>>> {
        ct.extract_all(self.ring())
            .into_iter()
            .map(|ct| FheBool::new(self.evaluator(), ct))
            .collect_vec()
    }

    pub fn serialize_pk(&self) -> bincode::Result<Vec<u8>> {
        self.ops.serialize_pk(self.pk())
    }

    pub fn serialize_rp_key(&self) -> bincode::Result<Vec<u8>> {
        self.ops.serialize_rp_key(self.rp_key())
    }

    pub fn serialize_bs_key(&self) -> bincode::Result<Vec<u8>> {
        self.ops.serialize_bs_key(self.bs_key())
    }

    pub fn pack<'a>(
        &self,
        cts: impl IntoIterator<Item = &'a FhewBoolCiphertextOwned<Elem<O::Ring>>>,
    ) -> FhewBoolPackedCiphertextOwned<Elem<O::PackingRing>> {
        self.ops.pack(self.rp_key_prep(), cts)
    }
}
