use phantom_zone_evaluator::boolean::fhew::prelude::{
    Elem, FhewBoolParam, ModulusOps, NoisyPrimeRing, NonNativePowerOfTwo, PrimeRing, RingOps,
    RingPackingKeyOwned, RingPackingParam,
};
use phantom_zone_evaluator::boolean::fhew::{
    FhewBoolCiphertextOwned, FhewBoolPackedCiphertext, FhewBoolPackedCiphertextOwned,
};

use crate::ops::Ops;
use crate::param::Param;

#[derive(Debug)]
pub struct PrimeOps {
    param: FhewBoolParam,
    ring_packing_param: RingPackingParam,
    ring: PrimeRing,
    mod_ks: NonNativePowerOfTwo,
}

impl Ops for PrimeOps {
    type Ring = PrimeRing;
    type EvaluationRing = NoisyPrimeRing;
    type KeySwitchMod = NonNativePowerOfTwo;
    type PackingRing = PrimeRing;

    fn new(param: Param) -> Self {
        let ring_packing_param = RingPackingParam {
            modulus: param.modulus,
            ring_size: param.ring_size,
            sk_distribution: param.sk_distribution,
            noise_distribution: param.noise_distribution,
            auto_decomposition_param: param.ring_packing_auto_decomposition_param,
        };
        Self {
            param: **param,
            ring_packing_param,
            ring: RingOps::new(param.modulus, param.ring_size),
            mod_ks: ModulusOps::new(param.lwe_modulus),
        }
    }

    fn param(&self) -> &FhewBoolParam {
        &self.param
    }

    fn ring_packing_param(&self) -> &RingPackingParam {
        &self.ring_packing_param
    }

    fn ring(&self) -> &Self::Ring {
        &self.ring
    }

    fn mod_ks(&self) -> &Self::KeySwitchMod {
        &self.mod_ks
    }

    fn ring_rp(&self) -> &Self::PackingRing {
        &self.ring
    }

    fn pack<'a>(
        &self,
        rp_key: &RingPackingKeyOwned<<Self::PackingRing as RingOps>::EvalPrep>,
        cts: impl IntoIterator<Item = &'a FhewBoolCiphertextOwned<Elem<Self::Ring>>>,
    ) -> FhewBoolPackedCiphertextOwned<Elem<Self::PackingRing>> {
        FhewBoolPackedCiphertext::pack(self.ring_rp(), rp_key, cts)
    }
}
