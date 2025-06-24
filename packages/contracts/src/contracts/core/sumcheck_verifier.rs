use crate::utils::backends::{PrecompileG1ArithmeticBackend, PrecompileHashBackend};
use alloc::vec::Vec;
use stylus_sdk::{abi::Bytes, prelude::*};
use ultrahonk::decider::types::VerifierMemory;
use ultrahonk::decider::verifier::DeciderVerifier;
use ultrahonk::serialize::BytesDeserializable;
use ultrahonk::transcript::Transcript;
use ultrahonk::types::{HonkProof, ScalarField};

sol_storage! {
    #[cfg_attr(feature = "sumcheck-verifier", entrypoint)]
    pub struct SumcheckVerifierContract {
    }
}

#[public]
impl SumcheckVerifierContract {
    pub fn verify(&self, memory_bytes: Bytes, transcript_bytes: Bytes, circuit_size: u32) -> bool {

        let memory = VerifierMemory::deserialize_from_bytes(memory_bytes.as_slice()).unwrap();
        let mut transcript = Transcript::deserialize_from_bytes(transcript_bytes.as_slice()).unwrap();

        let mut decider_verifier = DeciderVerifier::<PrecompileG1ArithmeticBackend, PrecompileHashBackend>::new(memory);

        decider_verifier.verify_sumcheck(circuit_size, &mut transcript).unwrap();

        true
    }
}
