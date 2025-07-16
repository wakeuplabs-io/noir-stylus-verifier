use crate::must_deser;
use crate::utils::backends::{PrecompileG1ArithmeticBackend, PrecompileHashBackend};
use alloc::vec::Vec;
use stylus_sdk::{abi::Bytes, prelude::*};
use ultrahonk::decider::shplemini::verifier::{ShpleminiVerifier, ShpleminiVerifierMemory};
use ultrahonk::decider::types::ClaimedEvaluations;
use ultrahonk::serialize::BytesSerializable;
use ultrahonk::transcript::Transcript;
use ultrahonk::types::{G1Affine, PrecomputedEntities, ScalarField, WitnessEntities};

sol_storage! {
    #[cfg_attr(any(feature = "shplemini-verifier", feature = "zk-shplemini-verifier"), entrypoint)]
    pub struct ShpleminiVerifierContract {}
}

#[public]
impl ShpleminiVerifierContract {
    pub fn verify(
        &self,
        transcript_bytes: Bytes,
        witness_commitments_bytes: Bytes,
        verifier_commitments_bytes: Bytes,
        claimed_evaluations_bytes: Bytes,
        multivariate_challenge_bytes: Bytes,
        libra_commitments_bytes: Bytes,
        circuit_size: u32,
    ) -> (bool, Bytes, Bytes) {
        // deserialize parameters
        let mut transcript = must_deser!(Transcript, transcript_bytes);
        let multivariate_challenge = must_deser!(Vec<ScalarField>, multivariate_challenge_bytes);
        let libra_commitments = must_deser!(Vec<G1Affine>, libra_commitments_bytes);
        let witness_commitments = must_deser!(WitnessEntities<G1Affine>, witness_commitments_bytes);
        let claimed_evaluations = must_deser!(ClaimedEvaluations, claimed_evaluations_bytes);
        let verifier_commitments =
            must_deser!(PrecomputedEntities<G1Affine>, verifier_commitments_bytes);

        // build shplemini verifier
        let mut shplemini_verifier = ShpleminiVerifier::new(ShpleminiVerifierMemory::new(
            witness_commitments,
            verifier_commitments,
            claimed_evaluations,
        ));
        let (pcs_verified, libra_evaluations, gemini_evaluation_challenge) = shplemini_verifier
            .verify_shplemini::<PrecompileHashBackend, PrecompileG1ArithmeticBackend>(
                &mut transcript,
                multivariate_challenge,
                circuit_size,
                libra_commitments,
            )
            .unwrap();

        (
            pcs_verified,
            libra_evaluations.to_vec().serialize_to_bytes().into(),
            gemini_evaluation_challenge.serialize_to_bytes().into(),
        )
    }
}
