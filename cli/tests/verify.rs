
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

const BIN: &str = env!("CARGO_PKG_NAME");
const PROJECT_NAME: &str = "my-project";
const RPC_URL: &str = "http://localhost:8547";

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
        .stdout(predicate::str::contains("contract size"));
}
