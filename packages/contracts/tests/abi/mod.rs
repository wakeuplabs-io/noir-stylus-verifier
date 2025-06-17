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
        function testEcAdd(bytes memory a_bytes, bytes memory b_bytes) external view returns (bytes);
    }

    #[sol(rpc)]
    contract HashPrecompileTestContract {
        #[derive(Debug)]
        function testEcAdd(bytes memory a_bytes, bytes memory b_bytes) external view returns (bytes);
    }
);
