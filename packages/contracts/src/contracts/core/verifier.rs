use crate::utils::backends::{PrecompileG1ArithmeticBackend, PrecompileHashBackend};
use alloc::vec::Vec;
use stylus_sdk::{abi::Bytes, prelude::*};
use ultrahonk::crs::parser::CrsParser;
use ultrahonk::keys::verification_key::{VerifyingKey, VerifyingKeyBarretenberg};
use ultrahonk::serialize::BytesDeserializable;
use ultrahonk::types::{HonkProof, ScalarField};
use ultrahonk::verifier::UltraHonk;

sol_storage! {
    #[cfg_attr(feature = "verifier", entrypoint)]
    pub struct VerifierContract {
    }
}

#[public]
impl VerifierContract {
    pub fn verify(&self, proof_bytes: Bytes, public_inputs_bytes: Bytes, vk_bytes: Bytes) -> bool {
        // let proof = HonkProof::from_buffer(&proof_bytes).unwrap();

        // let public_inputs =
        //     Vec::<ScalarField>::deserialize_from_bytes(&public_inputs_bytes).unwrap();

        // // parse the crs
        // let verifier_crs = CrsParser::get_crs_g2().unwrap();

        // // parse verification key file
        // let vk = VerifyingKeyBarretenberg::from_buffer(&vk_bytes).unwrap();
        // let vk = VerifyingKey::from_barrettenberg_and_crs(vk, verifier_crs);

        // UltraHonk::<PrecompileG1ArithmeticBackend, PrecompileHashBackend>::verify_2(
        //     proof,
        //     &public_inputs,
        //     &vk,
        // )
        // .unwrap();

        true
    }
}
