#![cfg(feature = "e2e-backends")]

use core::panic;
use abi::{G1ArithmeticPrecompileTestContract, Verifier};
use e2e::{Account, Revert};
use eyre::Result;

mod abi;

#[e2e::test]
async fn test_ec_add(ctx: TestContext) -> Result<()> {
    panic!("test_ec_add once deployment is possible");
}

#[e2e::test]
async fn test_ec_mul(ctx: TestContext) -> Result<()> {
    panic!("test_ec_mul once deployment is possible");
}

#[e2e::test]
async fn test_ec_pairing(ctx: TestContext) -> Result<()> {
    panic!("test_ec_pairing once deployment is possible");
}

#[e2e::test]
async fn test_hash(ctx: TestContext) -> Result<()> {
    panic!("test_hash once deployment is possible");
}
