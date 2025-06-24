use crate::decider::types::{ClaimedEvaluations, RelationParameters};
use crate::serialize::{BytesDeserializable, BytesSerializable};
use crate::transcript::{self, TranscriptManifest};
use crate::{
    backends::{G1ArithmeticBackend, HashBackend},
    decider::{
        types::{VerifierCommitments, VerifierMemory},
        verifier::DeciderVerifier,
    },
    keys::verification_key::VerifyingKey,
    oink::verifier::OinkVerifier,
    transcript::Transcript,
    types::{HonkProof, HonkProofError, ScalarField},
    CONST_PROOF_SIZE_LOG_N,
};
use alloc::vec::Vec;
use core::marker::PhantomData;

pub struct UltraHonk<P: G1ArithmeticBackend, H: HashBackend> {
    phantom_data: PhantomData<P>,
    phantom_hasher: PhantomData<H>,
}

pub(crate) type HonkVerifyResult<T> = Result<T, HonkProofError>;

impl<P: G1ArithmeticBackend, H: HashBackend> UltraHonk<P, H> {
    fn generate_gate_challenges(transcript: &mut Transcript) -> Vec<ScalarField> {
        let mut gate_challenges: Vec<ScalarField> = Vec::with_capacity(CONST_PROOF_SIZE_LOG_N);

        for idx in 0..CONST_PROOF_SIZE_LOG_N {
            let chall = transcript.get_challenge::<H>(format!("Sumcheck:gate_challenge_{}", idx));
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

        let mut transcript = Transcript::new_verifier(honk_proof);

        let oink_verifier = OinkVerifier::<P>::default();
        let oink_result = oink_verifier.build_memory::<H>(verifying_key, &mut transcript)?;

        let circuit_size = verifying_key.circuit_size;

        let mut memory = VerifierMemory::from_memory_and_key(oink_result, verifying_key);
        memory.relation_parameters.gate_challenges =
            Self::generate_gate_challenges(&mut transcript);
        let decider_verifier = DeciderVerifier::<P, H>::new(memory);
        decider_verifier.verify(circuit_size, transcript)
    }

    pub fn verify_2(
        honk_proof: HonkProof,
        public_inputs: &[ScalarField],
        verifying_key: &VerifyingKey,
    ) -> HonkVerifyResult<bool> {
        let honk_proof = honk_proof.insert_public_inputs(public_inputs.to_vec());

        let mut transcript = Transcript::new_verifier(honk_proof);


        let mut memory = VerifierMemory::deserialize_from_bytes(
            VerifierMemory::from_key_and_transcript::<P, H>(verifying_key, &mut transcript)
                .serialize_to_bytes()
                .as_slice(),
        )
        .unwrap();

        let mut decider_verifier = DeciderVerifier::<P, H>::new(memory);

        decider_verifier.verify_sumcheck(verifying_key.circuit_size, &mut transcript)?;

        let mut memory_2 = VerifierMemory::deserialize_from_bytes(
            decider_verifier.memory.serialize_to_bytes().as_slice(),
        )
        .unwrap();

        let mut transcript_bytes = transcript.serialize_to_bytes();
        let mut transcript_2 = Transcript::deserialize_from_bytes(transcript_bytes.as_slice()).unwrap();

        let mut decider_verifier_2 = DeciderVerifier::<P, H>::new(memory_2);

        decider_verifier_2.verify_shplemini(verifying_key.circuit_size, &mut transcript_2)?;

        Ok(true)
    }
}
