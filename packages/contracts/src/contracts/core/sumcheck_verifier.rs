use crate::utils::backends::PrecompileHashBackend;
use alloc::vec::Vec;
use stylus_sdk::{abi::Bytes, prelude::*};
use ultrahonk::decider::types::VerifierMemory;
use ultrahonk::decider::verifier::DeciderVerifier;
use ultrahonk::serialize::{BytesDeserializable, BytesSerializable};
use ultrahonk::transcript::Transcript;

#[cfg_attr(feature = "sumcheck-verifier", entrypoint)]
#[storage]
pub struct SumcheckVerifierContract {}

#[public]
impl SumcheckVerifierContract {
    pub fn verify(
        &self,
        memory_bytes: Bytes,
        transcript_bytes: Bytes,
        circuit_size: u32,
    ) -> (Bytes, Bytes, Bytes, bool) {
        let memory = VerifierMemory::deserialize_from_bytes(memory_bytes.as_slice()).unwrap();
        let mut transcript =
            Transcript::deserialize_from_bytes(transcript_bytes.as_slice()).unwrap();

        let mut decider_verifier = DeciderVerifier::new(memory);

        let sumcheck_output = decider_verifier
            .verify_sumcheck::<PrecompileHashBackend>(&mut transcript, circuit_size)
            .unwrap();

        (
            transcript.serialize_to_bytes().into(),
            decider_verifier.memory.serialize_to_bytes().into(),
            sumcheck_output
                .multivariate_challenge
                .serialize_to_bytes()
                .into(),
            sumcheck_output.verified,
        )
    }
}
