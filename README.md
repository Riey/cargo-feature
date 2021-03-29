# cargo-feature

![preview](https://github.com/Riey/cargo-feature/blob/master/preview.png)

## Install

`cargo install cargo-feature`

## Usage

```
# add serde_derive feature to build-dependency of serde
cargo feature -t build serde +serde_derive

# add HtmlDivElement feature to dependency of web_sys 
cargo feature web_sys +HtmlDivElement

# you can skip typing +
cargo feature web_sys HtmlDivElement

# same as above but remove
cargo feature web_sys ^HtmlDivElement
```
