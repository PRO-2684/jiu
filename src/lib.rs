//! # `jiu` library crate
//!
//! If you are reading this, you are reading the documentation for the `jiu` library crate. For the cli, kindly refer to the README file.

#![deny(missing_docs)]
#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::cargo)]

mod arguments;
#[cfg(feature = "cli")]
mod cli;

use anyhow::{Context, Result, bail};
use arguments::{ArgumentDefinition, ResolvedArgument};
#[cfg(feature = "cli")]
pub use cli::Action;
use owo_colors::OwoColorize;
use serde::{Deserialize, de::Error};
use std::collections::{HashMap, VecDeque};

/// The configuration.
#[derive(Deserialize, Debug)]
pub struct Config {
    /// Description of the configuration.
    #[serde(default)]
    pub description: String,
    /// Default recipe to run when invoked without any arguments.
    ///
    /// - List all recipes if empty.
    /// - Default recipe must be able to accept no arguments.
    #[serde(default)]
    pub default: String,
    /// Recipes.
    #[serde(default)]
    pub recipes: Vec<Recipe>,
}

impl Config {
    /// Summarizes the configuration.
    #[must_use]
    pub fn summarize(&self, color: bool) -> String {
        let description = if self.description.is_empty() {
            String::new()
        } else {
            // Add an extra new line
            format!("{}\n", self.description)
        };

        let recipes = self.summarize_recipes(color);
        format!("{description}\n{recipes}")
    }

    /// Summarizes the recipes.
    fn summarize_recipes(&self, color: bool) -> String {
        if self.recipes.is_empty() {
            return "No recipes found".to_string();
        }

        // A pack of (definition, definition_length, description)
        let pack: Vec<_> = self
            .recipes
            .iter()
            .map(|recipe| {
                let (def, def_len) = recipe.summarize_definition(color);
                (def, def_len, &recipe.description)
            })
            .collect();
        let max_def_len = pack.iter().map(|(_, len, _)| *len).max().unwrap_or(0);

        let recipes = pack
            .into_iter()
            .map(|(def, def_len, description)| {
                // Calculate required padding
                let padding = max_def_len.saturating_sub(def_len);
                let padding = " ".repeat(padding);

                // Format the description
                let description = if description.is_empty() {
                    String::new()
                } else {
                    let s = format!(" # {description}");
                    if color { s.dimmed().to_string() } else { s }
                };

                // Format the summary for this recipe
                format!("  {def}{padding}{description}")
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!("Available recipes:\n{recipes}")
    }
}

/// The recipe.
#[derive(Deserialize, Debug)]
pub struct Recipe {
    /// Names of the recipe.
    ///
    /// Must contain at least one name and each name should be unique across all recipes.
    pub names: Vec<String>,
    /// Description of the recipe.
    #[serde(default)]
    pub description: String,
    /// Arguments to the recipe.
    #[serde(default)]
    arguments: Vec<ArgumentDefinition>,
    /// Command to run.
    command: Vec<Component>,
}

impl Recipe {
    /// Resolves to a command with the given arguments.
    ///
    /// ## Errors
    ///
    /// - If an argument could not be resolved.
    /// - If a referenced argument is not defined.
    /// - If a referenced argument does not match the defined type.
    /// - If unexpected arguments are left after resolving.
    pub fn resolve(self, mut args: VecDeque<String>) -> Result<Vec<String>> {
        let Self {
            arguments, command, ..
        } = self;

        // Resolve the arguments
        let mut resolved_args = HashMap::new();
        for arg in arguments {
            let resolved_arg = arg.arg_type.resolve(&mut args).with_context(|| {
                format!("While resolving argument \"{}\"", arg.summarize(false).0)
            })?;
            resolved_args.insert(arg.name, resolved_arg);
        }

        // Resolve the command
        let mut resolved_command = Vec::new();
        for component in command {
            match component {
                Component::Literal(literal) => resolved_command.push(literal),
                Component::Argument(ref_arg) => {
                    let Some(resolved_arg) = resolved_args.get(&ref_arg.name) else {
                        bail!("Argument {} not found", ref_arg.name);
                    };
                    if !resolved_arg.matches(&ref_arg.arg_type) {
                        bail!(
                            "Argument \"{}\" defined as {} but referenced as {}",
                            ref_arg.name,
                            resolved_arg.arg_type(),
                            ref_arg.arg_type,
                        );
                    }
                    match resolved_arg {
                        ResolvedArgument::Required(value) => resolved_command.push(value.clone()),
                        ResolvedArgument::Optional(value) => {
                            if let Some(v) = value {
                                resolved_command.push(v.clone());
                            }
                        }
                        ResolvedArgument::Variadic(values)
                        | ResolvedArgument::RequiredVariadic(values) => {
                            for value in values {
                                resolved_command.push(value.clone());
                            }
                        }
                    }
                }
                Component::EnvVar(var_name) => {
                    let value = std::env::var(&var_name)?;
                    resolved_command.push(value);
                }
            }
        }

        // Check if there are any remaining arguments
        if !args.is_empty() {
            bail!("Unexpected argument(s): {args:?}");
        }

        Ok(resolved_command)
    }

    /// Summarizes the recipe definition, returning a string representation and the length.
    #[must_use]
    pub fn summarize_definition(&self, color: bool) -> (String, usize) {
        let sep = if color {
            "/".dimmed().to_string()
        } else {
            "/".to_string()
        };
        let names = self.names.join(&sep);
        let mut def_len = self.names.iter().map(|name| name.len() + 1).sum();
        def_len -= 2; // One for the extra separator and one for the extra space

        let arguments: Vec<String> = self
            .arguments
            .iter()
            .map(|arg| {
                let (arg_name, arg_len) = arg.summarize(color);
                def_len += arg_len + 1; // +1 for the space
                arg_name
            })
            .collect();
        let arguments = if arguments.is_empty() {
            String::new()
        } else {
            format!(" {}", arguments.join(" "))
        };
        (format!("{names}{arguments}"), def_len)
    }
}

/// A component of a command.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Component {
    /// A string literal.
    Literal(String),
    /// An argument.
    Argument(ArgumentDefinition),
    /// An environment variable.
    EnvVar(String),
}

impl<'de> Deserialize<'de> for Component {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum InnerRepr {
            Literal(String),
            Array(Vec<String>),
        }

        match InnerRepr::deserialize(deserializer)? {
            InnerRepr::Array(mut array) => {
                // Only accept arrays of length 1
                let placeholder = array
                    .pop()
                    .ok_or_else(|| Error::custom("Expected a single argument, but got none"))?;
                if !array.is_empty() {
                    return Err(Error::custom(
                        "Expected a single argument, but got multiple",
                    ));
                }

                // Parse the content as an environment variable (if starts with $)
                if placeholder.starts_with('$') {
                    let mut var_name = placeholder;
                    var_name.remove(0); // Remove the leading $
                    return Ok(Self::EnvVar(var_name));
                }

                // Parse the content as an argument
                let arg = ArgumentDefinition::from_string::<D>(placeholder)?;
                Ok(Self::Argument(arg))
            }
            InnerRepr::Literal(literal) => Ok(Self::Literal(literal)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arguments::ArgumentType;

    #[test]
    fn test_config() {
        let config: Config = toml::from_str(
            r#"
            default = "test"
            [[recipes]]
            names = ["test"]
            description = "Test recipe"
            arguments = ["arg0", "?arg1", "*arg2", "+arg3"]
            command = ["echo", "Hello", ["arg0"], ["?arg1"], ["*arg2"], ["+arg3"]]
        "#,
        )
        .unwrap();

        assert_eq!(config.description, "");
        assert_eq!(config.default, "test");
        assert_eq!(config.recipes.len(), 1);

        let recipe = &config.recipes[0];
        assert_eq!(recipe.names, vec!["test"]);
        assert_eq!(recipe.description, "Test recipe");
        assert_eq!(recipe.arguments.len(), 4);

        assert_eq!(recipe.arguments[0].name, "arg0");
        assert_eq!(recipe.arguments[0].arg_type, ArgumentType::Required);
        assert_eq!(recipe.arguments[1].name, "arg1");
        assert_eq!(recipe.arguments[1].arg_type, ArgumentType::Optional);
        assert_eq!(recipe.arguments[2].name, "arg2");
        assert_eq!(recipe.arguments[2].arg_type, ArgumentType::Variadic);
        assert_eq!(recipe.arguments[3].name, "arg3");
        assert_eq!(recipe.arguments[3].arg_type, ArgumentType::RequiredVariadic);

        assert_eq!(recipe.command.len(), 6);
        assert_eq!(recipe.command[0], Component::Literal("echo".to_string()));
        assert_eq!(recipe.command[1], Component::Literal("Hello".to_string()));
        assert_eq!(
            recipe.command[2],
            Component::Argument(recipe.arguments[0].clone())
        );
        assert_eq!(
            recipe.command[3],
            Component::Argument(recipe.arguments[1].clone())
        );
        assert_eq!(
            recipe.command[4],
            Component::Argument(recipe.arguments[2].clone())
        );
        assert_eq!(
            recipe.command[5],
            Component::Argument(recipe.arguments[3].clone())
        );
    }
}
