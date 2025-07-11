extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::Address;
use stylus_sdk::{abi::Bytes, prelude::*};

#[allow(deprecated)]
use stylus_sdk::call::Call as InterfaceCall;

sol_storage! {
    #[entrypoint]
    pub struct VerifierContract {
        address verifier_address;
    }
}

sol_interface! {
    interface IGlobalVerifier {
        function verify(bytes memory proof, bytes memory public_inputs, bytes memory vk) external returns (bool);
    }
}

#[public]
impl VerifierContract {
    #[constructor]
    pub fn constructor(&mut self, verifier_address: Address) {
        self.verifier_address.set(verifier_address);
    }

    pub fn verify(&mut self, proof_bytes: Bytes, y: Bytes, z: Bytes) -> bool {
        IGlobalVerifier::new(self.verifier_address.get()).verify(
            #[allow(deprecated)]
            InterfaceCall::new(),
            proof_bytes.to_vec().into(),
            [y.to_vec(), z.to_vec()].concat().into(),
            include_bytes!("../../circuit/target/vk").to_vec().into(),
        ).unwrap_or(false)
    }

    pub fn get_verifier_address(&self) -> Address {
        self.verifier_address.get()
    }
}
