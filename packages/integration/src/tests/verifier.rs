//! Integration tests for precompile functionality

use std::{env, path::PathBuf};

use crate::{
    assert_true_result, constants::MANIFEST_DIR_ENV_VAR, integration_test_async, TestContext,
};
use eyre::Result;

/// Test how the contracts call the `ecAdd` precompile
async fn test_verifier(ctx: TestContext) -> Result<()> {
    let contract = ctx.verifier_contract();

    let current_dir = PathBuf::from(env::var(MANIFEST_DIR_ENV_VAR).unwrap());
    let workspace_path = current_dir.ancestors().nth(2).unwrap();

    let test_vectors_dir = workspace_path.join("test_vectors");
    let test_vectors = std::fs::read_dir(test_vectors_dir).unwrap();

    for entry in test_vectors {
        let entry = entry.unwrap();
        let path = entry.path();
        if !path.is_dir() {
            // Skip if not a directory
            continue;
        }

        let proof_file = format!("{}/kat/proof", path.display());
        let vk_file = format!("{}/kat/vk", path.display());
        let public_inputs_file = format!("{}/kat/public_inputs", path.display());

        let proof_u8 = std::fs::read(proof_file).unwrap();
        let public_inputs_u8 = std::fs::read(public_inputs_file).unwrap();
        let vk_u8 = std::fs::read(vk_file).unwrap();

        let res = match contract
            .verify(proof_u8.into(), public_inputs_u8.into(), vk_u8.into())
            .call()
            .await {
                Ok(result) => result._0,
                Err(e) => {
                    println!("Verification error: {}", e);
                    false
                }
            };

        assert_true_result!(res);
    }

    Ok(())
}
integration_test_async!(test_verifier);
