use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

const BIN: &str = env!("CARGO_PKG_NAME");

#[test]
fn get_help() {
    let mut cmd = Command::cargo_bin(BIN).unwrap();

    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn get_help_per_command() {
    let mut cmd = Command::cargo_bin(BIN).unwrap();

    cmd.arg("new").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}
