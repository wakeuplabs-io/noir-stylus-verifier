use crate::utils::backends::{PrecompileG1ArithmeticBackend, PrecompileHashBackend};
use alloc::vec::Vec;
use alloy_sol_types::sol;
use stylus_sdk::{abi::Bytes, prelude::*};
use ultrahonk::decider::types::VerifierMemory;
use ultrahonk::decider::verifier::DeciderVerifier;
use ultrahonk::serialize::BytesDeserializable;
use ultrahonk::transcript::Transcript;
use ultrahonk::types::ScalarField;

sol_storage! {
    #[cfg_attr(feature = "shplemini-verifier", entrypoint)]
    pub struct ShpleminiVerifierContract {}
}

// Define errors that can occur during the execution of the contract
sol! {
    error TranscriptDeserializationFailed();
    error MemoryDeserializationFailed();
    error ShpleminiVerificationFailed();
    error MultivariateChallengeDeserializationFailed();
}

#[derive(SolidityError)]
pub enum VerifierErrors {
    TranscriptDeserializationFailed(TranscriptDeserializationFailed),
    MemoryDeserializationFailed(MemoryDeserializationFailed),
    ShpleminiVerificationFailed(ShpleminiVerificationFailed),
    MultivariateChallengeDeserializationFailed(MultivariateChallengeDeserializationFailed),
}

#[public]
impl ShpleminiVerifierContract {
    pub fn verify(
        &self,
        memory_bytes: Bytes,
        transcript_bytes: Bytes,
        multivariate_challenge: Bytes,
        circuit_size: u32,
    ) -> Result<bool, VerifierErrors> {
        // deserialize transcript and multivariate challenge
        let mut transcript = Transcript::deserialize_from_bytes(transcript_bytes.as_slice())
            .map_err(|_| {
                VerifierErrors::TranscriptDeserializationFailed(TranscriptDeserializationFailed {})
            })?;

        // deserialize multivariate challenge
        let multivariate_challenge = Vec::<ScalarField>::deserialize_from_bytes(
            multivariate_challenge.as_slice(),
        )
        .map_err(|_| {
            VerifierErrors::MultivariateChallengeDeserializationFailed(
                MultivariateChallengeDeserializationFailed {},
            )
        })?;

        // deserialize memory
        let memory =
            VerifierMemory::deserialize_from_bytes(memory_bytes.as_slice()).map_err(|_| {
                VerifierErrors::MemoryDeserializationFailed(MemoryDeserializationFailed {})
            })?;

        // verify shplemini
        let shplemini_ok = DeciderVerifier::new(memory)
            .verify_shplemini::<PrecompileHashBackend, PrecompileG1ArithmeticBackend>(
                &mut transcript,
                multivariate_challenge,
                circuit_size,
            )
            .map_err(|_| {
                VerifierErrors::ShpleminiVerificationFailed(ShpleminiVerificationFailed {})
            })?;

        Ok(shplemini_ok)
    }
}
