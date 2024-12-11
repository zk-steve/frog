use phantom_zone_evaluator::boolean::fhew::prelude::{
    FhewBoolMpiCrs, HierarchicalSeedableRng, RingPackingCrs,
};
use rand::prelude::StdRng;
use rand::{thread_rng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Crs(<StdRng as SeedableRng>::Seed);

impl Crs {
    pub fn new(seed: <StdRng as SeedableRng>::Seed) -> Self {
        Self(seed)
    }

    pub fn from_entropy() -> Self {
        Self::new(thread_rng().gen())
    }

    pub fn fhew(&self) -> FhewBoolMpiCrs<StdRng> {
        FhewBoolMpiCrs::new(StdRng::from_hierarchical_seed(self.0, &[0]).gen())
    }

    pub fn ring_packing(&self) -> RingPackingCrs<StdRng> {
        RingPackingCrs::new(StdRng::from_hierarchical_seed(self.0, &[1]).gen())
    }
}
