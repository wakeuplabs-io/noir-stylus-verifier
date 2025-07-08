use alloc::vec::Vec;

use crate::{
    backends::{G1ArithmeticBackend, HashBackend},
    decider::{types::VerifierMemory, verifier::DeciderVerifier},
    keys::verification_key::VerifyingKey,
    serialize::{BytesDeserializable, BytesSerializable},
    transcript::Transcript,
    types::{HonkProof, HonkProofError, ScalarField},
};

pub struct UltraHonk;

pub(crate) type HonkVerifyResult<T> = Result<T, HonkProofError>;

impl UltraHonk {
    pub fn verify<H: HashBackend, P: G1ArithmeticBackend>(
        honk_proof: HonkProof,
        public_inputs: &[ScalarField],
        vk: &VerifyingKey,
    ) -> HonkVerifyResult<bool> {
        let honk_proof = honk_proof.insert_public_inputs(public_inputs.to_vec());

        let mut transcript = Transcript::new_verifier(honk_proof);
        let memory = VerifierMemory::from_key_and_transcript::<H>(vk, &mut transcript);
        // let memory_bytes = memory.serialize_to_bytes();

        let mut decider_verifier = DeciderVerifier::new(memory);
        let sumcheck_output =
            decider_verifier.verify_sumcheck::<H>(&mut transcript, vk.circuit_size)?;

        transcript =
            Transcript::deserialize_from_bytes(transcript.serialize_to_bytes().as_slice()).unwrap();
        let memory = VerifierMemory::deserialize_from_bytes(
            decider_verifier.memory.serialize_to_bytes().as_slice(),
        )
        .unwrap();
        let mut decider_verifier2 = DeciderVerifier::new(memory);

        let shplemini_output = decider_verifier2.verify_shplemini::<H, P>(
            &mut transcript,
            sumcheck_output.multivariate_challenge,
            vk.circuit_size,
        )?;

        Ok(sumcheck_output.verified && shplemini_output)
    }
}
