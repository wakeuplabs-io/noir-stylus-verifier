use crate::must_deser;
use crate::utils::backends::PrecompileHashBackend;
use alloc::vec::Vec;
use stylus_sdk::{abi::Bytes, prelude::*};
use ultrahonk::constants::{BATCHED_RELATION_PARTIAL_LENGTH, BATCHED_RELATION_PARTIAL_LENGTH_ZK};
use ultrahonk::decider::sumcheck::verifier::{SumcheckVerifier, SumcheckVerifierMemory};
use ultrahonk::oink::types::Challenges;
use ultrahonk::serialize::BytesSerializable;
use ultrahonk::transcript::Transcript;
use ultrahonk::types::ScalarField;

sol_storage! {
    #[cfg_attr(any(feature = "sumcheck-verifier", feature = "zk-sumcheck-verifier"), entrypoint)]
    pub struct SumcheckVerifierContract {}
}

#[public]
impl SumcheckVerifierContract {
    pub fn verify(
        &self,
        transcript_bytes: Bytes,
        challenges_bytes: Bytes,
        public_input_delta_bytes: Bytes,
        gate_challenges_bytes: Bytes,
        circuit_size: u32,
    ) -> (bool, Bytes, Bytes, Bytes, Bytes, Bytes) {
        // deserialize parameters
        let mut transcript = must_deser!(Transcript, transcript_bytes);
        let challenges = must_deser!(Challenges, challenges_bytes);
        let public_input_delta = must_deser!(ScalarField, public_input_delta_bytes);
        let gate_challenges = must_deser!(Vec<ScalarField>, gate_challenges_bytes);

        // build sumcheck verifier
        let mut sumcheck_verifier =
            SumcheckVerifier::new(SumcheckVerifierMemory::from_memory_and_gate_challenges::<
                PrecompileHashBackend,
            >(
                &challenges, gate_challenges, public_input_delta
            ));

        let (sumcheck_output, libra_commitments) = sumcheck_verifier
            .verify_sumcheck::<PrecompileHashBackend, {
                #[cfg(feature = "zk-sumcheck-verifier")]
                {
                    BATCHED_RELATION_PARTIAL_LENGTH_ZK
                }

                #[cfg(not(feature = "zk-sumcheck-verifier"))]
                {
                    BATCHED_RELATION_PARTIAL_LENGTH
                }
            }>(
                &mut transcript,
                circuit_size,
                cfg!(feature = "zk-sumcheck-verifier"),
            )
            .unwrap();

        (
            sumcheck_output.verified,
            sumcheck_verifier
                .memory
                .claimed_evaluations
                .serialize_to_bytes()
                .into(),
            transcript.serialize_to_bytes().into(),
            sumcheck_output
                .multivariate_challenge
                .serialize_to_bytes()
                .into(),
            sumcheck_output
                .claimed_libra_evaluation
                .serialize_to_bytes()
                .into(),
            libra_commitments.serialize_to_bytes().into(),
        )
    }
}
