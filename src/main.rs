#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::cargo)]

use anyhow::{Context, Result, bail};
use jiu::Config;
use std::{collections::VecDeque, fs};

fn main() -> Result<()> {
    let toml = fs::read_to_string(".jiu.toml")?;
    let config: Config = toml::de::from_str(&toml)?;
    println!("{config:#?}");

    let mut iter = std::env::args();
    let program_name = iter.next().unwrap_or_else(|| "jiu".to_string());
    let recipe_name = iter.next().unwrap_or(config.default);
    let args: VecDeque<String> = iter.collect();
    let Some(recipe) = config
        .recipes
        .into_iter()
        .find(|r| r.names.contains(&recipe_name))
    else {
        bail!("Recipe \"{recipe_name}\" not found");
    };
    let resolved = recipe
        .resolve(args)
        .with_context(|| format!("Error resolving recipe \"{recipe_name}\""))?;
    println!("{program_name} {recipe_name} {resolved:#?}");

    Ok(())
}
