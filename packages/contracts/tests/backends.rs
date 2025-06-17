#![cfg(feature = "e2e-backends")]

use abi::{G1ArithmeticPrecompileTestContract, Verifier};
use alloy_primitives::keccak256;
use alloy_primitives::Bytes;
use ark_ec::AffineRepr;
use ark_ff::UniformRand;
use contracts::utils::serde_def_types::{SerdeG1Affine, SerdeG2Affine, SerdeScalarField};
use core::panic;
use e2e::{Account, Revert};
use eyre::Result;
use rand::{thread_rng, RngCore};
use serde::Serialize;
use std::assert_eq;
use ultrahonk::{
    serialize::{BytesDeserializable, BytesSerializable},
    types::{G1Affine, G2Affine, ScalarField},
};

mod abi;

#[e2e::test]
async fn test_ec_add(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let mut rng = thread_rng();
    let contract = G1ArithmeticPrecompileTestContract::new(contract_addr, &alice.wallet);

    let a = G1Affine::rand(&mut rng);
    let b = G1Affine::rand(&mut rng);

    let c_bytes = contract
        .testEcAdd(
            serialize_to_calldata(&SerdeG1Affine(a))?,
            serialize_to_calldata(&SerdeG1Affine(b))?,
        )
        .call()
        .await?
        ._0;
    let c: SerdeG1Affine = postcard::from_bytes(&c_bytes)?;

    assert_eq!(c.0, a + b);

    Ok(())
}

#[e2e::test]
async fn test_ec_mul(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = G1ArithmeticPrecompileTestContract::new(contract_addr, &alice.wallet);
    let mut rng = thread_rng();

    let a = ScalarField::rand(&mut rng);
    let b = G1Affine::rand(&mut rng);

    let c_bytes = contract
        .testEcMul(
            serialize_to_calldata(&SerdeScalarField(a))?,
            serialize_to_calldata(&SerdeG1Affine(b))?,
        )
        .call()
        .await?
        ._0;
    let c: SerdeG1Affine = postcard::from_bytes(&c_bytes)?;

    let mut expected = b.into_group();
    expected *= a;

    assert_eq!(c.0, expected);

    Ok(())
}

#[e2e::test]
async fn test_ec_pairing(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = G1ArithmeticPrecompileTestContract::new(contract_addr, &alice.wallet);
    let mut rng = thread_rng();

    let a = G1Affine::rand(&mut rng);
    let b = G2Affine::rand(&mut rng);

    let res = contract
        .testEcPairing(
            serialize_to_calldata(&SerdeG1Affine(a))?,
            serialize_to_calldata(&SerdeG2Affine(b))?,
        )
        .call()
        .await?
        ._0;

    assert!(res);

    Ok(())
}

#[e2e::test]
async fn test_hash(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = G1ArithmeticPrecompileTestContract::new(contract_addr, &alice.wallet);
    let mut rng = thread_rng();

    let mut msg = [0u8; 32];
    rng.fill_bytes(&mut msg);

    let c_bytes = contract
        .testHash(serialize_to_calldata(&msg)?)
        .call()
        .await?
        ._0;

    assert_eq!(c_bytes, keccak256(&msg).to_vec());

    Ok(())
}

pub fn serialize_to_calldata<T: Serialize>(t: &T) -> Result<Bytes> {
    Ok(postcard::to_allocvec(t)?.into())
}
