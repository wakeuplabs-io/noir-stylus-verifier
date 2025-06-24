use crate::utils::backends::{PrecompileG1ArithmeticBackend, PrecompileHashBackend};
use alloc::vec::Vec;
use stylus_sdk::{abi::Bytes, prelude::*};
use ultrahonk::crs::parser::CrsParser;
use ultrahonk::keys::verification_key::{VerifyingKey, VerifyingKeyBarretenberg};
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
    pub fn verify(&self, proof_bytes: Bytes, public_inputs_bytes: Bytes, vk_bytes: Bytes) -> bool {
        // let public_inputs = Vec::<ScalarField>::deserialize_from_bytes(public_inputs_bytes.as_slice()).unwrap();

        // // deserialize the proof
        // let proof = HonkProof::from_buffer(&proof_bytes).expect("Failed to deserialize proof");
        // let proof = proof.insert_public_inputs(public_inputs.to_vec());

        // // parse the crs
        // let verifier_crs = CrsParser::get_crs_g2().expect("Failed to get crs");

        // // parse verification key file
        // // TODO: collapse these 2 into one
        // let vk = VerifyingKeyBarretenberg::from_buffer(&vk_bytes).expect("Failed to deserialize vk");
        // let vk = VerifyingKey::from_barrettenberg_and_crs(vk, verifier_crs); 

        // let decider_verifier =
        //     DeciderVerifier::<P, H>::new(VerifierMemory::from_key_and_transcript::<P, H>(
        //         &vk,
        //         &mut Transcript::new_verifier(proof.clone()),
        //     ));
        // decider_verifier.verify_sumcheck(&mut transcript, verifying_key.circuit_size)?;

        true
    }
}
