use std::fmt;

use phantom_zone_evaluator::boolean::fhew::prelude::{
    Elem, FhewBoolParam, ModulusOps, NativeRing, NoisyNativeRing, NonNativePowerOfTwo, PrimeRing,
    RingOps, RingPackingKeyOwned, RingPackingParam,
};
use phantom_zone_evaluator::boolean::fhew::{
    FhewBoolCiphertextOwned, FhewBoolPackedCiphertext, FhewBoolPackedCiphertextOwned,
};
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::ops::Ops;
use crate::param::Param;

#[derive(Debug, Clone)]
pub struct NativeOps {
    param: Param,
    ring_packing_param: RingPackingParam,
    ring: NativeRing,
    mod_ks: NonNativePowerOfTwo,
    ring_rp: PrimeRing,
}

impl Serialize for NativeOps {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("NativeOps", 1)?;
        state.serialize_field("param", &self.param)?;
        state.end()
    }
}
impl<'de> Deserialize<'de> for NativeOps {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Param,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl Visitor<'_> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`param`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "param" => Ok(Field::Param),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct NativeOpsVisitor;

        impl<'de> Visitor<'de> for NativeOpsVisitor {
            type Value = NativeOps;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct NativeOps")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<NativeOps, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let param = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                Ok(NativeOps::new(param))
            }

            fn visit_map<V>(self, mut map: V) -> Result<NativeOps, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut param = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Param => {
                            if param.is_some() {
                                return Err(de::Error::duplicate_field("param"));
                            }
                            param = Some(map.next_value()?);
                        }
                    }
                }
                let param = param.unwrap();
                Ok(NativeOps::new(param))
            }
        }

        const FIELDS: &[&str] = &["param"];
        deserializer.deserialize_struct("NativeOps", FIELDS, NativeOpsVisitor)
    }
}

impl Ops for NativeOps {
    type Ring = NativeRing;
    type EvaluationRing = NoisyNativeRing;
    type KeySwitchMod = NonNativePowerOfTwo;
    type PackingRing = PrimeRing;

    fn new(param: Param) -> Self {
        let ring_packing_param = RingPackingParam {
            modulus: param.ring_packing_modulus.unwrap(),
            ring_size: param.ring_size,
            sk_distribution: param.sk_distribution,
            noise_distribution: param.noise_distribution,
            auto_decomposition_param: param.ring_packing_auto_decomposition_param,
        };
        Self {
            param,
            ring_packing_param,
            ring: RingOps::new(param.modulus, param.ring_size),
            mod_ks: ModulusOps::new(param.lwe_modulus),
            ring_rp: RingOps::new(ring_packing_param.modulus, param.ring_size),
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
        &self.ring_rp
    }

    fn pack<'a>(
        &self,
        rp_key: &RingPackingKeyOwned<<Self::PackingRing as RingOps>::EvalPrep>,
        cts: impl IntoIterator<Item = &'a FhewBoolCiphertextOwned<Elem<Self::Ring>>>,
    ) -> FhewBoolPackedCiphertextOwned<Elem<Self::PackingRing>> {
        FhewBoolPackedCiphertext::pack_ms(self.ring(), self.ring_rp(), rp_key, cts)
    }
}
