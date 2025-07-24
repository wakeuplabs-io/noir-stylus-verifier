use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;

const BIN: &str = env!("CARGO_PKG_NAME");
const PROJECT_NAME: &str = "my-project";

#[test]
fn happy_path() {
    let project_dir = assert_fs::TempDir::new().unwrap();
    let project_dir = project_dir.child(PROJECT_NAME);
    project_dir.copy_from("tests/fixtures/hello_world", &["**/*"]).unwrap();
    fs::remove_dir_all(project_dir.child("contracts").path()).unwrap();

    // run command
    let mut cmd = Command::cargo_bin(BIN).unwrap();
    cmd.arg("generate")
        .current_dir(project_dir.path())
        .assert()
        .success();

    // check that the contract files exist
    project_dir.child("contracts").assert(predicate::path::exists());
    project_dir.child("contracts/src/lib.rs").assert(predicate::path::exists());
    project_dir.child("contracts/src/main.rs").assert(predicate::path::exists());
    project_dir.child("contracts/Cargo.toml").assert(predicate::path::exists());
    project_dir.child("contracts/Cargo.lock").assert(predicate::path::exists());
    project_dir.child("contracts/rust-toolchain.toml").assert(predicate::path::exists());

    // check that we copied the vk and bytecode files
    project_dir.child("contracts/assets/vk").assert(predicate::path::exists());
    project_dir.child("contracts/assets/bytecode.json").assert(predicate::path::exists());
}

// TODO: pass in bytecode and vk