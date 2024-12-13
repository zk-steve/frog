use std::ops::Deref;

use phantom_zone_evaluator::boolean::fhew::prelude::{
    bs_key_share_gen, pk_share_gen, rp_key_share_gen, Elem, FhewBoolMpiKeyShareOwned,
    HierarchicalSeedableRng, LweDecryptionShare, LweSecretKey, LweSecretKeyOwned,
    RingPackingKeyShareOwned, RlweDecryptionShareListOwned, RlwePublicKeyOwned, RlweSecretKey,
    RlweSecretKeyOwned, SeededRlwePublicKey, SeededRlwePublicKeyOwned, StdLweRng,
};
use phantom_zone_evaluator::boolean::fhew::{
    FhewBoolBatchedCiphertextOwned, FhewBoolCiphertextOwned, FhewBoolPackedCiphertextOwned,
};
use rand::prelude::StdRng;
use rand::SeedableRng;

use crate::crs::Crs;
use crate::ops::Ops;
use crate::param::Param;

#[derive(Debug)]
pub struct Client<O: Ops> {
    param: Param,
    crs: Crs,
    ops: O,
    share_idx: usize,
    seed: <StdRng as SeedableRng>::Seed,
    pk: Option<RlwePublicKeyOwned<Elem<O::Ring>>>,
}

impl<O: Ops> Deref for Client<O> {
    type Target = O;

    fn deref(&self) -> &Self::Target {
        &self.ops
    }
}

impl<O: Ops> Client<O> {
    pub fn new(
        param: Param,
        crs: Crs,
        share_idx: usize,
        seed: <StdRng as SeedableRng>::Seed,
        pk_bytes: Option<&[u8]>,
    ) -> bincode::Result<Self> {
        let mut client = Self {
            param,
            crs,
            ops: O::new(param),
            share_idx,
            seed,
            pk: None,
        };
        if let Some(pk_bytes) = pk_bytes {
            client.with_pk(client.deserialize_pk(pk_bytes)?);
        }
        Ok(client)
    }

    pub fn seed(&self) -> <StdRng as SeedableRng>::Seed {
        self.seed
    }

    pub fn sk(&self) -> RlweSecretKeyOwned<i64> {
        RlweSecretKey::sample(
            self.param.ring_size,
            self.param.sk_distribution,
            &mut StdRng::from_hierarchical_seed(self.seed, &[0, 0]),
        )
    }

    pub fn sk_ks(&self) -> LweSecretKeyOwned<i64> {
        LweSecretKey::sample(
            self.param.lwe_dimension,
            self.param.lwe_sk_distribution,
            &mut StdRng::from_hierarchical_seed(self.seed, &[0, 1]),
        )
    }

    pub fn pk(&self) -> &RlwePublicKeyOwned<Elem<O::Ring>> {
        self.pk.as_ref().unwrap()
    }

    pub fn pk_share_gen(&self) -> SeededRlwePublicKeyOwned<Elem<O::Ring>> {
        let mut pk = SeededRlwePublicKey::allocate(self.param.ring_size);
        pk_share_gen(
            self.ring(),
            &mut pk,
            &self.param,
            &self.crs.fhew(),
            &self.sk(),
            &mut StdRng::from_hierarchical_seed(self.seed, &[1, 0]),
        );
        pk
    }

    pub fn rp_key_share_gen(&self) -> RingPackingKeyShareOwned<Elem<O::PackingRing>> {
        let mut rp_key = RingPackingKeyShareOwned::allocate(*self.ring_packing_param());
        rp_key_share_gen(
            self.ring_rp(),
            &mut rp_key,
            &self.crs.ring_packing(),
            &self.sk(),
            &mut StdRng::from_hierarchical_seed(self.seed, &[1, 1]),
        );
        rp_key
    }

    pub fn with_pk(&mut self, pk: RlwePublicKeyOwned<Elem<O::Ring>>) {
        self.pk = Some(pk);
    }

    pub fn bs_key_share_gen(
        &self,
    ) -> FhewBoolMpiKeyShareOwned<Elem<O::Ring>, Elem<O::KeySwitchMod>> {
        let mut bs_key_share = FhewBoolMpiKeyShareOwned::allocate(*self.param, self.share_idx);
        bs_key_share_gen(
            self.ring(),
            self.mod_ks(),
            &mut bs_key_share,
            &self.crs.fhew(),
            &self.sk(),
            self.pk(),
            &self.sk_ks(),
            &mut StdRng::from_hierarchical_seed(self.seed, &[1, 1]),
        );
        bs_key_share
    }

    pub fn batched_pk_encrypt(
        &self,
        ms: impl IntoIterator<Item = bool>,
    ) -> FhewBoolBatchedCiphertextOwned<Elem<O::Ring>> {
        self.ops.batched_pk_encrypt(self.pk(), ms)
    }

    pub fn decrypt_share(
        &self,
        ct: &FhewBoolCiphertextOwned<Elem<O::Ring>>,
    ) -> LweDecryptionShare<Elem<O::Ring>> {
        ct.decrypt_share(
            &self.param,
            self.ring(),
            self.sk().as_view(),
            &mut StdLweRng::from_entropy(),
        )
    }

    pub fn rp_decrypt_share(
        &self,
        ct: &FhewBoolPackedCiphertextOwned<Elem<O::PackingRing>>,
    ) -> RlweDecryptionShareListOwned<Elem<O::PackingRing>> {
        ct.decrypt_share(
            &self.param,
            self.ring_rp(),
            self.sk().as_view(),
            &mut StdLweRng::from_entropy(),
        )
    }

    pub fn serialize_pk(&self) -> bincode::Result<Vec<u8>> {
        self.ops.serialize_pk(self.pk())
    }
}
