//! # Fiat-Shamir Transcript
//!
//! This module implements the Fiat-Shamir transcript used in Ultra Honk for
//! generating cryptographic challenges from proof data. The transcript maintains
//! the verifier's state during the interactive protocol simulation.
//!
//! ## Functionality
//!
//! The transcript provides:
//! - Proof data management and reading
//! - Challenge generation using cryptographic hashing
//! - State management for multiple rounds of interaction
//! - Serialization support for cross-environment compatibility
//!
//! ## Protocol Flow
//!
//! 1. Initialize with proof data
//! 2. Read commitments and values from the proof
//! 3. Generate challenges using Fiat-Shamir transformation
//! 4. Maintain state across multiple verification rounds

use crate::alloc::borrow::ToOwned;
use crate::backends::HashBackend;
use crate::constants::{NUM_BASEFIELD_ELEMENTS, NUM_SCALARFIELD_ELEMENTS};
use crate::serialize::{BytesDeserializable, BytesSerializable};
use crate::types::{G1Affine, HonkProof, HonkProofError, HonkVerifyResult, ScalarField};
use crate::Utils;
use alloc::vec::Vec;
use ark_ec::AffineRepr;
use ark_ff::{BigInteger, PrimeField, Zero};

/// Fiat-Shamir transcript for Ultra Honk verification.
///
/// The transcript manages the verifier's state during proof verification,
/// including proof data, challenge generation, and round management. It
/// implements the Fiat-Shamir transformation to convert interactive protocols
/// into non-interactive ones.
#[derive(Clone)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct Transcript {
    /// The proof data containing all commitments and values from the prover
    pub(crate) proof_data: Vec<ScalarField>,
    /// Number of field elements written to the transcript
    pub(crate) num_frs_written: usize,
    /// Number of field elements read from the proof data
    pub(crate) num_frs_read: usize,
    /// Current round number in the protocol
    pub(crate) round_number: usize,
    /// Whether this is the first challenge being generated
    pub(crate) is_first_challenge: bool,
    /// Data accumulated in the current round for challenge generation
    pub(crate) current_round_data: Vec<ScalarField>,
    /// The previous challenge value for chaining
    pub(crate) previous_challenge: ScalarField,
}

impl Default for Transcript {
    fn default() -> Self {
        Self::new()
    }
}

impl Transcript {
    /// Creates a new empty transcript for proof generation.
    pub fn new() -> Self {
        Self {
            proof_data: Default::default(),
            num_frs_written: 0,
            num_frs_read: 0,
            round_number: 0,
            is_first_challenge: true,
            current_round_data: Default::default(),
            previous_challenge: Default::default(),
        }
    }

    /// Creates a new transcript for verification with the given proof data.
    ///
    /// # Arguments
    ///
    /// * `proof` - The Ultra Honk proof containing all prover commitments and values
    pub fn new_verifier(proof: HonkProof) -> Self {
        Self {
            proof_data: proof.inner(),
            num_frs_written: 0,
            num_frs_read: 0,
            round_number: 0,
            is_first_challenge: true,
            current_round_data: Default::default(),
            previous_challenge: Default::default(),
        }
    }

    /// Consumes the transcript and returns the accumulated proof data.
    ///
    /// # Returns
    ///
    /// The HonkProof containing all the data written to the transcript.
    pub fn get_proof(self) -> HonkProof {
        HonkProof::new(self.proof_data)
    }

    fn add_element_frs_to_hash_buffer(&mut self, elements: &[ScalarField]) {
        // Add an entry to the current round of the manifest
        let len = elements.len();
        self.current_round_data.extend(elements);
        self.num_frs_written += len;
    }

    // Adds an element to the transcript.
    // Serializes the element to frs and adds it to the current_round_data buffer. Does NOT add the element to the proof. This is used for elements which should be part of the transcript but are not in the final proof (e.g. circuit size)
    fn add_to_hash_buffer(&mut self, elements: &[ScalarField]) {
        self.add_element_frs_to_hash_buffer(elements);
    }

    pub(crate) fn add_u64_to_hash_buffer(&mut self, element: u64) {
        let el = ScalarField::from(element);
        self.add_to_hash_buffer(&[el]);
    }

    fn receive_n_from_prover(&mut self, n: usize) -> HonkVerifyResult<Vec<ScalarField>> {
        if self.num_frs_read + n > self.proof_data.len() {
            return Err(HonkProofError::ProofTooSmall);
        }
        let elements = self.proof_data[self.num_frs_read..self.num_frs_read + n].to_owned();
        self.num_frs_read += n;

        self.add_element_frs_to_hash_buffer(&elements);
        Ok(elements)
    }

    pub(crate) fn receive_fr_from_prover(&mut self) -> HonkVerifyResult<ScalarField> {
        let elements = self.receive_n_from_prover(NUM_SCALARFIELD_ELEMENTS)?;

        Ok(Utils::convert_scalarfield_back(&elements))
    }

    pub(crate) fn receive_point_from_prover(&mut self) -> HonkVerifyResult<G1Affine> {
        let elements = self.receive_n_from_prover(NUM_BASEFIELD_ELEMENTS * 2)?;

        let coords = elements
            .chunks_exact(NUM_BASEFIELD_ELEMENTS)
            .map(Utils::convert_basefield_back)
            .collect::<Vec<_>>();

        let x = coords[0];
        let y = coords[1];

        let res = if x.is_zero() && y.is_zero() {
            G1Affine::zero()
        } else {
            G1Affine::new(x, y)
        };

        Ok(res)
    }

    pub(crate) fn receive_fr_vec_from_verifier(
        &mut self,
        n: usize,
    ) -> HonkVerifyResult<Vec<ScalarField>> {
        let elements = self.receive_n_from_prover(NUM_SCALARFIELD_ELEMENTS * n)?;

        let elements = elements
            .chunks_exact(NUM_SCALARFIELD_ELEMENTS)
            .map(Utils::convert_scalarfield_back)
            .collect();

        Ok(elements)
    }

    pub(crate) fn receive_fr_array_from_verifier<const SIZE: usize>(
        &mut self,
    ) -> HonkVerifyResult<[ScalarField; SIZE]> {
        let mut res: [ScalarField; SIZE] = [ScalarField::zero(); SIZE];
        let elements = self.receive_n_from_prover(NUM_SCALARFIELD_ELEMENTS * SIZE)?;

        for (src, des) in elements
            .chunks_exact(NUM_SCALARFIELD_ELEMENTS)
            .zip(res.iter_mut())
        {
            let el = Utils::convert_scalarfield_back(src);
            *des = el;
        }
        Ok(res)
    }

    fn split_challenge(challenge: ScalarField) -> [ScalarField; 2] {
        // Get the 32 bytes (256 bits) in little-endian order
        let bytes = challenge.into_bigint().to_bytes_le();

        // Lower 128 bits (first 16 bytes)
        let mut lo_bytes = [0u8; 32];
        lo_bytes[..16].copy_from_slice(&bytes[..16]);
        let lo = ScalarField::from_le_bytes_mod_order(&lo_bytes);

        // Upper 128 bits (next 16 bytes)
        let mut hi_bytes = [0u8; 32];
        hi_bytes[..16].copy_from_slice(&bytes[16..32]);
        let hi = ScalarField::from_le_bytes_mod_order(&hi_bytes);

        [lo, hi]
    }

    fn get_next_duplex_challenge_buffer<H: HashBackend>(
        &mut self,
        num_challenges: usize,
    ) -> [ScalarField; 2] {
        // challenges need at least 110 bits in them to match the presumed security parameter of the BN254 curve.
        assert!(num_challenges <= 2);
        // Prevent challenge generation if this is the first challenge we're generating,
        // AND nothing was sent by the prover.
        if self.is_first_challenge {
            assert!(!self.current_round_data.is_empty());
        }
        // concatenate the previous challenge (if this is not the first challenge) with the current round data.

        let mut full_buffer = Vec::new();
        core::mem::swap(&mut full_buffer, &mut self.current_round_data);

        if self.is_first_challenge {
            // Update is_first_challenge for the future
            self.is_first_challenge = false;
        } else {
            // if not the first challenge, we can use the previous_challenge
            full_buffer.insert(0, self.previous_challenge);
        }

        // Hash the full buffer with poseidon2, which is believed to be a collision resistant hash function and a random
        // oracle, removing the need to pre-hash to compress and then hash with a random oracle, as we previously did
        // with Pedersen and Blake3s.
        let new_challenge_bytes = H::hash(full_buffer.serialize_to_bytes().as_slice());
        let (new_challenge, _) = ScalarField::deserialize_from_bytes(&new_challenge_bytes).unwrap();
        let new_challenges = Self::split_challenge(new_challenge);

        // update previous challenge buffer for next time we call this function
        self.previous_challenge = new_challenge;
        new_challenges
    }

    /// Generates a single cryptographic challenge using the Fiat-Shamir transformation.
    ///
    /// # Type Parameters
    ///
    /// * `H` - Hash backend used for challenge generation
    ///
    /// # Returns
    ///
    /// A random field element derived from the current transcript state.
    pub fn get_challenge<H: HashBackend>(&mut self) -> ScalarField {
        let challenge = self.get_next_duplex_challenge_buffer::<H>(1)[0];
        let res = challenge.to_owned();
        self.round_number += 1;
        res
    }

    /// Generates multiple cryptographic challenges efficiently.
    ///
    /// This method generates multiple challenges by splitting hash outputs,
    /// reducing the number of hash operations needed.
    ///
    /// # Type Parameters
    ///
    /// * `H` - Hash backend used for challenge generation
    ///
    /// # Arguments
    ///
    /// * `num_challenges` - Number of challenges to generate
    ///
    /// # Returns
    ///
    /// Vector of random field elements derived from the transcript state.
    pub(crate) fn get_challenges<H: HashBackend>(
        &mut self,
        num_challenges: usize,
    ) -> Vec<ScalarField> {
        let mut res = Vec::with_capacity(num_challenges);
        for _ in 0..num_challenges >> 1 {
            let challenge_buffer = self.get_next_duplex_challenge_buffer::<H>(2);
            res.push(challenge_buffer[0].to_owned());
            res.push(challenge_buffer[1].to_owned());
        }
        if num_challenges & 1 == 1 {
            let challenge_buffer = self.get_next_duplex_challenge_buffer::<H>(1);
            res.push(challenge_buffer[0].to_owned());
        }

        self.round_number += 1;
        res
    }
}
