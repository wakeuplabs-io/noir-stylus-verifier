//! # Ultra Honk Verifier
//!
//! This module contains the main verification logic for Ultra Honk proofs.
//! It orchestrates the three-phase verification process: Oink, Sumcheck, and Shplemini.

use crate::{
    backends::{G1ArithmeticBackend, HashBackend},
    constants::{BATCHED_RELATION_PARTIAL_LENGTH, BATCHED_RELATION_PARTIAL_LENGTH_ZK},
    decider::{
        shplemini::verifier::{ShpleminiVerifier, ShpleminiVerifierMemory},
        sumcheck::verifier::{SumcheckVerifier, SumcheckVerifierMemory},
    },
    keys::verification_key::VerifyingKey,
    oink::verifier::OinkVerifier,
    transcript::Transcript,
    types::{HonkProof, HonkVerifyResult, ScalarField},
    CONST_PROOF_SIZE_LOG_N,
};

/// The main Ultra Honk verifier implementation.
/// 
/// This struct provides the entry point for verifying Ultra Honk proofs. It coordinates
/// the three-phase verification process:
/// 1. **Oink**: Handles the initial verification setup and polynomial commitments
/// 2. **Sumcheck**: Performs the interactive sumcheck protocol verification
/// 3. **Shplemini**: Verifies the polynomial commitment scheme and final consistency
/// 
/// The verifier is designed to work with pluggable backends for cryptographic operations,
/// allowing it to run in different environments (e.g., native Rust, smart contracts).
pub struct UltraHonk;

impl UltraHonk {
    /// Verifies an Ultra Honk proof with the given public inputs and verification key.
    /// 
    /// This is the main verification entry point that orchestrates the complete
    /// Ultra Honk verification protocol. The verification process consists of three phases:
    /// 
    /// 1. **Oink Phase**: Verifies polynomial commitments and extracts challenges
    /// 2. **Sumcheck Phase**: Performs the interactive sumcheck protocol to verify 
    ///    the constraint satisfaction
    /// 3. **Shplemini Phase**: Verifies the polynomial commitment scheme using the
    ///    KZG-style opening proofs
    /// 
    /// # Type Parameters
    /// 
    /// * `H` - Hash backend for transcript operations and Fiat-Shamir challenges
    /// * `P` - G1 arithmetic backend for elliptic curve operations and pairings
    /// 
    /// # Arguments
    /// 
    /// * `honk_proof` - The Ultra Honk proof to verify
    /// * `public_inputs` - Public inputs to the circuit 
    /// * `vk` - Verification key containing circuit-specific parameters
    /// * `has_zk` - Whether this is a zero-knowledge proof (affects batch sizes)
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(true)` if the proof is valid, `Ok(false)` if invalid, or an error
    /// if verification cannot be completed due to malformed inputs or cryptographic failures.
    /// 
    /// # Errors
    /// 
    /// This function can return various `HonkProofError` variants if:
    /// - The proof is malformed or too small
    /// - Cryptographic operations fail (e.g., pairing checks)
    /// - The evaluation challenge is in a small subgroup
    /// - Internal consistency checks fail
    pub fn verify<H: HashBackend, P: G1ArithmeticBackend>(
        honk_proof: HonkProof,
        public_inputs: &[ScalarField],
        vk: &VerifyingKey,
        has_zk: bool,
    ) -> HonkVerifyResult<bool> {
        let mut transcript =
            Transcript::new_verifier(honk_proof.insert_public_inputs(public_inputs.to_vec()));

        let oink_verifier = OinkVerifier::default();
        let oink_memory = oink_verifier.verify::<H>(vk, &mut transcript).unwrap();

        let mut gate_challenges: Vec<ScalarField> = Vec::with_capacity(CONST_PROOF_SIZE_LOG_N);
        for _ in 0..CONST_PROOF_SIZE_LOG_N {
            let chall = transcript.get_challenge::<H>(); // format!("Sumcheck:gate_challenge_{}", idx)
            gate_challenges.push(chall);
        }

        // build sumcheck verifier
        let mut sumcheck_verifier = SumcheckVerifier::new(
            SumcheckVerifierMemory::from_memory_and_gate_challenges::<H>(
                &oink_memory.challenges,
                gate_challenges,
                oink_memory.public_input_delta,
            ),
        );
        let (sumcheck_output, libra_commitments) = if has_zk {
            sumcheck_verifier.verify_sumcheck::<H, BATCHED_RELATION_PARTIAL_LENGTH_ZK>(
                &mut transcript,
                vk.circuit_size,
                has_zk,
            )?
        } else {
            sumcheck_verifier.verify_sumcheck::<H, BATCHED_RELATION_PARTIAL_LENGTH>(
                &mut transcript,
                vk.circuit_size,
                has_zk,
            )?
        };
        if !sumcheck_output.verified {
            return Ok(false);
        }

        // build shplemini verifier
        let mut shplemini_verifier = ShpleminiVerifier::new(ShpleminiVerifierMemory::new(
            oink_memory.witness_commitments.clone(),
            vk.commitments.clone(),
            sumcheck_verifier.memory.claimed_evaluations.clone(),
        ));
        let (pcs_verified, libra_evaluations, gemini_evaluation_challenge) = shplemini_verifier
            .verify_shplemini::<H, P>(
                &mut transcript,
                sumcheck_output.multivariate_challenge.clone(),
                vk.circuit_size,
                libra_commitments,
            )?;
        if !pcs_verified {
            return Ok(false);
        }

        // consistency check
        let mut consistency_checked = true;
        if has_zk {
            consistency_checked = ShpleminiVerifier::check_evaluations_consistency(
                &libra_evaluations,
                gemini_evaluation_challenge,
                &sumcheck_output.multivariate_challenge,
                sumcheck_output.claimed_libra_evaluation,
            )?;
        }

        Ok(sumcheck_output.verified && pcs_verified && consistency_checked)
    }
}
