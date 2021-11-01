# cargo-feature

[![Crates.io](https://img.shields.io/crates/v/cargo-feature)](https://crates.io/crates/cargo-feature)

[![Packaging status](https://repology.org/badge/vertical-allrepos/cargo-feature.svg)](https://repology.org/project/cargo-feature/versions)

![preview](https://github.com/Riey/cargo-feature/raw/master/preview.png)

## Install

### Cargo

`cargo install cargo-feature`

### Arch

`pacman -Syu cargo-feature`

### NixOS

`nix-env -iA nixos.cargo-feature`

## Usage

```
# add serde_derive feature to build-dependency of serde
cargo feature -t build serde +serde_derive

# disable default-features
cargo feature serde ^default

# same as above but more explict
cargo feature serde --disable-default-features

# if you want list all features, just type crate name
cargo feature serde

# enable default-features
cargo feature serde default

# same as above but more explict
cargo feature serde --enable-default-features

# add HtmlDivElement feature to dependency of web_sys 
cargo feature web_sys +HtmlDivElement

# you can skip typing +
cargo feature web_sys HtmlDivElement

# same as above but use `target.'cfg(target_arch = "wasm32")'.dependencies`
cargo feature --target="cfg(target_arch = \"wasm32\")" web_sys HtmlDivElement

# use `^` to remove feature
cargo feature web_sys ^HtmlDivElement
```
