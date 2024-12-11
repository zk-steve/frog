use phantom_zone_evaluator::boolean::fhew::prelude::{
    DecompositionParam, FheU64, FhewBoolMpiParam, FhewBoolParam, Gaussian, Modulus,
    NoiseDistribution, RgswDecompositionParam, SecretDistribution, Ternary,
};
use phantom_zone_evaluator::boolean::BoolEvaluator;

pub fn fhe_function<E: BoolEvaluator>(a: &FheU64<E>, b: &FheU64<E>) -> FheU64<E> {
    a + b
}

pub fn u64_to_binary<const N: usize>(v: u64) -> Vec<bool> {
    assert!((v as u128) < 2u128.pow(N as u32));
    let mut result = vec![false; N];
    for (i, item) in result.iter_mut().enumerate() {
        if (v >> i) & 1 == 1 {
            *item = true;
        }
    }
    result
}

pub fn binary_to_u64(v: Vec<bool>) -> u64 {
    assert!(v.len() <= 64);
    v.iter()
        .enumerate()
        .fold(0u64, |acc, (i, &bit)| acc | ((bit as u64) << i))
}

pub const I_2P_60: FhewBoolMpiParam = FhewBoolMpiParam {
    param: FhewBoolParam {
        message_bits: 2,
        modulus: Modulus::PowerOfTwo(64),
        ring_size: 1024,
        sk_distribution: SecretDistribution::Ternary(Ternary),
        noise_distribution: NoiseDistribution::Gaussian(Gaussian(3.19)),
        u_distribution: SecretDistribution::Ternary(Ternary),
        auto_decomposition_param: DecompositionParam {
            log_base: 18,
            level: 1,
        },
        rlwe_by_rgsw_decomposition_param: RgswDecompositionParam {
            log_base: 18,
            level_a: 1,
            level_b: 1,
        },
        lwe_modulus: Modulus::PowerOfTwo(18),
        lwe_dimension: 300,
        lwe_sk_distribution: SecretDistribution::Ternary(Ternary),
        lwe_noise_distribution: NoiseDistribution::Gaussian(Gaussian(3.19)),
        lwe_ks_decomposition_param: DecompositionParam {
            log_base: 4,
            level: 3,
        },
        q: 1024,
        g: 5,
        w: 10,
    },
    rgsw_by_rgsw_decomposition_param: RgswDecompositionParam {
        log_base: 15,
        level_a: 3,
        level_b: 3,
    },
    total_shares: 2,
};
