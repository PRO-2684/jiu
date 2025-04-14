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
    pub arguments: Vec<Argument>,
    /// Command to run.
    pub command: Vec<StringOrArgument>,
}

/// A recipe argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Argument {
    /// The name of the argument.
    pub name: String,
    /// The type of the argument.
    pub arg_type: ArgumentType,
}

impl<'de> Deserialize<'de> for Argument {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Take a string and parse it into an Argument
        let mut name: String = String::deserialize(deserializer)?;
        let first = name.chars().next().ok_or(serde::de::Error::custom("Empty argument"))?;
        let arg_type = match first {
            '?' => ArgumentType::Optional,
            '*' => ArgumentType::Variadic,
            '+' => ArgumentType::RequiredVariadic,
            _ => ArgumentType::Required,
        };
        if arg_type != ArgumentType::Required {
            name.remove(0); // Remove the leading symbol
        }

        Ok(Self {
            name,
            arg_type,
        })
    }
}

/// The argument type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgumentType {
    /// A required argument.
    Required,
    /// An optional argument. (`?`)
    Optional,
    /// A variadic argument. (`*`)
    Variadic,
    /// A required variadic argument. (`+`)
    RequiredVariadic,
}

/// A string literal or an argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringOrArgument {
    /// A string literal.
    Literal(String),
    /// An argument.
    Argument(Argument),
}

impl<'de> Deserialize<'de> for StringOrArgument {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum InnerRepr {
            Literal(String),
            Arguments(Vec<Argument>),
        }

        match InnerRepr::deserialize(deserializer)? {
            InnerRepr::Arguments(args) => {
                // Only accept arrays of length 1
                if args.len() == 1 {
                    Ok(Self::Argument(args.into_iter().next().unwrap()))
                } else {
                    Err(serde::de::Error::custom("Expected a single argument"))
                }
            }
            InnerRepr::Literal(literal) => Ok(Self::Literal(literal)),
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO: Add tests for the library
}
