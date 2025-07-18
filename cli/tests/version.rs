use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

const BIN: &str = env!("CARGO_PKG_NAME");

#[test]
fn get_version() {
    let mut cmd = Command::cargo_bin(BIN).unwrap();

    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}
