# Silicon

[![Crates.io](https://img.shields.io/crates/v/silicon.svg)](https://crates.io/crates/silicon)
[![Documentation](https://docs.rs/silicon/badge.svg)](https://docs.rs/silicon)
[![Build Status](https://travis-ci.org/Aloxaf/silicon.svg?branch=master)](https://travis-ci.org/Aloxaf/silicon)
![License](https://img.shields.io/crates/l/silicon.svg)

Silicon is an alternative to [Carbon](https://github.com/dawnlabs/carbon) implemented in Rust.

It can render your source code into a beautiful image.

<img width="66%" src="http://storage.aloxaf.cn/silicon.png?v=2">

## Why Silicon

Carbon is a wonderful tool to create a beautiful image of your source code.

But it is a web application, which brings the following disadvantages:
 - Cannot work without Internet & browser.
 - Doesn't work well with shell. (Although there is _carbon-now-cli_, its experience is not very good, especially when the network is not so good.)

However, Silicon doesn't have these problem.
It's is implemented in Rust and can work without browser & Internet.

Silicon can render your source code on the fly while _carbon-now-cli_ takes several seconds on it.

## Disadvantages

It's not as beautiful as Carbon...

## Install

### Cargo

```bash
cargo install silicon

# or the latest version (Linux/macOS)

cargo install --git https://github.com/Aloxaf/silicon

# for Windows (see #11)

git clone --single-branch --branch dev https://github.com/Aloxaf/silicon
cd silicon
cargo build --release
```

### AUR

Silicon is available on AUR (Thanks to @radmen).

You can install it with any AUR helpers you like.

eg.
```bash
pikaur -S silicon
```

## Dependencies

### Arch Linux

```bash
sudo pacman -S --needed pkgconf freetype2 fontconfig libxcb xclip
```

## Basic Usage

Read code from file

```bash
silicon main.rs -o main.png 
```

Read code from clipboard, and copy the result image to clipboard(`--to-clipboard` is only available on Linux)

```bash
silicon --from-clipboard -l rs --to-clipboard
```

Use multiple fonts

```bash
silicon main.rs -o main.png -f 'Hack; SimSun'
```

Highlight specified line

```bash
silicon main.rs -o main.png --highlight-lines '1; 3-4'
```

Custom the image

```bash
silicon ./target/test.rs -o test.png \
    --shadow-color '#555555' --background '#ffffff' \
    --shadow-blur-radius 30 --no-window-controls
```

Transparent background

```bash
silicon ./target/test.rs -o test.png --background '#00ffffff'
```

see `silicon --help` for detail
