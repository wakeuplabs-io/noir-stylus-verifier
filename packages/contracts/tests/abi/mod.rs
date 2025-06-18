#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
   contract Verifier {
        #[derive(Debug)]
        function verify() internal pure returns (bool);
    }

    #[sol(rpc)]
    contract PrecompileTestContract {
        #[derive(Debug)]
        function testEcAdd(bytes memory a_bytes, bytes memory b_bytes) external view returns (bytes);
        #[derive(Debug)]
        function testEcMul(bytes memory a_bytes, bytes memory b_bytes) external view returns (bytes);
        #[derive(Debug)]
        function testEcPairing(bytes memory a_bytes, bytes memory b_bytes) external view returns (bool);
        #[derive(Debug)]
        function testHash(bytes memory a_bytes) external view returns (bytes);
    }

);
