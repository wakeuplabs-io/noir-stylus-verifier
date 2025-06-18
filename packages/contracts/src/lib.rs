// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

pub mod utils;
use alloc::vec::Vec;

/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::prelude::*;

// Define some persistent storage using the Solidity ABI.
sol_storage! {
    #[entrypoint]
    pub struct VerifierContract {
    }
}

/// Declare that `VerifierContract` is a contract with the following external methods.
#[public]
impl VerifierContract {
    /// Gets the number from storage.
    pub fn verify(&self) -> bool {
        true
    }
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
