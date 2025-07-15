#![no_std]
#![warn(clippy::iter_over_hash_type)]

#[macro_use]
extern crate alloc;

pub mod backends;
pub mod constants;
pub mod decider;
pub mod keys;
pub mod oink;
pub mod serialize;
pub mod transcript;
pub mod types;
pub mod verifier;

use alloc::borrow::ToOwned;
use ark_bn254::{Fq, Fr};
use ark_ff::{BigInteger, PrimeField};

use crate::{
    constants::{NUM_BASEFIELD_ELEMENTS, NUM_SCALARFIELD_ELEMENTS},
    types::ScalarField,
};

pub(crate) const NUM_ALPHAS: usize = decider::sumcheck::relations::NUM_SUBRELATIONS - 1;

/// The log of the max circuit size assumed in order to achieve constant sized Honk proofs
/// AZTEC TODO(<https://github.com/AztecProtocol/barretenberg/issues/1046>): Remove the need for const sized proofs
pub(crate) const CONST_PROOF_SIZE_LOG_N: usize = 28;

// The interleaving trick needed for Translator adds 2 extra claims to Gemini fold claims
// AZTEC TODO(https://github.com/AztecProtocol/barretenberg/issues/1293): Decouple Gemini from Interleaving
pub(crate) const NUM_INTERLEAVING_CLAIMS: u32 = 2;

pub(crate) struct Utils {}

impl Utils {
    fn convert_scalarfield_back(src: &[ScalarField]) -> ScalarField {
        debug_assert_eq!(src.len(), NUM_SCALARFIELD_ELEMENTS);
        src[0].to_owned()
    }

    fn convert_basefield_back(src: &[Fr]) -> Fq {
        debug_assert_eq!(src.len(), NUM_BASEFIELD_ELEMENTS);

        // Get the raw field element as little-endian bytes
        let res0_bytes = src[0].into_bigint().to_bytes_le();
        let res1_bytes = src[1].into_bigint().to_bytes_le();

        // Extract lower 136 bits from res0 (17 bytes)
        let mut value_bytes = [0u8; 32];
        value_bytes[..17].copy_from_slice(&res0_bytes[..17]);

        // Extract upper 118 bits from res1 (15 bytes)
        // Place them at offset 17 (i.e., shifted by 136 bits)
        value_bytes[17..(15 + 17)].copy_from_slice(&res1_bytes[..15]);

        // Now value_bytes is the 256-bit little-endian representation
        Fq::from_le_bytes_mod_order(&value_bytes)
    }
}
