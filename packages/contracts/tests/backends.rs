#![cfg(feature = "e2e-backends")]

use abi::{G1ArithmeticPrecompileTestContract, Verifier};
use e2e::{Account, Revert};
use eyre::Result;

mod abi;

#[e2e::test]
async fn ecrecover_works(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = G1ArithmeticPrecompileTestContract::new(contract_addr, &alice.wallet);

    let G1ArithmeticPrecompileTestContract::demoReturn { _0: result } = contract.demo().call().await?;

    assert_eq!(true, result);

    Ok(())
}
