use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;

const BIN: &str = env!("CARGO_PKG_NAME");
const PROJECT_NAME: &str = "hello_world";

#[test]
fn happy_path() {
    let project_dir = assert_fs::TempDir::new().unwrap();
    let project_dir = project_dir.child(PROJECT_NAME);
    project_dir
        .copy_from("tests/fixtures/hello_world", &["**/*"])
        .unwrap();
    fs::remove_dir_all(project_dir.child("contracts").path()).unwrap();

    // run command
    let mut cmd = Command::cargo_bin(BIN).unwrap();
    cmd.arg("generate")
        .current_dir(project_dir.path())
        .assert()
        .success();

    // check that the contract files exist
    project_dir
        .child("contracts")
        .assert(predicate::path::exists());
    project_dir
        .child("contracts/src/lib.rs")
        .assert(predicate::path::exists());
    project_dir
        .child("contracts/src/main.rs")
        .assert(predicate::path::exists());
    project_dir
        .child("contracts/Cargo.toml")
        .assert(predicate::path::exists());
    project_dir
        .child("contracts/Cargo.lock")
        .assert(predicate::path::exists());
    project_dir
        .child("contracts/rust-toolchain.toml")
        .assert(predicate::path::exists());

    // check that we copied the vk and bytecode files
    project_dir
        .child("contracts/assets/vk")
        .assert(predicate::path::exists());
    project_dir
        .child("contracts/assets/bytecode.json")
        .assert(predicate::path::exists());
}

#[test]
fn happy_path_with_bytecode_and_vk() {
    let project_dir = assert_fs::TempDir::new().unwrap();
    let project_dir = project_dir.child(PROJECT_NAME);
    project_dir
        .copy_from("tests/fixtures/hello_world", &["**/*"])
        .unwrap();
    fs::remove_dir_all(project_dir.child("contracts").path()).unwrap();

    // run command
    let mut cmd = Command::cargo_bin(BIN).unwrap();
    cmd.arg("generate")
        .arg("--bytecode-path")
        .arg("kat/bytecode.json")
        .arg("--vk-path")
        .arg("kat/vk")
        .current_dir(project_dir.path())
        .assert()
        .success();

    // check that the contract files exist
    project_dir
        .child("contracts")
        .assert(predicate::path::exists());
    project_dir
        .child("contracts/src/lib.rs")
        .assert(predicate::path::exists());

    // also that we used the provided bytecode and vk, meaning no `target` folder was created as no compilation was done but we still copied the vk and bytecode files
    project_dir
        .child("target")
        .assert(predicate::path::missing());
    project_dir
        .child("contracts/assets/bytecode.json")
        .assert(predicate::path::exists());
    project_dir
        .child("contracts/assets/vk")
        .assert(predicate::path::exists());
}

#[test]
fn provided_bytecode_and_vk_files_do_not_exist() {
    let project_dir = assert_fs::TempDir::new().unwrap();
    let project_dir = project_dir.child(PROJECT_NAME);
    project_dir
        .copy_from("tests/fixtures/hello_world", &["**/*"])
        .unwrap();
    fs::remove_dir_all(project_dir.child("contracts").path()).unwrap();

    // run command
    let mut cmd = Command::cargo_bin(BIN).unwrap();
    cmd.arg("generate")
        .arg("--bytecode-path")
        .arg("kat/NOT_EXISTING_BYTECODE.json")
        .arg("--vk-path")
        .arg("kat/NOT_EXISTING_VK")
        .current_dir(project_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("File not found"));

    project_dir
        .child("contracts")
        .assert(predicate::path::missing());
}
