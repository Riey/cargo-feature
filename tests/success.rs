use predicates::prelude::*;
use assert_cmd::Command;

fn bin() -> Command {
    let mut cmd = Command::cargo_bin("cargo-feature").unwrap();
    cmd.current_dir("tests/test-ws").arg("feature").arg("-p");
    cmd
}

#[test]
// issue #6
fn hypen_underline() {
    let mut cmd = bin();
    cmd.arg("show_image");
    cmd.assert().success();
}

#[test]
// issue #6
fn optional_dep() {
    let mut cmd = bin();
    cmd.arg("serde").arg("+derive");
    cmd.assert().success();
}

#[test]
fn ui_not_exist() {
    let mut cmd = bin();
    cmd.arg("not-exists").arg("+foo");
    cmd.assert().failure().stderr(predicate::eq("Can't find package from metadata! please check package not-exists is exists in manifest"));
}
