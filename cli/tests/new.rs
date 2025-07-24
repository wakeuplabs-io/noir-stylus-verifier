use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::process::Command;

const BIN: &str = env!("CARGO_PKG_NAME");
const PROJECT_NAME: &str = "test_project";

#[test]
fn happy_path() {
    let temp = assert_fs::TempDir::new().unwrap();
    let project_dir = temp.child(PROJECT_NAME);

    let mut cmd = Command::cargo_bin(BIN).unwrap();
    cmd.arg("new")
        .arg(PROJECT_NAME)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("What's Next?"));

    // check that the project directory exists
    project_dir.assert(predicate::path::exists());

    // check project files
    project_dir
        .child("Nargo.toml")
        .assert(predicate::path::exists());
    project_dir
        .child("src/main.nr")
        .assert(predicate::path::exists());
    project_dir
        .child("package.json")
        .assert(predicate::path::exists());
    project_dir
        .child("pnpm-lock.yaml")
        .assert(predicate::path::exists());
    project_dir
        .child("scripts/verify-global.js")
        .assert(predicate::path::exists());
    project_dir
        .child("scripts/verify.js")
        .assert(predicate::path::exists());
    project_dir
        .child("README.md")
        .assert(predicate::path::exists());

    // contracts folder should not exist, we create with generate later
    project_dir
        .child("contracts")
        .assert(predicate::path::missing());

    // check we respected project name for package.json and Nargo.toml
    project_dir
        .child("package.json")
        .assert(predicate::str::contains(PROJECT_NAME));
    project_dir
        .child("Nargo.toml")
        .assert(predicate::str::contains(PROJECT_NAME));
}

#[test]
fn fails_if_project_name_is_invalid() {
    let temp = assert_fs::TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin(BIN).unwrap();
    cmd.arg("new")
        .arg("Invalid Name")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Name is invalid"));

    // check that the project directory does not exist
    temp.child("Invalid Name")
        .assert(predicate::path::missing());
}

#[test]
fn fails_if_project_name_is_already_taken() {
    let temp = assert_fs::TempDir::new().unwrap();
    let project_dir = temp.child(PROJECT_NAME);
    project_dir.create_dir_all().unwrap();

    let mut cmd = Command::cargo_bin(BIN).unwrap();
    cmd.arg("new")
        .arg(PROJECT_NAME)
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Directory already exists"));
}
