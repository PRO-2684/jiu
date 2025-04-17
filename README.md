# jiu

[![GitHub License](https://img.shields.io/github/license/PRO-2684/jiu?logo=opensourceinitiative)](https://github.com/PRO-2684/jiu/blob/main/LICENSE)
[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/PRO-2684/jiu/release.yml?logo=githubactions)](https://github.com/PRO-2684/jiu/blob/main/.github/workflows/release.yml)
[![GitHub Release](https://img.shields.io/github/v/release/PRO-2684/jiu?logo=githubactions)](https://github.com/PRO-2684/jiu/releases)
[![GitHub Downloads (all assets, all releases)](https://img.shields.io/github/downloads/PRO-2684/jiu/total?logo=github)](https://github.com/PRO-2684/jiu/releases)
[![Crates.io Version](https://img.shields.io/crates/v/jiu?logo=rust)](https://crates.io/crates/jiu)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/jiu?logo=rust)](https://crates.io/crates/jiu)
[![docs.rs](https://img.shields.io/docsrs/jiu?logo=rust)](https://docs.rs/jiu)

A minimal command runner.

## 🤔 Comparison

This tool is heavily inspired by [`just`](https://github.com/casey/just/), but is fundamentally different. To summarize:

- Pro: It handles arguments correctly, and without any ambiguity
    - `just` could cause argument splitting issues
    - Although [there are workarounds](https://just.systems/man/en/avoiding-argument-splitting.html), corner cases still exist
- Pro: Is independent of shell
- Con: But at the cost of much less customization and features

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

<details><summary>Demo asciicast</summary>

[![asciicast](https://asciinema.org/a/bnuQc8QP9IcgUoAY2gytD7h5T.svg)](https://asciinema.org/a/bnuQc8QP9IcgUoAY2gytD7h5T)

</details>

See [`.jiu.toml`](./.jiu.toml) for a simple example used in this repository, ~~or the [`tests`](./tests) directory for more complex examples~~. Here's an example involving complex arguments:

```shell
jiu dummy 1 "2" '"3"' " 4" "" "5 6"
```

Which will invoke [`dummy.sh`](./scripts/dummy.sh), printing arguments it received:

```shell
TERM = xterm-256color
Arguments:
1
2
"3"
 4

5 6
```

Note that the arguments are all handled correctly.

## 📖 Usage

### Configuration

The config file is a simple TOML file named `.jiu.toml`. The format is as follows:

```toml
description = "`jiu`: A minimal command runner." # Description of the configuration (Optional)
# - Will be displayed when listing recipes
# - To add some colors to the dull description, use ANSI escape codes like:
#   description = "\u001b[1;36mjiu\u001b[22;39m: A minimal command runner."
default = "run" # Default recipe to run when invoked without any arguments (Optional)
# - List all recipes if empty
# - Default recipe must be able to accept no arguments

[[recipes]]
names = ["run", "r"] # Names of the recipe (Required)
description = "Compile and run" # Description of the recipe (Optional)
arguments = ["*rest"] # Arguments to the recipe (Optional)
command = ["cargo", "run", "--", ["*rest"]] # Command to run (Required)

# ...More recipes
```

#### Names

The `names` field is a list of names that the recipe can be called with. It should contain at least one name, otherwise the recipe will never be matched. Each name:

- Should be unique across all recipes, otherwise only the first one will be matched.
- Should not contain spaces.
- Should not start with special characters, especially `-`, which would be interpreted as an option.
- Should not be empty.

Where "should" means that it is a good practice to follow, but not explicitly enforced. For example, you can have a recipe with the name `my recipe`, but to call it you would have to escape the space or use quotes, which would be inconvenient.

#### Arguments

The `arguments` field is a list of arguments that the recipe accepts. It should be a list of strings, where each string represents an argument. An argument is made up of an optional leading symbol and a name.

##### Types

The type of the argument is determined by the leading symbol, which can be one of the following:

- `*`: A variadic argument. This means that the argument can accept zero or more values.
- `+`: A required variadic argument. This means that the argument must accept one or more values.
- `?`: An optional argument. This means that the argument can accept zero or one value.

If the leading symbol is omitted, the argument is treated as a required argument.

##### Greedy Matching

> [!NOTE]
> This behavior may be changed in the future.

The `*` and `+` arguments are greedy, meaning that they will consume all remaining arguments. For example, if you have a recipe with the following arguments:

```toml
arguments = ["*arg0", "*arg1"]
```

Then `*arg1` will always be empty, since `*arg0` will consume all remaining arguments. Also consider:

```toml
arguments = ["*arg0", "arg1"]
```

In this case, `*arg0` will consume all remaining arguments, leaving required argument `arg1` empty. So `jiu` will return an error, although the arguments can be interpreted without ambiguity.

Also be careful when working with optional arguments, since they share the same greedy behavior. For example:

```toml
arguments = ["?arg0", "arg1"]
```

When a single argument is passed, `?arg0` will consume it, leaving `arg1` empty. So this will also cause an error.

#### Command

The `command` field is a list representing the command to run, and  is made up of strings and arrays of length 1. Each string is treated as a literal, while each array is treated as a placeholder.

The placeholders are interpolated with concrete values when the recipe is run. After interpolation, the command is executed in the directory of the config file.

A placeholder can be one of the following:

- `$VAR`: An environment variable. This will be replaced with the value of the environment variable `VAR`.
    - If the variable is not set, an error will be returned.
    - If the variable is empty, it will still be passed as an empty argument.
- Others: An argument. This will be replaced with the value of the argument. If the argument is variadic, it will be replaced with all values of the argument.

### Running

To run a recipe, simply call `jiu` with the name of the recipe and arguments for the recipe:

```shell
jiu <recipe> [<args>...]
```

### Debugging

Run with environment variable `JIU_DEBUG` set to enable debug mode. In bash, you can do this with:

```shell
JIU_DEBUG=1 jiu <recipe> [<args>...]
```

Which would provide additional information for debugging purposes.

## ✅ TODO

- [ ] `env` field on recipes and global
- [ ] Set working directories
    - [ ] Where the config file is located (default)
    - [ ] Where the command is invoked
    - [ ] Custom working directory, relative to the config file
- [ ] Options
    - [ ] Only allow recipe names starting with letters (or valid identifiers in Rust?)
    - [ ] `jiu -l`/`jiu --list`: List recipes
    - [ ] `jiu -h`/`jiu --help`: Help message
    - [ ] `jiu -v`/`jiu --version`: Version
    - [ ] Color control? Verbosity?
- [x] Interpolating environment variables in commands (`["$VAR"]`)

## 🎉 Credits

- [`just`](https://github.com/casey/just/) - where the inspiration came from
