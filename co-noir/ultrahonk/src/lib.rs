#![warn(clippy::iter_over_hash_type)]

pub(crate) mod honk_curve;
pub(crate) mod decider;
pub(crate) mod keccak_hash;
pub(crate) mod oink;
pub(crate) mod sponge_hasher;
pub(crate) mod verifier;
pub mod prelude;
pub mod transcript;
pub mod types;
pub mod polynomials;
pub mod keys;
pub mod serialize;

// use builder::{HonkProofResult};
use ark_ec::{ VariableBaseMSM};
use ark_ec::pairing::Pairing;
use ark_ff::PrimeField;
// use co_builder::HonkProofError;

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



pub type HonkProofResult<T> = std::result::Result<T, HonkProofError>;

/// The errors that may arise during the computation of a HONK proof.
#[derive(Debug, thiserror::Error)]
pub enum HonkProofError {
    /// Indicates that the witness is too small for the provided circuit.
    #[error("Cannot index into witness {0}")]
    CorruptedWitness(usize),
    /// Indicates that the crs is too small
    #[error("CRS too small")]
    CrsTooSmall,
    /// The proof has too few elements
    #[error("Proof too small")]
    ProofTooSmall,
    /// Invalid proof length
    #[error("Invalid proof length")]
    InvalidProofLength,
    /// Invalid key length
    #[error("Invalid key length")]
    InvalidKeyLength,
    /// Corrupted Key
    #[error("Corrupted Key")]
    CorruptedKey,
    /// Expected Public Witness, Shared received
    #[error("Expected Public Witness, Shared received")]
    ExpectedPublicWitness,
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    /// Gemini evaluation challenge is in the SmallSubgroup
    #[error("Gemini evaluation challenge is in the SmallSubgroup.")]
    GeminiSmallSubgroup,
    /// The Subgroup for the FFT domain is too large
    #[error("Too large Subgroup")]
    LargeSubgroup,
}


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

    pub fn msm<P: Pairing>(poly: &[P::ScalarField], crs: &[P::G1Affine]) -> HonkProofResult<P::G1> {
        if poly.len() > crs.len() {
            return Err(HonkProofError::CrsTooSmall);
        }
        Ok(P::G1::msm_unchecked(crs, poly))
    }
}
