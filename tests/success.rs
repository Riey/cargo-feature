use ansi_term::Color;
use assert_cmd::Command;
use predicates::prelude::*;

fn bin() -> Command {
    let mut cmd = Command::cargo_bin("cargo-feature").unwrap();
    cmd.current_dir("tests/test-ws").arg("feature").arg("-p");
    cmd
}

#[test]
fn list_features() {
    let mut cmd = bin();
    cmd.arg("test_lib");
    cmd.assert().success().stdout(predicate::str::diff(format!(
        "{} = [{}, {}]\n{} = []\n{} = []\n",
        Color::Purple.bold().paint("default"),
        Color::Yellow.paint("\"foo\""),
        Color::Yellow.paint("\"bar\""),
        Color::Green.bold().paint("bar"),
        Color::Green.bold().paint("foo"),
    )));
}

#[test]
fn list_optional_deps_as_feature() {
    let mut cmd = bin();
    cmd.arg("test_lib_dep");
    cmd.assert().success().stdout(predicate::str::diff(format!(
        "{} {}\n",
        Color::Yellow.bold().paint("test-lib"),
        Color::Yellow.dimmed().paint("(optional)"),
    )));
}

#[test]
// https://github.com/Riey/cargo-feature/issues/6
fn hypen_underline() {
    let mut cmd = bin();
    cmd.arg("test_lib");
    cmd.assert().success().stdout(predicate::str::diff(format!(
        "{} = [{}, {}]\n{} = []\n{} = []\n",
        Color::Purple.bold().paint("default"),
        Color::Yellow.paint("\"foo\""),
        Color::Yellow.paint("\"bar\""),
        Color::Green.bold().paint("bar"),
        Color::Green.bold().paint("foo"),
    )));
}

#[test]
// https://github.com/Riey/cargo-feature/issues/12
fn default_add() {
    let mut cmd = bin();
    cmd.arg("test-lib").arg("foo");
    cmd.assert().success().stdout(predicate::str::diff(
        r#"[package]
name = "test-ws"
version = "0.1.0"
authors = ["Riey <creeper844@gmail.com>"]
edition = "2018"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
test-lib = { path = "../test-lib", features = ["bar", "foo"] }

[target.'cfg(target_arch = "wasm32")'.dependencies]
# issue #9, #11
test-lib-dep = { path = "../test-lib-dep" }
"#,
    ));
}

#[test]
// https://github.com/Riey/cargo-feature/issues/9
fn remove_feature() {
    let mut cmd = bin();
    cmd.arg("test-lib").arg("^bar");
    cmd.assert().success().stdout(predicate::str::diff(
        r#"[package]
name = "test-ws"
version = "0.1.0"
authors = ["Riey <creeper844@gmail.com>"]
edition = "2018"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
test-lib = { path = "../test-lib" }

[target.'cfg(target_arch = "wasm32")'.dependencies]
# issue #9, #11
test-lib-dep = { path = "../test-lib-dep" }
"#,
    ));
}

#[test]
fn disable_default_features() {
    let mut cmd = bin();
    cmd.arg("test-lib")
        .arg("^default")
        .arg("--disable-default-features");
    cmd.assert().success().stdout(predicate::str::diff(
        r#"[package]
name = "test-ws"
version = "0.1.0"
authors = ["Riey <creeper844@gmail.com>"]
edition = "2018"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
test-lib = { path = "../test-lib", features = ["bar"], default-features = false }

[target.'cfg(target_arch = "wasm32")'.dependencies]
# issue #9, #11
test-lib-dep = { path = "../test-lib-dep" }
"#,
    ));
}

#[test]
// https://github.com/Riey/cargo-feature/issues/11
fn add_target_feature() {
    let mut cmd = bin();
    cmd.arg("test-lib-dep")
        .arg("+test-lib")
        .arg("--target=cfg(target_arch = \"wasm32\")");
    cmd.assert().success().stdout(predicate::str::diff(
        r#"[package]
name = "test-ws"
version = "0.1.0"
authors = ["Riey <creeper844@gmail.com>"]
edition = "2018"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
test-lib = { path = "../test-lib", features = ["bar"] }

[target.'cfg(target_arch = "wasm32")'.dependencies]
# issue #9, #11
test-lib-dep = { path = "../test-lib-dep", features = ["test-lib"] }
"#,
    ));
}

#[test]
fn ui_not_exist() {
    let mut cmd = bin();
    cmd.arg("not-exists").arg("+foo");
    cmd.assert().failure().stderr(predicate::eq("Can't find package from metadata! please check package `not-exists` is exists in manifest\n"));
}
