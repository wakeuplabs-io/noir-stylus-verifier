use crate::utils::backends::{PrecompileG1ArithmeticBackend, PrecompileHashBackend};
use alloc::vec::Vec;
use stylus_sdk::{abi::Bytes, prelude::*};
use ultrahonk::decider::types::VerifierMemory;
use ultrahonk::decider::verifier::DeciderVerifier;
use ultrahonk::serialize::BytesDeserializable;
use ultrahonk::transcript::Transcript;
use ultrahonk::types::ScalarField;

#[cfg_attr(feature = "shplemini-verifier", entrypoint)]
#[storage]
pub struct ShpleminiVerifierContract {}

#[public]
impl ShpleminiVerifierContract {
    pub fn verify(
        &self,
        memory_bytes: Bytes,
        transcript_bytes: Bytes,
        multivariate_challenge: Bytes,
        circuit_size: u32,
    ) -> bool {
        let memory = VerifierMemory::deserialize_from_bytes(memory_bytes.as_slice()).unwrap();
        let mut transcript =
            Transcript::deserialize_from_bytes(transcript_bytes.as_slice()).unwrap();
        let multivariate_challenge =
            Vec::<ScalarField>::deserialize_from_bytes(multivariate_challenge.as_slice()).unwrap();

        let mut decider_verifier = DeciderVerifier::new(memory);
        decider_verifier
            .verify_shplemini::<PrecompileHashBackend, PrecompileG1ArithmeticBackend>(
                &mut transcript,
                multivariate_challenge,
                circuit_size,
            )
            .unwrap();

        true
    }
}
