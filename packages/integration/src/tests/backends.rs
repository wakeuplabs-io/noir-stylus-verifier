//! Integration tests for precompile functionality

use ark_ec::AffineRepr;
use ark_ff::UniformRand;
use contracts::utils::serde_def_types::{SerdeG1Affine, SerdeG2Affine, SerdeScalarField};
use eyre::Result;
use rand::{thread_rng, RngCore};
use ultrahonk::types::{G1Affine, G2Affine, ScalarField};
use crate::{abis::PrecompileTestContract, assert_eq_result, assert_true_result, integration_test_async, utils::serialize_to_calldata, TestContext};

/// Test how the contracts call the `ecAdd` precompile
async fn test_ec_add(ctx: TestContext) -> Result<()> {
    let contract = PrecompileTestContract::new(ctx.precompiles_contract_address, ctx.client.provider());
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
    let contract = PrecompileTestContract::new(ctx.precompiles_contract_address, ctx.client.provider());
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
    let contract = PrecompileTestContract::new(ctx.precompiles_contract_address, ctx.client.provider());
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
