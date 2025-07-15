use crate::{
    backends::{G1ArithmeticBackend, HashBackend},
    decider::{types::VerifierMemory, verifier::DeciderVerifier},
    keys::verification_key::VerifyingKey,
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
        let mut decider_verifier = DeciderVerifier::new(memory);

        let sumcheck_output =
            decider_verifier.verify_sumcheck::<H>(&mut transcript, vk.circuit_size)?;
        let shplemini_output = decider_verifier.verify_shplemini::<H, P>(
            &mut transcript,
            sumcheck_output.multivariate_challenge,
            vk.circuit_size,
        )?;

        Ok(sumcheck_output.verified && shplemini_output)
    }
}