#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::cargo)]

use anyhow::{Context, Result, bail};
use jiu::Config;
use std::{collections::VecDeque, env, fs};

fn main() -> Result<()> {
    // Checking environment
    let debug = env::var("JIU_DEBUG").is_ok();

    // Locating and parsing config file
    let config = locate_config_file(debug)?;

    // Collecting arguments
    let mut iter = env::args();
    let program_name = iter.next().unwrap_or_else(|| "jiu".to_string());
    let recipe_name = iter.next().unwrap_or(config.default);
    let args: VecDeque<String> = iter.collect();
    if debug {
        eprintln!("I am \"{program_name}\" running recipe \"{recipe_name}\"");
        eprintln!("Received recipe arguments: {args:?}");
    }

    // Listing recipies if the name is empty
    if recipe_name.is_empty() {
        if config.recipes.is_empty() {
            bail!("No recipes found");
        }
        if !config.description.is_empty() {
            println!("{}\n", config.description);
        }
        println!("Available recipes:");
        for recipe in config.recipes {
            println!("  {recipe}");
        }
        return Ok(());
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
