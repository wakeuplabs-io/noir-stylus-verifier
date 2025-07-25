// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use stylus_sdk::{alloy_primitives::U256, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct Voting {
    }
}

#[public]
impl Voting {
    
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_counter() {
        use stylus_sdk::testing::*;
        let vm = TestVM::default();
        let mut contract = Counter::from(&vm);

        assert_eq!(U256::ZERO, contract.number());

        contract.increment();
        assert_eq!(U256::from(1), contract.number());

        contract.add_number(U256::from(3));
        assert_eq!(U256::from(4), contract.number());

        contract.mul_number(U256::from(2));
        assert_eq!(U256::from(8), contract.number());

        contract.set_number(U256::from(100));
        assert_eq!(U256::from(100), contract.number());

        // Override the msg value for future contract method invocations.
        vm.set_value(U256::from(2));

        contract.add_from_msg_value();
        assert_eq!(U256::from(102), contract.number());
    }
}
