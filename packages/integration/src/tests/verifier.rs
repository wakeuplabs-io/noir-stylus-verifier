//! Integration tests for precompile functionality

use crate::{
    assert_true_result, constants::MANIFEST_DIR_ENV_VAR, integration_test_async, TestContext,
};
use eyre::Result;
use std::env;

macro_rules! generate_tests {
    ($($name:ident),* $(,)?) => {
        $(
            async fn $name(ctx: TestContext) -> Result<()> {
                // build path to test vector data
                let workspace_path = std::path::Path::new(&env::var(MANIFEST_DIR_ENV_VAR).unwrap())
                    .ancestors()
                    .nth(2) // Go up 2 levels: integration/ -> packages/ -> workspace root
                    .unwrap()
                    .to_path_buf();
                let test_vector_path = workspace_path.join("test_vectors").join(stringify!($name));
                let proof_file = test_vector_path.join("kat/proof");
                let vk_file = test_vector_path.join("kat/vk");
                let public_inputs_file = test_vector_path.join("kat/public_inputs");

                let proof_u8 = std::fs::read(proof_file)?;
                let public_inputs_u8 = std::fs::read(public_inputs_file)?;
                let vk_u8 = std::fs::read(vk_file)?;

                let is_valid = ctx.verifier_contract()
                .verify(proof_u8.into(), public_inputs_u8.into(), vk_u8.into())
                .call()
                .await?._0;

                assert_true_result!(is_valid)
            }
            integration_test_async!($name);
        )*
    };
}

// Run this to generate the tests:
// echo "generate_tests!(" && for d in ./test_vectors/*; do [ -d "$d" ] && name=$(basename "$d" | sed 's/__*/_/g') && echo "    $name," ; done && echo ");"
generate_tests!(
    add3u64,
    addition_multiplication,
    approx_sigmoid,
    assert,
    bb_sha256_compression,
    get_bytes,
    if_then,
    negative,
    poseidon,
    poseidon_assert,
    poseidon_input2,
    poseidon_stdlib,
    poseidon2,
    quantized,
    random_access,
    slice,
    to_radix32,
    unconstrained_fn,
    unconstrained_fn_field,
    write_access,
);
