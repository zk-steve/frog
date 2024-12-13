use std::ops::Deref;

use phantom_zone_evaluator::boolean::fhew::prelude::{
    DecompositionParam, FhewBoolMpiParam, Modulus,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Param {
    pub param: FhewBoolMpiParam,
    pub ring_packing_modulus: Option<Modulus>,
    pub ring_packing_auto_decomposition_param: DecompositionParam,
}

impl Deref for Param {
    type Target = FhewBoolMpiParam;

    fn deref(&self) -> &Self::Target {
        &self.param
    }
}
