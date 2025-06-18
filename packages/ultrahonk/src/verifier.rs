use crate::{
    backends::HashBackend,
    decider::{types::VerifierMemory, verifier::DeciderVerifier},
    honk_curve::HonkCurve,
    keys::verification_key::VerifyingKey,
    oink::verifier::OinkVerifier,
    transcript::Transcript,
    types::{HonkProof, HonkProofError, ScalarField},
    CONST_PROOF_SIZE_LOG_N,
};
use alloc::vec::Vec;
use core::{marker::PhantomData};

pub struct UltraHonk<P: HonkCurve, H: HashBackend> {
    phantom_data: PhantomData<P>,
    phantom_hasher: PhantomData<H>,
}

pub(crate) type HonkVerifyResult<T> = Result<T, HonkProofError>;

impl<P: HonkCurve, H: HashBackend> UltraHonk<P, H> {
    pub(crate) fn generate_gate_challenges(transcript: &mut Transcript<H>) -> Vec<ScalarField> {
        let mut gate_challenges: Vec<ScalarField> = Vec::with_capacity(CONST_PROOF_SIZE_LOG_N);

        for idx in 0..CONST_PROOF_SIZE_LOG_N {
            let chall = transcript.get_challenge(format!("Sumcheck:gate_challenge_{}", idx));
            gate_challenges.push(chall);
        }
        gate_challenges
    }

    pub fn verify(
        honk_proof: HonkProof,
        public_inputs: &[ScalarField],
        verifying_key: &VerifyingKey,
    ) -> HonkVerifyResult<bool> {
        let honk_proof = honk_proof.insert_public_inputs(public_inputs.to_vec());

        let mut transcript = Transcript::<H>::new_verifier(honk_proof);

        let oink_verifier = OinkVerifier::<P, H>::default();
        let oink_result = oink_verifier.verify(verifying_key, &mut transcript)?;

        let circuit_size = verifying_key.circuit_size;
        let crs = verifying_key.crs;

        let mut memory = VerifierMemory::from_memory_and_key(oink_result, verifying_key);
        memory.relation_parameters.gate_challenges =
            Self::generate_gate_challenges(&mut transcript);
        let decider_verifier = DeciderVerifier::<P, H>::new(memory);
        decider_verifier.verify(circuit_size, &crs, transcript)
    }
}
