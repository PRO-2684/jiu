//! # `jiu` library crate
//!
//! If you are reading this, you are reading the documentation for the `jiu` library crate. For the cli, kindly refer to the README file.

#![deny(missing_docs)]
#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::cargo)]

mod arguments;

use anyhow::{Result, bail};
use arguments::{ArgumentDefinition, ResolvedArgument};
use serde::Deserialize;
use std::collections::{HashMap, VecDeque};

/// The configuration.
#[derive(Deserialize, Debug)]
pub struct Config {
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
    command: Vec<LitOrArg>,
}

impl Recipe {
    /// Resolves to a command with the given arguments.
    pub fn resolve(self, mut args: VecDeque<String>) -> Result<Vec<String>> {
        let Self {
            arguments, command, ..
        } = self;

        // Resolve the arguments
        let mut resolved_args = HashMap::new();
        for arg in arguments {
            let resolved_arg = arg.arg_type.resolve(&mut args)?;
            resolved_args.insert(arg.name, resolved_arg);
        }

        // Resolve the command
        let mut resolved_command = Vec::new();
        for component in command {
            match component {
                LitOrArg::Literal(literal) => resolved_command.push(literal),
                LitOrArg::Argument(arg) => {
                    let Some(resolved_arg) = resolved_args.get(&arg.name) else {
                        bail!("Argument {} not found", arg.name);
                    };
                    if !resolved_arg.matches(&arg.arg_type) {
                        bail!(
                            "Argument {} does not match type {:?}",
                            arg.name,
                            arg.arg_type
                        );
                    }
                    match resolved_arg {
                        ResolvedArgument::Required(value) => resolved_command.push(value.clone()),
                        ResolvedArgument::Optional(value) => {
                            if let Some(v) = value {
                                resolved_command.push(v.clone());
                            }
                        }
                        ResolvedArgument::Variadic(values) => {
                            for value in values {
                                resolved_command.push(value.clone());
                            }
                        }
                        ResolvedArgument::RequiredVariadic(values) => {
                            for value in values {
                                resolved_command.push(value.clone());
                            }
                        }
                    }
                }
            }
        }

        // Check if there are any remaining arguments
        if !args.is_empty() {
            bail!("Unexpected argument(s): {args:?}");
        }

        Ok(resolved_command)
    }
}

/// A string literal or an argument.
#[derive(Debug, Clone, PartialEq, Eq)]
enum LitOrArg {
    /// A string literal.
    Literal(String),
    /// An argument.
    Argument(ArgumentDefinition),
}

impl<'de> Deserialize<'de> for LitOrArg {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum InnerRepr {
            Literal(String),
            Arguments(Vec<ArgumentDefinition>),
        }

        match InnerRepr::deserialize(deserializer)? {
            InnerRepr::Arguments(mut args) => {
                // Only accept arrays of length 1
                let Some(arg) = args.pop() else {
                    return Err(serde::de::Error::custom(
                        "Expected a single argument, but got none",
                    ));
                };
                if !args.is_empty() {
                    return Err(serde::de::Error::custom(
                        "Expected a single argument, but got multiple",
                    ));
                }
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
        assert_eq!(recipe.command[0], LitOrArg::Literal("echo".to_string()));
        assert_eq!(recipe.command[1], LitOrArg::Literal("Hello".to_string()));
        assert_eq!(
            recipe.command[2],
            LitOrArg::Argument(recipe.arguments[0].clone())
        );
        assert_eq!(
            recipe.command[3],
            LitOrArg::Argument(recipe.arguments[1].clone())
        );
        assert_eq!(
            recipe.command[4],
            LitOrArg::Argument(recipe.arguments[2].clone())
        );
        assert_eq!(
            recipe.command[5],
            LitOrArg::Argument(recipe.arguments[3].clone())
        );
    }
}
