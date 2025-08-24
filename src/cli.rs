//! Module for parsing command line arguments.

use anyhow::{bail, Context, Result};
use std::{collections::VecDeque, env, fs};
use completers::Completion;
use super::Config;

/// Possible types of actions.
#[derive(Debug)]
pub enum Action {
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

impl Action {
    /// Parse the action from the command line arguments, removing the first argument.
    pub fn parse(args: &mut VecDeque<String>) -> Result<Self> {
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
}

/// Handles completion request and exit, if any.
pub fn handle_completion(debug: bool) -> Result<()> {
    if let Some(completion) = Completion::init()? {
        match completion.word_index {
            0 => Completion::complete::<[&str; 0]>([]),
            1 => {
                let search = &completion.words[1];
                let options = vec![
                    "--help", "-h",
                    "--version", "-v",
                    "--list", "-l",
                ];
                let option_candidates = options.iter().filter_map(|option| {
                    if option.starts_with(search) {
                        Some(option.to_string())
                    } else {
                        None
                    }
                });
                let config = locate_config_file(debug)?;
                let recipe_candidates = config.recipes.into_iter().flat_map(|recipe| {
                    recipe.names.into_iter().filter(|name| name.starts_with(search))
                });
                let candidates = option_candidates.chain(recipe_candidates);
                Completion::complete(candidates);
            },
            _ => {
                // TODO: Delegate
                Completion::complete::<[&str; 0]>([]);
            },
        }
    }
    Ok(())
}


/// Locate config file in the current directory and its parents. To be specific:
///
/// 1. Find the closest parent directory that contains a `.jiu.toml` file.
/// 2. Deserialize the file into a [`Config`] struct.
/// 3. Set working directory to the directory containing the config file.
pub fn locate_config_file(debug: bool) -> Result<Config> {
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

