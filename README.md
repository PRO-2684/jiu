# jiu

[![GitHub License](https://img.shields.io/github/license/PRO-2684/jiu?logo=opensourceinitiative)](https://github.com/PRO-2684/jiu/blob/main/LICENSE)
[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/PRO-2684/jiu/release.yml?logo=githubactions)](https://github.com/PRO-2684/jiu/blob/main/.github/workflows/release.yml)
[![GitHub Release](https://img.shields.io/github/v/release/PRO-2684/jiu?logo=githubactions)](https://github.com/PRO-2684/jiu/releases)
[![GitHub Downloads (all assets, all releases)](https://img.shields.io/github/downloads/PRO-2684/jiu/total?logo=github)](https://github.com/PRO-2684/jiu/releases)
[![Crates.io Version](https://img.shields.io/crates/v/jiu?logo=rust)](https://crates.io/crates/jiu)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/jiu?logo=rust)](https://crates.io/crates/jiu)
[![docs.rs](https://img.shields.io/docsrs/jiu?logo=rust)](https://docs.rs/jiu)

A minimal command runner.

## 📥 Installation

### Using [`binstall`](https://github.com/cargo-bins/cargo-binstall)

```shell
cargo binstall jiu
```

### Downloading from Releases

Navigate to the [Releases page](https://github.com/PRO-2684/jiu/releases) and download respective binary for your platform. Make sure to give it execute permissions.

### Compiling from Source

```shell
cargo install jiu
```

## 💡 Examples

TODO

## 📖 Usage

TODO

## 🤔 Comparison

This tool is heavily inspired by [`just`](https://github.com/casey/just/), but is fundamentally different. To summarize:

- Pro: It handles arguments correctly, and without any ambiguity
    - `just` could cause argument splitting issues
    - Although [there are workarounds](https://just.systems/man/en/avoiding-argument-splitting.html), corner cases still exist
- Pro: Is independent of shell
- Con: But at the cost of much less customization and features

## 🎉 Credits

TODO
