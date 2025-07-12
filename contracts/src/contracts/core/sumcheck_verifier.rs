use crate::utils::backends::PrecompileHashBackend;
use alloc::vec::Vec;
use alloy_sol_types::sol;
use stylus_sdk::{abi::Bytes, prelude::*};
use ultrahonk::decider::types::VerifierMemory;
use ultrahonk::decider::verifier::DeciderVerifier;
use ultrahonk::serialize::{BytesDeserializable, BytesSerializable};
use ultrahonk::transcript::Transcript;

sol_storage! {
    #[cfg_attr(feature = "sumcheck-verifier", entrypoint)]
    pub struct SumcheckVerifierContract {}
}

// Define errors that can occur during the execution of the contract
sol! {
    error TranscriptDeserializationFailed();
    error MemoryDeserializationFailed();
    error SumcheckVerificationFailed();
}

#[derive(SolidityError)]
pub enum VerifierErrors {
    SumcheckVerificationFailed(SumcheckVerificationFailed),
    TranscriptDeserializationFailed(TranscriptDeserializationFailed),
    MemoryDeserializationFailed(MemoryDeserializationFailed),
}

#[public]
impl SumcheckVerifierContract {
    pub fn verify(
        &self,
        memory_bytes: Bytes,
        transcript_bytes: Bytes,
        circuit_size: u32,
    ) -> Result<(Bytes, Bytes, Bytes, bool), VerifierErrors> {
        // deserialize transcript
        let mut transcript = Transcript::deserialize_from_bytes(transcript_bytes.as_slice())
            .map_err(|_| {
                VerifierErrors::TranscriptDeserializationFailed(TranscriptDeserializationFailed {})
            })?;

        // deserialize memory and create decider verifier
        let memory =
            VerifierMemory::deserialize_from_bytes(memory_bytes.as_slice()).map_err(|_| {
                VerifierErrors::MemoryDeserializationFailed(MemoryDeserializationFailed {})
            })?;
        let mut decider_verifier = DeciderVerifier::new(memory);

        // verify sumcheck
        let sumcheck_output = decider_verifier
            .verify_sumcheck::<PrecompileHashBackend>(&mut transcript, circuit_size)
            .map_err(|_| {
                VerifierErrors::SumcheckVerificationFailed(SumcheckVerificationFailed {})
            })?;

        Ok((
            decider_verifier.memory.serialize_to_bytes().into(),
            transcript.serialize_to_bytes().into(),
            sumcheck_output
                .multivariate_challenge
                .serialize_to_bytes()
                .into(),
            sumcheck_output.verified,
        ))
    }
}
