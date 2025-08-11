#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

#[cfg(not(any(test, feature = "export-abi")))]
#[no_mangle]
pub extern "C" fn main() {}

#[cfg(feature = "export-abi")]
fn main() {
    use stylus_sdk::abi::export::print_abi;

    #[cfg(any(feature = "verifier", feature = "zk-verifier"))] {
        use contracts::contracts::core::verifier::VerifierContract;
        print_abi::<VerifierContract>("MIT-OR-APACHE-2.0", "pragma solidity ^0.8.23;");
    }

    #[cfg(any(feature = "sumcheck-verifier", feature = "zk-sumcheck-verifier"))] {
        use contracts::contracts::core::sumcheck_verifier::SumcheckVerifierContract;
        print_abi::<SumcheckVerifierContract>("MIT-OR-APACHE-2.0", "pragma solidity ^0.8.23;");
    }

    #[cfg(any(feature = "shplemini-verifier", feature = "zk-shplemini-verifier"))] {
        use contracts::contracts::core::shplemini_verifier::ShpleminiVerifierContract;
        print_abi::<ShpleminiVerifierContract>("MIT-OR-APACHE-2.0", "pragma solidity ^0.8.23;");
    }
}
