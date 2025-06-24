use crate::utils::backends::{PrecompileG1ArithmeticBackend, PrecompileHashBackend};
use alloc::vec::Vec;
use stylus_sdk::{abi::Bytes, prelude::*};
use ultrahonk::crs::parser::CrsParser;
use ultrahonk::keys::verification_key::{VerifyingKey, VerifyingKeyBarretenberg};
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
    pub fn verify(&self, proof_bytes: Bytes, public_inputs_bytes: Bytes, vk_bytes: Bytes) -> bool {
        // let public_inputs = Vec::<ScalarField>::deserialize_from_bytes(public_inputs_bytes.as_slice()).unwrap();

        // // deserialize the proof
        // let proof = HonkProof::from_buffer(&proof_bytes).expect("Failed to deserialize proof");
        // let proof = proof.insert_public_inputs(public_inputs.to_vec());

        // // parse the crs
        // let verifier_crs = CrsParser::get_crs_g2().expect("Failed to get crs");

        // // parse verification key file
        // let vk = VerifyingKeyBarretenberg::from_buffer(&vk_bytes).expect("Failed to deserialize vk");
        // let vk = VerifyingKey::from_barrettenberg_and_crs(vk, verifier_crs);

        // let mut transcript = Transcript::new_verifier(proof);

        // let memory = VerifierMemory::from_key_and_transcript::<PrecompileG1ArithmeticBackend, PrecompileHashBackend>(&vk, &mut transcript);

        // let mut decider_verifier = DeciderVerifier::<P, H>::new(memory);

        // decider_verifier.verify_sumcheck(&verifying_key, &mut transcript)?;
        // decider_verifier.verify_shplemini(&verifying_key, &mut transcript)?;

        true
    }
}