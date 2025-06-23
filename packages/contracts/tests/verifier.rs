#![cfg(feature = "verifier")]

use abi::{Verifier};
use alloy_primitives::Bytes;
use e2e::{Account};
use eyre::Result;
use serde::Serialize;

mod abi;


#[e2e::test]
async fn test_verifier(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Verifier::new(contract_addr, &alice.wallet);

     // parse proof file
     let proof_u8 = std::fs::read("./test_vectors/add3u64/kat/proof").unwrap();
 
     // parse public_inputs file
     let public_inputs_u8 = std::fs::read("./test_vectors/add3u64/kat/public_inputs").unwrap();
 
     // parse verification key file
     let vk_u8 = std::fs::read("./test_vectors/add3u64/kat/vk").unwrap();

     println!("proof_u8: {:?}", proof_u8);
     println!("public_inputs_u8: {:?}", public_inputs_u8);
     println!("vk_u8: {:?}", vk_u8);

    let c_bytes = contract
        .verify(
            serialize_to_calldata(&proof_u8)?,
            serialize_to_calldata(&public_inputs_u8)?,
            serialize_to_calldata(&vk_u8)?,
        )
        .call()
        .await?
        ._0;

    assert_eq!(c_bytes, true);

    Ok(())
}


fn serialize_to_calldata<T: Serialize>(t: &T) -> Result<Bytes> {
    Ok(postcard::to_allocvec(t)?.into())
}
