use std::fs;

use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

const BIN: &str = env!("CARGO_PKG_NAME");
const PROJECT_NAME: &str = "my-project";

#[test]
fn happy_path() {
    let project_dir = assert_fs::TempDir::new().unwrap();
    let project_dir = project_dir.child(PROJECT_NAME);
    project_dir.copy_from("tests/fixtures/hello_world", &["**/*"]).unwrap();

    let mut cmd = Command::cargo_bin(BIN).unwrap();
    cmd.arg("prove")
        .current_dir(project_dir.path())
        .assert()
        .success();

    // check that the proof files exist
    project_dir.child("target/proof").assert(predicate::path::exists());
    project_dir.child("target/public_inputs").assert(predicate::path::exists());
}

#[test]
fn happy_path_zk() {
    let project_dir = assert_fs::TempDir::new().unwrap();
    let project_dir = project_dir.child(PROJECT_NAME);
    project_dir.copy_from("tests/fixtures/hello_world", &["**/*"]).unwrap();

    let mut cmd = Command::cargo_bin(BIN).unwrap();
    cmd.arg("prove")
        .arg("--zk")
        .current_dir(project_dir.path())
        .assert()
        .success();

    // check that the proof files exist
    project_dir.child("target/proof").assert(predicate::path::exists());
    project_dir.child("target/public_inputs").assert(predicate::path::exists());
}

// TODO: pass in witness and bytecode