// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[cfg(any(
    feature = "e2e-backends",
))]
pub mod mocks;

pub mod utils;

#[macro_use]
extern crate alloc;
use alloc::vec::Vec;
use stylus_sdk::{abi::Bytes, prelude::*};
use ultrahonk::crs::parser::CrsParser;
use ultrahonk::keys::verification_key::{VerifyingKey, VerifyingKeyBarretenberg};
use ultrahonk::verifier::UltraHonk;
use ultrahonk::serialize::BytesDeserializable;
use ultrahonk::types::{HonkProof, ScalarField};
use crate::utils::backends::{PrecompileHashBackend, PrecompileG1ArithmeticBackend};

sol_storage! {
    #[cfg_attr(feature = "verifier", entrypoint)]
    pub struct VerifierContract {
    }
}

#[public]
impl VerifierContract {
    pub fn verify(&self, proof_bytes: Bytes, public_inputs_bytes: Bytes, vk_bytes: Bytes) -> bool {
        let proof = HonkProof::from_buffer(&proof_bytes).unwrap();

        // parse public_inputs file
        let public_inputs =
            Vec::<ScalarField>::deserialize_from_bytes(&public_inputs_bytes).unwrap();

        // parse the crs
        let verifier_crs = CrsParser::get_crs_g2().unwrap();

        // parse verification key file
        let vk = VerifyingKeyBarretenberg::from_buffer(&vk_bytes).unwrap();
        let vk = VerifyingKey::from_barrettenberg_and_crs(vk, verifier_crs);

        UltraHonk::<PrecompileG1ArithmeticBackend, PrecompileHashBackend>::verify(
            proof,
            &public_inputs,
            &vk,
        )
        .unwrap()
    }
}
