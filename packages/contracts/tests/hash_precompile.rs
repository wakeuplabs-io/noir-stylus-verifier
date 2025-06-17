#![cfg(feature = "hash-precompile")]

use abi::{HashPrecompileTestContract, Verifier};
use e2e::{Account, Revert};
use eyre::Result;

mod abi;

// ============================================================================
// Integration Tests: ECDSA
// ============================================================================

#[e2e::test]
async fn ecrecover_works(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = HashPrecompileTestContract::new(contract_addr, &alice.wallet);

    let HashPrecompileTestContract::demoReturn { _0: result } = contract.demo().call().await?;

    assert_eq!(true, result);

    Ok(())
}
