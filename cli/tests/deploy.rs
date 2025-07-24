use std::fs;

use alloy::{providers::ProviderBuilder, sol};
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use regex::Regex;
use strip_ansi_escapes::strip;

const BIN: &str = env!("CARGO_PKG_NAME");
const PROJECT_NAME: &str = "my-project";
const RPC_URL: &str = "http://localhost:8547";
const PRIVATE_KEY: &str = "0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659";
const GLOBAL_VERIFIER_ADDRESS: &str = "0x0000000000000000000000000000000000000001";

sol! {
   #[sol(rpc)]
   contract Verifier {
        function getVerifierAddress() public view returns (address);
   }
}

#[tokio::test]
async fn happy_path() {
    let project_dir = assert_fs::TempDir::new().unwrap();

    let project_dir = project_dir.child(PROJECT_NAME);
    project_dir
        .copy_from("tests/fixtures/hello_world", &["**/*"])
        .unwrap();

    let mut cmd = Command::cargo_bin(BIN).unwrap();
    cmd.arg("deploy")
        .arg("--rpc-url")
        .arg(RPC_URL)
        .arg("--private-key")
        .arg(PRIVATE_KEY)
        .arg("--verifier-address")
        .arg(GLOBAL_VERIFIER_ADDRESS)
        .current_dir(project_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("deployed code at address"))
        .stdout(predicate::str::contains("deployment tx hash"));

    // extract contract address from output
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stripped = strip(stdout.as_bytes()).unwrap();
    let clean_stdout = String::from_utf8_lossy(&stripped);
    let re = Regex::new(r"deployed code at address:\s*(0x[a-fA-F0-9]+)").unwrap();
    let contract_address = re
        .captures(&clean_stdout)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str())
        .expect("Could not extract contract address from output");

    // verify global verifier address is correctly set
    let provider = ProviderBuilder::new().on_http(RPC_URL.parse().unwrap());
    let result = Verifier::new(contract_address.parse().unwrap(), provider)
        .getVerifierAddress()
        .call()
        .await
        .unwrap();
    assert_eq!(result._0.to_string(), GLOBAL_VERIFIER_ADDRESS);
}

#[test]
fn missing_contract() {
    let project_dir = assert_fs::TempDir::new().unwrap();
    let project_dir = project_dir.child(PROJECT_NAME);
    project_dir
        .copy_from("tests/fixtures/hello_world", &["**/*"])
        .unwrap();
    fs::remove_dir_all(project_dir.child("contracts").path()).unwrap();

    let mut cmd = Command::cargo_bin(BIN).unwrap();
    cmd.arg("check")
        .arg("--rpc-url")
        .arg(RPC_URL)
        .current_dir(project_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("We can\'t find your contracts"))
        .stderr(predicate::str::contains(
            "If they exist, try specifying the package with -p <package> or run nsv generate first",
        ));
}
