use crate::utils::helpers::call_helper;
use crate::utils::solidity::verifyCall;
use alloc::vec::Vec;
use alloy_primitives::Address;
use core::borrow::Borrow;
use stylus_sdk::call::Call;
use stylus_sdk::storage::StorageAddress;
use stylus_sdk::{abi::Bytes, prelude::*};
use ultrahonk::crs::parser::CrsParser;
use ultrahonk::keys::verification_key::{VerifyingKey, VerifyingKeyBarretenberg};
use ultrahonk::serialize::BytesDeserializable;
use ultrahonk::transcript::Transcript;
use ultrahonk::types::{HonkProof, ScalarField};

sol_storage! {
    #[cfg_attr(feature = "verifier", entrypoint)]
    pub struct VerifierContract {
        bool initialized;
        address sumcheck_verifier_address;
    }
}

sol_interface! {
    interface ISumcheckVerifier {
        function verify(bytes proof, bytes public_inputs, bytes vk) external returns (bool);
    }
}

#[public]
impl VerifierContract {
    /// The constructor sets the owner as the EOA that deployed the contract.
    pub fn initialize(&mut self, sumcheck_verifier_address: Address) {
        // if self.initialized.get() {
        //     return;
        // }

        // self.initialized.set(true);
        self.sumcheck_verifier_address.set(sumcheck_verifier_address);
    }

    pub fn get_sumcheck_verifier_address(&self) -> Address {
        self.sumcheck_verifier_address.get()
    }

    pub fn verify(
        &mut self,
        proof_bytes: Bytes,
        public_inputs_bytes: Bytes,
        vk_bytes: Bytes,
    ) -> bool {
        // let public_inputs =
        //     Vec::<ScalarField>::deserialize_from_bytes(public_inputs_bytes.as_slice()).unwrap();

        // // deserialize the proof
        // let proof = HonkProof::from_buffer(&proof_bytes).expect("Failed to deserialize proof");
        // let proof = proof.insert_public_inputs(public_inputs.to_vec());

        // // parse the crs
        // let verifier_crs = CrsParser::get_crs_g2().expect("Failed to get crs");

        // // parse verification key file
        // let vk =
        //     VerifyingKeyBarretenberg::from_buffer(&vk_bytes).expect("Failed to deserialize vk");
        // let vk = VerifyingKey::from_barrettenberg_and_crs(vk, verifier_crs);

        // let mut transcript = Transcript::new_verifier(proof);

        // let memory = VerifierMemory::from_key_and_transcript::<PrecompileG1ArithmeticBackend, PrecompileHashBackend>(&vk, &mut transcript);

        // let mut decider_verifier = DeciderVerifier::<P, H>::new(memory);

        // decider_verifier.verify_sumcheck(&verifying_key, &mut transcript)?;
        // decider_verifier.verify_shplemini(&verifying_key, &mut transcript)?;

        // https://docs.arbitrum.io/stylus-by-example/applications/multi_call

        // let result = call_helper::<verifyCall>(
        //     storage,
        //     self.sumcheck_verifier_address.get(),
        //     (
        //         proof_bytes.to_vec().into(),
        //         public_inputs_bytes.to_vec().into(),
        //         vk_bytes.to_vec().into(),
        //     ),
        // )
        // .unwrap();
        let sumcheck_verifier = ISumcheckVerifier::new(self.sumcheck_verifier_address.get());
        let config = Call::new();
        let result = sumcheck_verifier.verify(
            config,
            proof_bytes.to_vec().into(),
            public_inputs_bytes.to_vec().into(),
            vk_bytes.to_vec().into(),
        );

        result.unwrap()
    }
}

// cast call 0x32d44c68a89081d6b085bd64739256d4d05c565a -r http://localhost:8547 \
// "verify(bytes,bytes,bytes)(bool)" \
// $(cat test_vectors/poseidon/kat/proof | xxd -p -c 0) \
// $(cat test_vectors/poseidon/kat/public_inputs | xxd -p -c 0) \
// $(cat test_vectors/poseidon/kat/vk | xxd -p -c 0)

// just deploy-contract verifier "constructor(address)" "0xc09ef537e974a385bcc587e0e00f683dab4ba4b0"

// cast call 0x5be6cc8b8def1ca2335a504b0257536f24fac049 "getSumcheckVerifierAddress()(address)" -r http://localhost:8547