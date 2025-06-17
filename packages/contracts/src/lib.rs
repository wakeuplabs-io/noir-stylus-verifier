// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

pub mod mocks;
pub mod utils;

#[macro_use]
extern crate alloc;
use alloc::vec::Vec;
use stylus_sdk::prelude::*;

sol_storage! {
    #[cfg_attr(feature = "verifier", entrypoint)]
    pub struct VerifierContract {
    }
}

#[public]
impl VerifierContract {
    pub fn verify(&self) -> bool {
        true
    }

    // #[cfg(feature = "e2e")]
    // pub fn demo(&self) -> bool {
    //     true
    // }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_counter() {
        use stylus_sdk::testing::*;
        let vm = TestVM::default();
        let contract = VerifierContract::from(&vm);

        assert_eq!(true, contract.verify());
    }
}
