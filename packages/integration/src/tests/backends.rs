//! Integration tests for precompile functionality

use crate::{assert_eq_result, assert_true_result, integration_test_async, TestContext};
use alloy_primitives::keccak256;
use ark_ec::AffineRepr;
use ark_ff::UniformRand;
use eyre::Result;
use rand::{thread_rng, RngCore};
use ultrahonk::serialize::{BytesDeserializable, BytesSerializable};
use ultrahonk::types::{G1Affine, G2Affine, ScalarField};

/// Test how the contracts call the `ecAdd` precompile
async fn test_ec_add(ctx: TestContext) -> Result<()> {
    let contract = ctx.precompile_test_contract();
    let mut rng = thread_rng();

    let a = G1Affine::rand(&mut rng);
    let b = G1Affine::rand(&mut rng);

    let c_bytes = contract
        .testEcAdd(a.serialize_to_bytes().into(), b.serialize_to_bytes().into())
        .call()
        .await?
        ._0;
    let c: G1Affine = G1Affine::deserialize_from_bytes(&c_bytes).unwrap();

    assert_eq_result!(c, a + b)
}
integration_test_async!(test_ec_add);

/// Test how the contracts call the `ecMul` precompile
async fn test_ec_mul(ctx: TestContext) -> Result<()> {
    let contract = ctx.precompile_test_contract();
    let mut rng = thread_rng();

    let a = ScalarField::rand(&mut rng);
    let b = G1Affine::rand(&mut rng);

    let c_bytes = contract
        .testEcMul(a.serialize_to_bytes().into(), b.serialize_to_bytes().into())
        .call()
        .await?
        ._0;
    let c: G1Affine = G1Affine::deserialize_from_bytes(&c_bytes).unwrap();

    let mut expected = b.into_group();
    expected *= a;

    assert_eq_result!(c, expected)
}
integration_test_async!(test_ec_mul);

/// Test how the contracts call the `ecPairing` precompile
async fn test_ec_pairing(ctx: TestContext) -> Result<()> {
    let contract = ctx.precompile_test_contract();
    let mut rng = thread_rng();

    let a = G1Affine::rand(&mut rng);
    let b = G2Affine::rand(&mut rng);

    let res = contract
        .testEcPairing(a.serialize_to_bytes().into(), b.serialize_to_bytes().into())
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

    let c_bytes = contract.testHash(msg.to_vec().into()).call().await?._0;

    assert_eq_result!(c_bytes, keccak256(&msg).to_vec())
}
integration_test_async!(test_hash);
