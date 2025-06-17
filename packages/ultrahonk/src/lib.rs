#![warn(clippy::iter_over_hash_type)]

#[macro_use]
extern crate alloc;

pub mod backends;
pub mod decider;
pub mod gadgets;
pub mod honk_curve;
pub mod keys;
pub mod oink;
pub mod polynomials;
pub mod prelude;
pub mod serde_compat;
pub mod serialize;
pub mod transcript;
pub mod types;
pub mod verifier;

use ark_bn254::{Fq, Fr};
use ark_ff::{One, PrimeField};
use num_bigint::BigUint;

use crate::{
    honk_curve::{NUM_BASEFIELD_ELEMENTS, NUM_SCALARFIELD_ELEMENTS},
    types::ScalarField,
};

// TODO: check these constants are in use or remove

pub const NUM_ALPHAS: usize = decider::relations::NUM_SUBRELATIONS - 1;
/// The log of the max circuit size assumed in order to achieve constant sized Honk proofs
/// AZTEC TODO(<https://github.com/AztecProtocol/barretenberg/issues/1046>): Remove the need for const sized proofs
pub const CONST_PROOF_SIZE_LOG_N: usize = 28;
// For ZK Flavors: the number of the commitments required by Libra and SmallSubgroupIPA.
pub const NUM_LIBRA_COMMITMENTS: usize = 3;
pub const NUM_SMALL_IPA_EVALUATIONS: usize = 4;
// Upper bound on the number of claims produced GeminiProver:
// - Each fold polynomial is opened at two points, the number of resulting claims is bounded by 2*CONST_PROOF_SIZE_LOG_N
// - The interleaving trick needed for Translator adds 2 extra claims
// AZTEC TODO(https://github.com/AztecProtocol/barretenberg/issues/1293): Decouple Gemini from Interleaving
pub const NUM_GEMINI_CLAIMS: usize = 2 * CONST_PROOF_SIZE_LOG_N + 2;
// The interleaving trick needed for Translator adds 2 extra claims to Gemini fold claims
// TODO(https://github.com/AztecProtocol/barretenberg/issues/1293): Decouple Gemini from Interleaving
pub const NUM_INTERLEAVING_CLAIMS: u32 = 2;

pub struct Utils {}

impl Utils {
    pub fn get_msb32(inp: u32) -> u32 {
        inp.ilog2()
    }

    pub fn get_msb64(inp: u64) -> u32 {
        inp.ilog2()
    }

    fn batch_invert<F: PrimeField>(coeffs: &mut [F]) {
        ark_ff::batch_inversion(coeffs);
    }

    fn convert_scalarfield_back(src: &[ScalarField]) -> ScalarField {
        debug_assert_eq!(src.len(), NUM_SCALARFIELD_ELEMENTS);
        src[0].to_owned()
    }

    fn convert_basefield_back(src: &[Fr]) -> Fq {
        debug_assert_eq!(src.len(), NUM_BASEFIELD_ELEMENTS);
        bn254_fq_to_fr_rev(&src[0], &src[1])
    }
}

const NUM_LIMB_BITS: u32 = 68;
const TOTAL_BITS: u32 = 254;

fn bn254_fq_to_fr_rev(res0: &Fr, res1: &Fr) -> Fq {
    // Combines the two elements into one uint256_t, and then convert that to a grumpkin::fr

    let res0 = BigUint::from(*res0);
    let res1 = BigUint::from(*res1);

    debug_assert!(res0 < (BigUint::one() << (NUM_LIMB_BITS * 2))); // lower 136 bits
    debug_assert!(res1 < (BigUint::one() << (TOTAL_BITS - NUM_LIMB_BITS * 2))); // upper 254-136=118 bits

    let value = res0 + (res1 << (NUM_LIMB_BITS * 2));
    ark_bn254::Fq::from(value)
}
