//! Integration tests for precompile functionality

use crate::{
    assert_eq_result, assert_true_result, integration_test_async, utils::serialize_to_calldata,
    TestContext,
};
use alloy::hex;
use alloy_primitives::keccak256;
use alloy_sol_types::ContractError;
use ark_bn254::Bn254;
use ark_ec::VariableBaseMSM;
use ark_ec::{pairing::Pairing, AffineRepr};
use ark_ff::UniformRand;
use contracts::utils::serde_def_types::{SerdeG1Affine, SerdeG2Affine, SerdeScalarField};
use eyre::Result;
use rand::{thread_rng, RngCore};
use ultrahonk::serialize::BytesSerializable;
use ultrahonk::types::{G1Affine, G2Affine, ScalarField};

/// Test how the contracts call the `ecAdd` precompile
async fn test_ec_add(ctx: TestContext) -> Result<()> {
    let contract = ctx.precompile_test_contract();
    let mut rng = thread_rng();

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

    assert_eq_result!(c.0, a + b)
}
integration_test_async!(test_ec_add);

/// Test how the contracts call the `ecMul` precompile
async fn test_ec_mul(ctx: TestContext) -> Result<()> {
    let contract = ctx.precompile_test_contract();
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

    assert_eq_result!(c.0, expected)
}
integration_test_async!(test_ec_mul);

/// Test how the contracts call the `ecPairing` precompile
async fn test_ec_pairing(ctx: TestContext) -> Result<()> {
    let contract = ctx.precompile_test_contract();
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

    assert_true_result!(res)
}
integration_test_async!(test_ec_pairing);

async fn test_hash(ctx: TestContext) -> Result<()> {
    let contract = ctx.precompile_test_contract();
    let mut rng = thread_rng();

    let mut msg = [0u8; 32];
    rng.fill_bytes(&mut msg);

    let c_bytes = contract
        .testHash(serialize_to_calldata(&msg)?)
        .call()
        .await?
        ._0;

    assert_eq_result!(c_bytes, keccak256(&msg).to_vec())
}
integration_test_async!(test_hash);

async fn test_msm(ctx: TestContext) -> Result<()> {
    let contract = ctx.precompile_test_contract();
    let mut rng = thread_rng();

    let a = ScalarField::from(42u64);
    let b = G1Affine::generator();

    println!("a: {:?}", hex::encode(a.serialize_to_bytes()));
    println!("b: {:?}", hex::encode(b.serialize_to_bytes()));

    let expected = <Bn254 as Pairing>::G1::msm_unchecked(&[b], &[a]);

    let err = contract
        .testMsm(
            serialize_to_calldata(&[SerdeScalarField(a)])?,
            serialize_to_calldata(&[SerdeG1Affine(b)])?,
        )
        .call()
        .await.unwrap();

    println!("Decoded as: {:?}", err);

    // let c_bytes = c_bytes.unwrap().await?._0;

    // let c: SerdeG1Affine = postcard::from_bytes(&c_bytes)?;

    // assert_eq_result!(c.0, expected)
    Ok(())
}
integration_test_async!(test_msm);
