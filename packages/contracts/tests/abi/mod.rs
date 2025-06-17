#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
   contract Verifier {
        #[derive(Debug)]
        function verify() internal pure returns (bool);
    }

    #[sol(rpc)]
    contract G1ArithmeticPrecompileTestContract {
        #[derive(Debug)]
        function demo() internal pure returns (bool);
    }

    #[sol(rpc)]
    contract HashPrecompileTestContract {
        #[derive(Debug)]
        function demo() internal pure returns (bool);
    }
);

