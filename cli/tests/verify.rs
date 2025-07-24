use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

const BIN: &str = env!("CARGO_PKG_NAME");
const PROJECT_NAME: &str = "hello_world";

#[test]
fn happy_path_local() {
    let project_dir = assert_fs::TempDir::new().unwrap();

    let project_dir = project_dir.child(PROJECT_NAME);
    project_dir
        .copy_from("tests/fixtures/hello_world", &["**/*"])
        .unwrap();

    let mut cmd = Command::cargo_bin(BIN).unwrap();
    cmd.arg("verify")
        .arg("--proof")
        .arg("kat/proof")
        .arg("--public-input")
        .arg("kat/public_inputs")
        .arg("--vk")
        .arg("kat/vk")
        .current_dir(project_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "✅ Success! Proof verified locally",
        ));
}

#[test]
fn happy_path_local_zk() {
    let project_dir = assert_fs::TempDir::new().unwrap();

    let project_dir = project_dir.child(PROJECT_NAME);
    project_dir
        .copy_from("tests/fixtures/hello_world", &["**/*"])
        .unwrap();

    let mut cmd = Command::cargo_bin(BIN).unwrap();
    cmd.arg("verify")
        .arg("--proof")
        .arg("kat/proof_zk")
        .arg("--public-input")
        .arg("kat/public_inputs")
        .arg("--vk")
        .arg("kat/vk")
        .arg("--zk")
        .current_dir(project_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "✅ Success! Proof verified locally",
        ));
}

#[test]
fn missing_proof() {
    let project_dir = assert_fs::TempDir::new().unwrap();
    let project_dir = project_dir.child(PROJECT_NAME);
    project_dir
        .copy_from("tests/fixtures/hello_world", &["**/*"])
        .unwrap();

    let mut cmd = Command::cargo_bin(BIN).unwrap();
    cmd.arg("verify")
        .arg("--proof")
        .arg("kat/NOT_EXISTING_PROOF")
        .arg("--public-input")
        .arg("kat/public_inputs")
        .arg("--vk")
        .arg("kat/vk")
        .current_dir(project_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("File not found"));

    project_dir
        .child("target/proof")
        .assert(predicate::path::missing());
}
