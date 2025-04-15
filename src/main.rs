#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::cargo)]

use anyhow::{Context, Result, bail};
use jiu::Config;
use std::{collections::VecDeque, fs, env};

fn main() -> Result<()> {
    // Checking environment
    let debug = env::var("JIU_DEBUG").is_ok();

    // Parsing config
    let toml = fs::read_to_string(".jiu.toml")?;
    let config: Config = toml::de::from_str(&toml)?;
    if debug {
        eprintln!("{config:#?}");
    }

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
        println!("Available recipes:");
        for recipe in config.recipes {
            println!("- {recipe}");
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

    Ok(())
}
