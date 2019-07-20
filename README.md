# Silicon

[![crates.io](https://img.shields.io/crates/v/silicon.svg)](https://crates.io/crates/silicon)
[![Build Status](https://travis-ci.org/Aloxaf/silicon.svg?branch=master)](https://travis-ci.org/Aloxaf/silicon)
![license](https://img.shields.io/crates/l/silicon.svg)

Silicon is an alternative to [Carbon](https://github.com/dawnlabs/carbon) implemented in Rust.

It can render your source code into a beautiful image.

<img width="66%" src="http://storage.aloxaf.cn/silicon.png?v=1">

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
silicon --from-clipboard --l rs --to-clipboard
```

Use multiple fonts

```
silicon main.rs -o main.png -f 'Hack; SimSun'
```

Highlight specified line

```
silicon main.rs -o main.png --highlight-lines '1; 3-4'
```

Custom the image

```
silicon ./target/test.rs -o test.png --shadow-color '#555555' --background '#ffffff' --shadow-blur-radius 30 --no-window-controls
```

see `silicon --help` for detail
