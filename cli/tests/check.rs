use std::fs;

use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

const BIN: &str = env!("CARGO_PKG_NAME");
const PROJECT_NAME: &str = "my-project";
const RPC_URL: &str = "http://localhost:8547";

#[test]
fn happy_path() {
    let project_dir = assert_fs::TempDir::new().unwrap();

    let project_dir = project_dir.child(PROJECT_NAME);
    project_dir
        .copy_from("tests/fixtures/hello_world", &["**/*"])
        .unwrap();

    let mut cmd = Command::cargo_bin(BIN).unwrap();
    cmd.arg("check")
        .arg("--rpc-url")
        .arg(RPC_URL)
        .current_dir(project_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("contract size"))
        .stdout(predicate::str::contains("wasm size"));
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
