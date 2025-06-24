#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
   contract VerifierContract {
        #[derive(Debug)]
        function initialize(address sumcheck_verifier_address) external;

        #[derive(Debug)]
        function verify(bytes proof_bytes, bytes public_inputs_bytes, bytes vk_bytes) internal pure returns (bool);
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
