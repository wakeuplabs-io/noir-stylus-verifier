#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
   contract Verifier {
        #[derive(Debug)]
        function verify() internal pure returns (bool);
    }
);