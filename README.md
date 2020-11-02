# Silicon

[![Crates.io](https://img.shields.io/crates/v/silicon.svg)](https://crates.io/crates/silicon)
[![Documentation](https://docs.rs/silicon/badge.svg)](https://docs.rs/silicon)
[![CI](https://github.com/Aloxaf/silicon/workflows/CI/badge.svg)](https://github.com/Aloxaf/silicon/actions?query=workflow%3ACI)
[![Linux Build Status](https://travis-ci.org/Aloxaf/silicon.svg?branch=master)](https://travis-ci.org/Aloxaf/silicon)
[![Windows build status](https://ci.appveyor.com/api/projects/status/github/Aloxaf/silicon?svg=true)](https://ci.appveyor.com/project/Aloxaf/silicon)
![License](https://img.shields.io/crates/l/silicon.svg)

Silicon is an alternative to [Carbon](https://github.com/dawnlabs/carbon) implemented in Rust.

It can render your source code into a beautiful image.

<img width="66%" src="http://storage.aloxaf.cn/silicon.png?v=2">

## Why Silicon

Carbon is a wonderful tool to create a beautiful image of your source code.

But it is a web application, which brings the following disadvantages:
 - Cannot work without Internet & browser.
 - Doesn't work well with shell. (Although there is _carbon-now-cli_, its experience is not very good, especially when the network is not so good.)

However, Silicon doesn't have these problems.
It's is implemented in Rust and can work without browser & Internet.

Silicon can render your source code on the fly while _carbon-now-cli_ takes several seconds on it.

## Disadvantages

It's not as beautiful as Carbon...

## Install

### Cargo

```bash
cargo install silicon
```

### AUR

Silicon is available on AUR (Thanks to @radmen).

You can install it with any AUR helpers you like.

eg.
```bash
pikaur -S silicon
```

### Homebrew

You can install Silicon using [Homebrew](https://brew.sh):

```bash
brew install silicon
```

## Dependencies

### Ubuntu
```bash
 sudo apt install expat
 sudo apt install libxml2-dev
 sudo apt install pkg-config libasound2-dev libssl-dev cmake libfreetype6-dev libexpat1-dev libxcb-composite0-dev
```

### Arch Linux

```bash
sudo pacman -S --needed pkgconf freetype2 fontconfig libxcb xclip
```

## Examples

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
    --shadow-color '#555' --background '#fff' \
    --shadow-blur-radius 30 --no-window-controls
```

Transparent background

The color can be `#RGB[A]` or `#RRGGBB[AA]`

```bash
silicon ./target/test.rs -o test.png --background '#fff0'
```

see `silicon --help` for detail

## Adding new syntaxes / themes

Silicon reads syntax-definition and theme cache from bat's cache directory. 

You can find the steps to add new syntaxes / themes for bat here: [sharkdp/bat#adding-new-syntaxes--language-definitions](https://github.com/sharkdp/bat#adding-new-syntaxes--language-definitions).
