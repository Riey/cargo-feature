# Unreleased

* Print `cargo metadata` error message rather than panic #30

# 0.7.0

* Support Rust 1.60.0
* More styling
* Support renamed optional package

# 0.6.0

* **Breaking** Add `target` option to set `[target]` section (https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#platform-specific-dependencies)
* List features now also print optional dependencies
* List features now have stable order
* Fix some documents
* Let all tests check their outputs

# 0.5.5

* Fix #23
* Fix manifest description

# 0.5.4

* Fix wrong behavior with `+default` #21
* Update toml_edit to 0.6.0

# 0.5.3

* Support {en, dis}able default features

# 0.5.2

* Support nix build

# 0.5.1

* Set default dependency command `+`
* Support target dependencies

# 0.5.0

* Using `^` for remove feature command
