# Silicon

[![crates.io](https://img.shields.io/crates/v/silicon.svg)](https://crates.io/crates/silicon)
![license](https://img.shields.io/crates/l/silicon.svg)

Silicon is an alternative to [Carbon](https://github.com/dawnlabs/carbon) implemented in Rust.

<img src="http://storage.aloxaf.cn/silicon.png">

## Why Silicon

Carbon is a wonderful tool to create a beautiful image of your source code.

But it is a web application, which brings the following disadvantages:
 - Cannot work without Internet & browser.
 - Doesn't work well with shell. (Although there is _carbon-now-cli_, its experience is not very good, especially when the network is not so good.)

However, Silicon doesn't have these problem.
It's is implemented in Rust and can work without browser & Internet.

Silicon can render your source code within 0.2 second while _carbon-now-cli_ takes several seconds on it.

## Disadvantages

It's not as beautiful as Carbon...

## Install

```
cargo install silicon
```

`xclip` is required on Linux for clipboard support.

## Basic Usage

Read code from file

```
silicon main.rs -o main.png 
```

Read code from clipboard, and copy the result image to clipboard(`--to-clipboard` is only available on Linux)

```
silicon --from-clipboard --language rs --to-clipboard
```

see `silicon --help` for detail

## TODO

- [x] MVP 
- [x] More themes
- [x] More syntaxes support
- [x] Better font support
- [ ] Clipboard support
  - [x] Get code from clipboard
  - [ ] Paste the image to clipboard
    - [x] Linux
    - [ ] Windows
    - [ ] MacOS
- [ ] Watermark
