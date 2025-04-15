//! # `jiu` library crate
//!
//! If you are reading this, you are reading the documentation for the `jiu` library crate. For the cli, kindly refer to the README file.

#![deny(missing_docs)]
#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::cargo)]

use serde::Deserialize;

/// The configuration.
#[derive(Deserialize, Debug)]
pub struct Config {
    /// Default recipe to run.
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
    pub arguments: Vec<ParsedArgument>,
    /// Command to run.
    pub command: Vec<LitOrArg>,
}

/// A recipe argument parsed from the configuration file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedArgument {
    /// The name of the argument.
    pub name: String,
    /// The initial argument instance.
    pub arg: Argument,
}

impl<'de> Deserialize<'de> for ParsedArgument {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Take a string and parse it into an Argument
        let mut name: String = String::deserialize(deserializer)?;
        let first = name.chars().next().ok_or(serde::de::Error::custom("Empty argument"))?;
        let arg_type = match first {
            '?' => Argument::Optional(None),
            '*' => Argument::Variadic(Vec::new()),
            '+' => Argument::RequiredVariadic(Vec::new()),
            _ => Argument::Required("".to_string()),
        };
        if !matches!(arg_type, Argument::Required(_)) {
            name.remove(0); // Remove the leading symbol
        }

        Ok(Self {
            name,
            arg: arg_type,
        })
    }
}

/// A recipe argument with its value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Argument {
    /// A required argument.
    Required(String),
    /// An optional argument. (`?`)
    Optional(Option<String>),
    /// A variadic argument. (`*`)
    Variadic(Vec<String>),
    /// A required variadic argument. (`+`)
    RequiredVariadic(Vec<String>),
}

impl Argument {
    /// Validate the argument.
    pub fn validate(&self) -> Result<(), &'static str> {
        match self {
            Argument::Required(v) => {
                if v.is_empty() {
                    Err("Required argument not provided")
                } else {
                    Ok(())
                }
            }
            Argument::Optional(v) => {
                if let Some(value) = v {
                    if value.is_empty() {
                        return Err("Optional argument cannot be empty if provided");
                    }
                }
                Ok(())
            }
            Argument::Variadic(v) => {
                for value in v {
                    if value.is_empty() {
                        return Err("Variadic argument cannot contain empty values");
                    }
                }
                Ok(())
            }
            Argument::RequiredVariadic(v) => {
                if v.is_empty() {
                    return Err("Required variadic argument must contain at least one value");
                }
                for value in v {
                    if value.is_empty() {
                        return Err("Required variadic argument cannot contain empty values");
                    }
                }
                Ok(())
            }
        }
    }
}

/// A string literal or an argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LitOrArg {
    /// A string literal.
    Literal(String),
    /// An argument.
    Argument(ParsedArgument),
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
            Arguments(Vec<ParsedArgument>),
        }

        match InnerRepr::deserialize(deserializer)? {
            InnerRepr::Arguments(mut args) => {
                // Only accept arrays of length 1
                let Some(arg) = args.pop() else {
                    return Err(serde::de::Error::custom("Expected a single argument, but got none"));
                };
                if !args.is_empty() {
                    return Err(serde::de::Error::custom("Expected a single argument, but got multiple"));
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
        assert_eq!(recipe.arguments[0].arg, Argument::Required("".to_string()));
        assert_eq!(recipe.arguments[1].name, "arg1");
        assert_eq!(recipe.arguments[1].arg, Argument::Optional(None));
        assert_eq!(recipe.arguments[2].name, "arg2");
        assert_eq!(recipe.arguments[2].arg, Argument::Variadic(Vec::new()));
        assert_eq!(recipe.arguments[3].name, "arg3");
        assert_eq!(recipe.arguments[3].arg, Argument::RequiredVariadic(Vec::new()));

        assert_eq!(recipe.command.len(), 6);
        assert_eq!(recipe.command[0], LitOrArg::Literal("echo".to_string()));
        assert_eq!(recipe.command[1], LitOrArg::Literal("Hello".to_string()));
        assert_eq!(recipe.command[2], LitOrArg::Argument(recipe.arguments[0].clone()));
        assert_eq!(recipe.command[3], LitOrArg::Argument(recipe.arguments[1].clone()));
        assert_eq!(recipe.command[4], LitOrArg::Argument(recipe.arguments[2].clone()));
        assert_eq!(recipe.command[5], LitOrArg::Argument(recipe.arguments[3].clone()));
    }
}
