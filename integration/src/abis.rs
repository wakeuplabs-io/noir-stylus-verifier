#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
   contract VerifierContract {
        #[derive(Debug)]
        function constructor(address sumcheck_verifier_address, address shplemini_verifier_address) external;

        #[derive(Debug)]
        function verify(bytes proof_bytes, bytes public_inputs_bytes, bytes vk_bytes) external pure returns (bool);

        #[derive(Debug)]
        function getSumcheckVerifierAddress() external view returns (address);

        #[derive(Debug)]
        function getShpleminiVerifierAddress() external view returns (address);

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
        #[derive(Debug)]
        function testMsm(bytes memory a_bytes, bytes memory b_bytes) external view returns (bytes);
    }
);
