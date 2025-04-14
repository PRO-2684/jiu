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

See [`.jiu.toml`](./.jiu.toml) for a simple example used in this repository~~, or the [`tests`](./tests) directory for more complex examples~~.

## 📖 Usage

### Configuration

The config file is a simple TOML file named `.jiu.toml`. The format is as follows:

```toml
default = "run" # Default recipe to run (Optional, lists all recipes if empty)

[[recipes]]
names = ["run", "r"] # Names of the recipe (Required, must contain at least one name and each name should be unique across all recipes)
description = "Compile and run" # Description of the recipe (Optional)
arguments = ["*rest"] # Arguments to the recipe (Optional)
command = ["cargo", "run", "--", ["*rest"]] # Command to run (Required)

# ...More recipes
```

#### Arguments

The `arguments` field is a list of arguments that the recipe accepts. It should be a list of strings, where each string represents an argument. An argument is made up of an optional leading symbol and a name. The leading symbol can be one of the following:

- `*`: A variadic argument. This means that the argument can accept zero or more values.
- `+`: A required variadic argument. This means that the argument must accept one or more values.
- `?`: An optional argument. This means that the argument can accept zero or one value.

If the leading symbol is omitted, the argument is treated as a required argument.

#### Command

The `command` field is a list made up of strings and arrays that represents the command to run. Each string is treated as a literal string, while each array is treated as a placeholder for the arguments. The placeholders are replaced with the values of the arguments when the command is run.

## 🤔 Comparison

This tool is heavily inspired by [`just`](https://github.com/casey/just/), but is fundamentally different. To summarize:

- Pro: It handles arguments correctly, and without any ambiguity
    - `just` could cause argument splitting issues
    - Although [there are workarounds](https://just.systems/man/en/avoiding-argument-splitting.html), corner cases still exist
- Pro: Is independent of shell
- Con: But at the cost of much less customization and features

## ✅ TODO

- `env` field

## 🎉 Credits

- [`just`] - where the inspiration came from
