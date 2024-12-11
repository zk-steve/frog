use std::fmt::Debug;

use phantom_zone_evaluator::boolean::fhew::prelude::{
    Compact, Elem, FhewBoolKeyCompact, FhewBoolKeyOwned, FhewBoolMpiKeyShareCompact,
    FhewBoolMpiKeyShareOwned, FhewBoolParam, LweDecryptionShare, ModulusOps, RingOps,
    RingPackingKeyCompact, RingPackingKeyOwned, RingPackingKeyShareCompact,
    RingPackingKeyShareOwned, RingPackingParam, RlweDecryptionShareList,
    RlweDecryptionShareListOwned, RlwePublicKey, RlwePublicKeyOwned, SeededRlwePublicKey,
    SeededRlwePublicKeyOwned, StdLweRng,
};
use phantom_zone_evaluator::boolean::fhew::{
    FhewBoolBatchedCiphertext, FhewBoolBatchedCiphertextOwned, FhewBoolCiphertext,
    FhewBoolCiphertextOwned, FhewBoolPackedCiphertext, FhewBoolPackedCiphertextOwned,
};
use rand::SeedableRng;

use crate::param::Param;

pub trait Ops: Debug {
    type Ring: RingOps;
    type EvaluationRing: RingOps<Elem = Elem<Self::Ring>>;
    type KeySwitchMod: ModulusOps;
    type PackingRing: RingOps;

    fn new(param: Param) -> Self;

    fn param(&self) -> &FhewBoolParam;

    fn ring_packing_param(&self) -> &RingPackingParam;

    fn ring(&self) -> &Self::Ring;

    fn mod_ks(&self) -> &Self::KeySwitchMod;

    fn ring_rp(&self) -> &Self::PackingRing;

    fn pk_encrypt(
        &self,
        pk: &RlwePublicKeyOwned<Elem<Self::Ring>>,
        m: bool,
    ) -> FhewBoolCiphertextOwned<Elem<Self::Ring>> {
        FhewBoolCiphertextOwned::pk_encrypt(
            self.param(),
            self.ring(),
            pk,
            m,
            &mut StdLweRng::from_entropy(),
        )
    }

    fn batched_pk_encrypt(
        &self,
        pk: &RlwePublicKeyOwned<Elem<Self::Ring>>,
        ms: impl IntoIterator<Item = bool>,
    ) -> FhewBoolBatchedCiphertextOwned<Elem<Self::Ring>> {
        FhewBoolBatchedCiphertextOwned::pk_encrypt(
            self.param(),
            self.ring(),
            pk,
            ms,
            &mut StdLweRng::from_entropy(),
        )
    }

    fn pack<'a>(
        &self,
        rp_key: &RingPackingKeyOwned<<Self::PackingRing as RingOps>::EvalPrep>,
        cts: impl IntoIterator<Item = &'a FhewBoolCiphertextOwned<Elem<Self::Ring>>>,
    ) -> FhewBoolPackedCiphertextOwned<Elem<Self::PackingRing>>;

    fn aggregate_rp_decryption_shares<'a>(
        &self,
        ct: &FhewBoolPackedCiphertextOwned<Elem<Self::PackingRing>>,
        dec_shares: impl IntoIterator<Item = &'a RlweDecryptionShareListOwned<Elem<Self::PackingRing>>>,
    ) -> Vec<bool> {
        ct.aggregate_decryption_shares(self.ring_rp(), dec_shares)
    }

    fn aggregate_decryption_shares<'a>(
        &self,
        ct: &FhewBoolCiphertextOwned<Elem<Self::Ring>>,
        dec_shares: impl IntoIterator<Item = &'a LweDecryptionShare<Elem<Self::Ring>>>,
    ) -> bool {
        ct.aggregate_decryption_shares(self.ring(), dec_shares)
    }

    fn serialize_pk_share(
        &self,
        pk_share: &SeededRlwePublicKeyOwned<Elem<Self::Ring>>,
    ) -> bincode::Result<Vec<u8>> {
        bincode::serialize(&pk_share.compact(self.ring()))
    }

    fn deserialize_pk_share(
        &self,
        bytes: &[u8],
    ) -> bincode::Result<SeededRlwePublicKeyOwned<Elem<Self::Ring>>> {
        let pk_share_compact: SeededRlwePublicKey<Compact> = bincode::deserialize(bytes)?;
        Ok(pk_share_compact.uncompact(self.ring()))
    }

    fn serialize_rp_key_share(
        &self,
        rp_key_share: &RingPackingKeyShareOwned<Elem<Self::PackingRing>>,
    ) -> bincode::Result<Vec<u8>> {
        bincode::serialize(&rp_key_share.compact(self.ring_rp()))
    }

    fn deserialize_rp_key_share(
        &self,
        bytes: &[u8],
    ) -> bincode::Result<RingPackingKeyShareOwned<Elem<Self::PackingRing>>> {
        let rp_key_share_compact: RingPackingKeyShareCompact = bincode::deserialize(bytes)?;
        Ok(rp_key_share_compact.uncompact(self.ring_rp()))
    }

    fn serialize_bs_key_share(
        &self,
        bs_key_share: &FhewBoolMpiKeyShareOwned<Elem<Self::Ring>, Elem<Self::KeySwitchMod>>,
    ) -> bincode::Result<Vec<u8>> {
        bincode::serialize(&bs_key_share.compact(self.ring(), self.mod_ks()))
    }

    fn deserialize_bs_key_share(
        &self,
        bytes: &[u8],
    ) -> bincode::Result<FhewBoolMpiKeyShareOwned<Elem<Self::Ring>, Elem<Self::KeySwitchMod>>> {
        let bs_key_share_compact: FhewBoolMpiKeyShareCompact = bincode::deserialize(bytes)?;
        Ok(bs_key_share_compact.uncompact(self.ring(), self.mod_ks()))
    }

    fn serialize_pk(&self, pk: &RlwePublicKeyOwned<Elem<Self::Ring>>) -> bincode::Result<Vec<u8>> {
        bincode::serialize(&pk.compact(self.ring()))
    }

    fn deserialize_pk(
        &self,
        bytes: &[u8],
    ) -> bincode::Result<RlwePublicKeyOwned<Elem<Self::Ring>>> {
        let pk_compact: RlwePublicKey<Compact> = bincode::deserialize(bytes)?;
        Ok(pk_compact.uncompact(self.ring()))
    }

    fn serialize_rp_key(
        &self,
        rp_key: &RingPackingKeyOwned<Elem<Self::PackingRing>>,
    ) -> bincode::Result<Vec<u8>> {
        bincode::serialize(&rp_key.compact(self.ring_rp()))
    }

    fn deserialize_rp_key(
        &self,
        bytes: &[u8],
    ) -> bincode::Result<RingPackingKeyOwned<Elem<Self::PackingRing>>> {
        let rp_key_compact: RingPackingKeyCompact = bincode::deserialize(bytes)?;
        Ok(rp_key_compact.uncompact(self.ring_rp()))
    }

    fn serialize_bs_key(
        &self,
        bs_key: &FhewBoolKeyOwned<Elem<Self::EvaluationRing>, Elem<Self::KeySwitchMod>>,
    ) -> bincode::Result<Vec<u8>> {
        bincode::serialize(&bs_key.compact(self.ring(), self.mod_ks()))
    }

    fn deserialize_bs_key(
        &self,
        bytes: &[u8],
    ) -> bincode::Result<FhewBoolKeyOwned<Elem<Self::EvaluationRing>, Elem<Self::KeySwitchMod>>>
    {
        let bs_key_compact: FhewBoolKeyCompact = bincode::deserialize(bytes)?;
        Ok(bs_key_compact.uncompact(self.ring(), self.mod_ks()))
    }

    fn serialize_ct(
        &self,
        ct: &FhewBoolCiphertextOwned<Elem<Self::Ring>>,
    ) -> bincode::Result<Vec<u8>> {
        bincode::serialize(&ct.compact(self.ring()))
    }

    fn deserialize_ct(
        &self,
        bytes: &[u8],
    ) -> bincode::Result<FhewBoolCiphertextOwned<Elem<Self::Ring>>> {
        let ct_compact: FhewBoolCiphertext<Compact> = bincode::deserialize(bytes)?;
        Ok(ct_compact.uncompact(self.ring()))
    }

    fn serialize_batched_ct(
        &self,
        ct: &FhewBoolBatchedCiphertextOwned<Elem<Self::Ring>>,
    ) -> bincode::Result<Vec<u8>> {
        bincode::serialize(&ct.compact(self.ring()))
    }

    fn deserialize_batched_ct(
        &self,
        bytes: &[u8],
    ) -> bincode::Result<FhewBoolBatchedCiphertextOwned<Elem<Self::Ring>>> {
        let ct_compact: FhewBoolBatchedCiphertext<Compact> = bincode::deserialize(bytes)?;
        Ok(ct_compact.uncompact(self.ring()))
    }

    fn serialize_rp_ct(
        &self,
        ct: &FhewBoolPackedCiphertextOwned<Elem<Self::PackingRing>>,
    ) -> bincode::Result<Vec<u8>> {
        bincode::serialize(&ct.compact(self.ring_rp()))
    }

    fn deserialize_rp_ct(
        &self,
        bytes: &[u8],
    ) -> bincode::Result<FhewBoolPackedCiphertextOwned<Elem<Self::PackingRing>>> {
        let ct_compact: FhewBoolPackedCiphertext<Compact> = bincode::deserialize(bytes)?;
        Ok(ct_compact.uncompact(self.ring_rp()))
    }

    fn serialize_dec_shares(
        &self,
        dec_shares: &[LweDecryptionShare<Elem<Self::Ring>>],
    ) -> bincode::Result<Vec<u8>> {
        bincode::serialize(dec_shares)
    }

    fn deserialize_dec_shares(
        &self,
        bytes: &[u8],
    ) -> bincode::Result<Vec<LweDecryptionShare<Elem<Self::Ring>>>> {
        bincode::deserialize(bytes)
    }

    fn serialize_rp_dec_share(
        &self,
        dec_share: &RlweDecryptionShareListOwned<Elem<Self::PackingRing>>,
    ) -> bincode::Result<Vec<u8>> {
        bincode::serialize(&dec_share.compact(self.ring_rp()))
    }

    fn deserialize_rp_dec_share(
        &self,
        bytes: &[u8],
    ) -> bincode::Result<RlweDecryptionShareListOwned<Elem<Self::PackingRing>>> {
        let dec_share_compact: RlweDecryptionShareList<Compact> = bincode::deserialize(bytes)?;
        Ok(dec_share_compact.uncompact(self.ring_rp()))
    }
}
