use crate::utils::backends::{PrecompileG1ArithmeticBackend, PrecompileHashBackend};
use alloc::vec::Vec;
use stylus_sdk::{abi::Bytes, prelude::*};
use ultrahonk::decider::types::VerifierMemory;
use ultrahonk::decider::verifier::DeciderVerifier;
// use ultrahonk::decider::types::VerifierMemory;
// use ultrahonk::decider::verifier::DeciderVerifier;
use ultrahonk::keys::verification_key::VerifyingKey;
use ultrahonk::serialize::BytesDeserializable;
use ultrahonk::transcript::Transcript;
use ultrahonk::types::{HonkProof, ScalarField};

sol_storage! {
    #[cfg_attr(feature = "shplemini-verifier", entrypoint)]
    pub struct ShpleminiVerifierContract {
    }
}

#[public]
impl ShpleminiVerifierContract {
    pub fn verify(&self, memory_bytes: Bytes, transcript_bytes: Bytes, circuit_size: u32) -> bool {
        // let public_inputs =
        //     Vec::<ScalarField>::deserialize_from_bytes(public_inputs_bytes.as_slice()).unwrap();

        // // deserialize the proof
        // let proof = HonkProof::from_buffer(&proof_bytes).expect("Failed to deserialize proof");
        // let proof = proof.insert_public_inputs(public_inputs.to_vec());

        // // parse verification key file
        // let vk = VerifyingKey::from_buffer(&vk_bytes).expect("Failed to deserialize vk");

        // let mut transcript = Transcript::new_verifier(proof.clone());

        // let mut decider_verifier =
        //     DeciderVerifier::<PrecompileG1ArithmeticBackend, PrecompileHashBackend>::new(
        //         VerifierMemory::from_key_and_transcript::<
        //             PrecompileG1ArithmeticBackend,
        //             PrecompileHashBackend,
        //         >(&vk, &mut Transcript::new_verifier(proof.clone())),
        //     );
        // decider_verifier
        //     .verify_shplemini(&mut transcript, vk.circuit_size)
        //     .unwrap();

        let memory = VerifierMemory::deserialize_from_bytes(memory_bytes.as_slice()).unwrap();
        let mut transcript = Transcript::deserialize_from_bytes(transcript_bytes.as_slice()).unwrap();

        let mut decider_verifier = DeciderVerifier::<PrecompileG1ArithmeticBackend, PrecompileHashBackend>::new(memory);

        decider_verifier.verify_shplemini(circuit_size, &mut transcript).unwrap();

        true
    }
}
