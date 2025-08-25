#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::cargo)]

use anyhow::{Context, Ok, Result, bail};
use jiu::{Action, handle_completion, locate_config_file};
use std::{collections::VecDeque, env};
use supports_color::Stream;

fn main() -> Result<()> {
    // Checking environment & Shell completion
    let debug = env::var("JIU_DEBUG").is_ok();
    handle_completion(debug)?;
    let color = supports_color::on(Stream::Stdout)
        .map(|level| level.has_basic)
        .unwrap_or(false);

    // Collecting arguments
    let mut iter = env::args();
    let program_name = iter.next().unwrap_or_else(|| "jiu".to_string());
    let mut args: VecDeque<String> = iter.collect();

    // Resolving actions
    let action = Action::parse(&mut args)?;
    let (config, recipe_name) = match action {
        Action::Help => {
            help(&program_name);
            return Ok(());
        }
        Action::Version => {
            version();
            return Ok(());
        }
        Action::List => {
            let Some(config) = locate_config_file(debug)? else {
                bail!("No config file found");
            };
            println!("{}", config.summarize(color));
            return Ok(());
        }
        Action::Default => {
            let Some(config) = locate_config_file(debug)? else {
                bail!("No config file found");
            };
            if config.default.is_empty() {
                println!("{}", config.summarize(color));
                return Ok(());
            }
            let default = config.default.clone();
            (config, default)
        }
        Action::Recipe(name) => {
            let Some(config) = locate_config_file(debug)? else {
                bail!("No config file found");
            };
            (config, name)
        }
    };

    if debug {
        eprintln!("I am \"{program_name}\" running recipe \"{recipe_name}\"");
        eprintln!("Received recipe arguments: {args:?}");
    }

    // Finding the recipe
    let Some(recipe) = config
        .recipes
        .into_iter()
        .find(|r| r.names.contains(&recipe_name))
    else {
        bail!("Recipe \"{recipe_name}\" not found");
    };

    // Resolving the recipe
    let word_index = args.len() - 1; // We dont't need word index here, so set to the last word
    let resolved = recipe
        .resolve(args, word_index)
        .with_context(|| format!("Error resolving recipe \"{recipe_name}\""))?
        .0;
    if debug {
        eprintln!("Resolved command: {resolved:?}");
    }

    // Executing the command
    let status = std::process::Command::new(&resolved[0])
        .args(&resolved[1..])
        .spawn()
        .with_context(|| format!("Error spawning command \"{resolved:?}\""))?
        .wait()
        .with_context(|| format!("Error waiting for command \"{resolved:?}\""))?;

    if debug {
        eprintln!("Command exited with {status}");
    }
    std::process::exit(status.code().unwrap_or(1));
}

/// Show help message.
fn help(program_name: &str) {
    println!(
        "{}: {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_DESCRIPTION")
    );
    println!();
    println!("Usage: {program_name} [OPTION_OR_RECIPE] [ARGS]...");
    println!();
    println!("Options:");
    println!("  -h, --help       Show this help message");
    println!("  -v, --version    Show version information");
    println!("  -l, --list       List all available recipes");
    println!();
}

/// Show version information.
fn version() {
    println!("{}@{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}
