use crate::utils::backends::{PrecompileG1ArithmeticBackend, PrecompileHashBackend};
use alloc::vec::Vec;
use stylus_sdk::{abi::Bytes, prelude::*};
use ultrahonk::decider::types::VerifierMemory;
use ultrahonk::decider::verifier::DeciderVerifier;
use ultrahonk::serialize::{BytesDeserializable, BytesSerializable};
use ultrahonk::transcript::Transcript;
use ultrahonk::types::ScalarField;

sol_storage! {
    #[cfg_attr(feature = "shplemini-verifier", entrypoint)]
    pub struct ShpleminiVerifierContract {
    }
}

#[public]
impl ShpleminiVerifierContract {
    pub fn verify(
        &self,
        memory_bytes: Bytes,
        transcript_bytes: Bytes,
        multivariate_challenge: Bytes,
        circuit_size: u32,
    ) -> (Bytes, Bytes, bool) {
        let memory = VerifierMemory::deserialize_from_bytes(memory_bytes.as_slice()).unwrap();
        let mut transcript =
            Transcript::deserialize_from_bytes(transcript_bytes.as_slice()).unwrap();
        let multivariate_challenge =
            Vec::<ScalarField>::deserialize_from_bytes(multivariate_challenge.as_slice()).unwrap();

        let mut decider_verifier =
            DeciderVerifier::<PrecompileG1ArithmeticBackend, PrecompileHashBackend>::new(memory);

        let shplemini_output = decider_verifier
            .verify_shplemini(&mut transcript, multivariate_challenge, circuit_size)
            .unwrap();

        (
            decider_verifier.memory.serialize_to_bytes().into(),
            transcript.serialize_to_bytes().into(),
            shplemini_output,
        )
    }
}
