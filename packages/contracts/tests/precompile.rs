#![cfg(feature = "e2e")]

use abi::Verifier;
// use alloy::primitives::{address, b256, uint, Address, B256};
use e2e::{Account, Revert};
use eyre::Result;

mod abi;

// ============================================================================
// Integration Tests: ECDSA
// ============================================================================

#[e2e::test]
async fn ecrecover_works(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Verifier::new(contract_addr, &alice.wallet);

    let Verifier::verifyReturn { _0: result } = contract.verify().call().await?;

    assert_eq!(true, result);

    Ok(())
}
