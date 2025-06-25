use crate::{
    backends::{G1ArithmeticBackend, HashBackend},
    decider::{types::VerifierMemory, verifier::DeciderVerifier},
    keys::verification_key::VerifyingKey,
    transcript::Transcript,
    types::{HonkProof, HonkProofError, ScalarField},
};
use core::marker::PhantomData;

pub struct UltraHonk<P: G1ArithmeticBackend, H: HashBackend> {
    phantom_data: PhantomData<P>,
    phantom_hasher: PhantomData<H>,
}

pub(crate) type HonkVerifyResult<T> = Result<T, HonkProofError>;

impl<P: G1ArithmeticBackend, H: HashBackend> UltraHonk<P, H> {
    pub fn verify(
        honk_proof: HonkProof,
        public_inputs: &[ScalarField],
        vk: &VerifyingKey,
    ) -> HonkVerifyResult<bool> {
        let honk_proof = honk_proof.insert_public_inputs(public_inputs.to_vec());

        let mut transcript = Transcript::new_verifier(honk_proof);
        let memory = VerifierMemory::from_key_and_transcript::<P, H>(vk, &mut transcript);

        let mut decider_verifier = DeciderVerifier::<P, H>::new(memory);
        let sumcheck_output = decider_verifier.verify_sumcheck(&mut transcript, vk.circuit_size)?;
        let shplemini_output = decider_verifier.verify_shplemini(
            &mut transcript,
            sumcheck_output.multivariate_challenge,
            vk.circuit_size,
        )?;

        Ok(sumcheck_output.verified && shplemini_output)
    }
}
