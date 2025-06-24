use crate::utils::backends::{PrecompileG1ArithmeticBackend, PrecompileHashBackend};
use alloc::vec::Vec;
use stylus_sdk::{abi::Bytes, prelude::*};
use ultrahonk::decider::types::VerifierMemory;
use ultrahonk::decider::verifier::DeciderVerifier;
use ultrahonk::serialize::{BytesDeserializable, BytesSerializable};
use ultrahonk::transcript::Transcript;

sol_storage! {
    #[cfg_attr(feature = "shplemini-verifier", entrypoint)]
    pub struct ShpleminiVerifierContract {
    }
}

#[public]
impl ShpleminiVerifierContract {
    pub fn verify(&self, memory_bytes: Bytes, transcript_bytes: Bytes, circuit_size: u32) -> (bool, Bytes, Bytes) {
        let memory = VerifierMemory::deserialize_from_bytes(memory_bytes.as_slice()).unwrap();
        let mut transcript = Transcript::deserialize_from_bytes(transcript_bytes.as_slice()).unwrap();

        let mut decider_verifier =
            DeciderVerifier::<PrecompileG1ArithmeticBackend, PrecompileHashBackend>::new(memory);

        decider_verifier
            .verify_shplemini(circuit_size, &mut transcript)
            .unwrap();

        (
            true,
            decider_verifier.memory.serialize_to_bytes().into(),
            transcript.serialize_to_bytes().into(),
        )
    }
}
