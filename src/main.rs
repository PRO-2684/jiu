#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::cargo)]

use anyhow::{Context, Ok, Result, bail};
use jiu::Config;
use std::{collections::VecDeque, env, fs};
use supports_color::Stream;

fn main() -> Result<()> {
    // Checking environment
    let color = supports_color::on(Stream::Stdout)
        .map(|level| level.has_basic)
        .unwrap_or(false);
    let debug = env::var("JIU_DEBUG").is_ok();

    // Collecting arguments
    let mut iter = env::args();
    let program_name = iter.next().unwrap_or_else(|| "jiu".to_string());
    let mut args: VecDeque<String> = iter.collect();

    // Resolving actions
    let action = resolve_actions(&mut args)?;
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
            let config = locate_config_file(debug)?;
            println!("{}", config.summarize(color));
            return Ok(());
        }
        Action::Default => {
            let config = locate_config_file(debug)?;
            if config.default.is_empty() {
                println!("{}", config.summarize(color));
                return Ok(());
            }
            let default = config.default.clone();
            (config, default)
        }
        Action::Recipe(name) => {
            let config = locate_config_file(debug)?;
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
    let resolved = recipe
        .resolve(args)
        .with_context(|| format!("Error resolving recipe \"{recipe_name}\""))?;
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

/// Possible types of actions.
#[derive(Debug)]
enum Action {
    /// Display help message.
    Help,
    /// Display version information.
    Version,
    /// List all available recipes.
    List,
    /// Execute the default recipe.
    Default,
    /// Execute a recipe.
    Recipe(String),
}

/// Locate config file in the current directory and its parents. To be specific:
///
/// 1. Find the closest parent directory that contains a `.jiu.toml` file.
/// 2. Deserialize the file into a [`Config`] struct.
/// 3. Set working directory to the directory containing the config file.
fn locate_config_file(debug: bool) -> Result<Config> {
    let mut path = env::current_dir()?;
    loop {
        let config_path = path.join(".jiu.toml");
        if config_path.exists() {
            let config = fs::read_to_string(&config_path)
                .with_context(|| format!("Error reading config file \"{config_path:?}\""))?;
            if debug {
                eprintln!("Found config file: {config_path:?}");
            }
            let config: Config = toml::de::from_str(&config)
                .with_context(|| format!("Error deserializing config file \"{config_path:?}\""))?;
            if debug {
                eprintln!("Deserialized config: {config:#?}");
            }

            // Set the working directory to the directory containing the config file
            env::set_current_dir(&path)
                .with_context(|| format!("Error setting working directory to \"{path:?}\""))?;
            if debug {
                eprintln!("Set working directory to: {path:?}");
            }

            return Ok(config);
        }
        if !path.pop() {
            break;
        }
    }
    bail!("No config file found")
}

/// Resolve proper action from the command line arguments.
fn resolve_actions(args: &mut VecDeque<String>) -> Result<Action> {
    let first = args.pop_front();
    let Some(first) = first.as_ref() else {
        return Ok(Action::Default);
    };
    let action = match first.as_str() {
        "--help" | "-h" => Action::Help,
        "--version" | "-v" => Action::Version,
        "--list" | "-l" => Action::List,
        _ => {
            if first.starts_with('-') {
                bail!("Unknown option \"{first}\"");
            }
            Action::Recipe(first.to_string())
        }
    };

    Ok(action)
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
