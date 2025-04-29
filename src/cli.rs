//! Module for parsing command line arguments.

use anyhow::{Result, bail};
use std::collections::VecDeque;

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
