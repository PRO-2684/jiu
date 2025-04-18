//! Module for parsing and resolving recipe arguments.

use anyhow::{Result, bail};
use owo_colors::OwoColorize;
use serde::{Deserialize, de::Error};
use std::{collections::VecDeque, fmt::Display};

/// A recipe argument defined the configuration file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentDefinition {
    /// The name of the argument.
    pub name: String,
    /// The argument type.
    pub arg_type: ArgumentType,
}

impl<'de> Deserialize<'de> for ArgumentDefinition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Take a string and parse it into an Argument
        let arg = String::deserialize(deserializer)?;
        Self::from_string::<D>(arg)
    }
}

impl ArgumentDefinition {
    /// Creates a new argument definition from a string.
    pub fn from_string<'de, D>(mut arg: String) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let first = arg
            .chars()
            .next()
            .ok_or_else(|| Error::custom("Empty argument"))?;
        let arg_type = match first {
            '?' => ArgumentType::Optional,
            '*' => ArgumentType::Variadic,
            '+' => ArgumentType::RequiredVariadic,
            _ => ArgumentType::Required,
        };
        if arg_type != ArgumentType::Required {
            arg.remove(0); // Remove the leading symbol
        }

        Ok(Self {
            name: arg,
            arg_type,
        })
    }
    /// Summarizes the argument, returning a string representation and the length.
    pub fn summarize(&self, color: bool) -> (String, usize) {
        let symbol = match self.arg_type {
            ArgumentType::Required => "",
            ArgumentType::Optional => "?",
            ArgumentType::Variadic => "*",
            ArgumentType::RequiredVariadic => "+",
        };
        let len = self.name.len() + symbol.len();
        let summary = if color {
            format!("{}{}", symbol.magenta(), self.name.cyan())
        } else {
            format!("{}{}", symbol, self.name)
        };
        (summary, len)
    }
}

/// A recipe argument with its value.
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl ArgumentType {
    /// Resolves the argument value.
    pub fn resolve(&self, args: &mut VecDeque<String>) -> Result<ResolvedArgument> {
        match self {
            Self::Required => {
                let Some(value) = args.pop_front() else {
                    bail!("Required argument not provided");
                };
                Ok(ResolvedArgument::Required(value))
            }
            Self::Optional => {
                let value = args.pop_front();
                Ok(ResolvedArgument::Optional(value))
            }
            Self::Variadic => {
                // Take all remaining arguments
                let values: Vec<String> = args.drain(..).collect();
                Ok(ResolvedArgument::Variadic(values))
            }
            Self::RequiredVariadic => {
                if args.is_empty() {
                    bail!("Required variadic argument must contain at least one value");
                }
                // Take all remaining arguments
                let values: Vec<String> = args.drain(..).collect();
                Ok(ResolvedArgument::RequiredVariadic(values))
            }
        }
    }
}

impl Display for ArgumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Required => write!(f, "Required"),
            Self::Optional => write!(f, "?Optional"),
            Self::Variadic => write!(f, "*Variadic"),
            Self::RequiredVariadic => write!(f, "+RequiredVariadic"),
        }
    }
}

/// A resolved argument with concrete values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedArgument {
    /// A required argument.
    Required(String),
    /// An optional argument. (`?`)
    Optional(Option<String>),
    /// A variadic argument. (`*`)
    Variadic(Vec<String>),
    /// A required variadic argument. (`+`)
    RequiredVariadic(Vec<String>),
}

impl ResolvedArgument {
    /// Gets the argument type.
    pub const fn arg_type(&self) -> ArgumentType {
        match self {
            Self::Required(_) => ArgumentType::Required,
            Self::Optional(_) => ArgumentType::Optional,
            Self::Variadic(_) => ArgumentType::Variadic,
            Self::RequiredVariadic(_) => ArgumentType::RequiredVariadic,
        }
    }

    /// Checks that the argument matches the expected type.
    pub fn matches(&self, arg_type: &ArgumentType) -> bool {
        self.arg_type() == *arg_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argument_resolving_1() {
        // Test the resolving of required and optional arguments
        let mut args = VecDeque::from(vec![
            "arg1".to_string(),
            "arg2".to_string(),
            "arg3".to_string(),
        ]);

        let arg = ArgumentType::Optional.resolve(&mut args).unwrap();
        assert_eq!(arg, ResolvedArgument::Optional(Some("arg1".to_string())));
        assert_eq!(
            args,
            VecDeque::from(vec!["arg2".to_string(), "arg3".to_string()])
        );

        let arg = ArgumentType::Required.resolve(&mut args).unwrap();
        assert_eq!(arg, ResolvedArgument::Required("arg2".to_string()));
        assert_eq!(args, VecDeque::from(vec!["arg3".to_string()]));

        let arg = ArgumentType::Optional.resolve(&mut args).unwrap();
        assert_eq!(arg, ResolvedArgument::Optional(Some("arg3".to_string())));
        assert_eq!(args, VecDeque::from(vec![]));

        let err = ArgumentType::Required.resolve(&mut args).unwrap_err();
        assert_eq!(err.to_string(), "Required argument not provided");
        assert_eq!(args, VecDeque::from(vec![]));

        let arg = ArgumentType::Optional.resolve(&mut args).unwrap();
        assert_eq!(arg, ResolvedArgument::Optional(None));
        assert_eq!(args, VecDeque::from(vec![]));
    }

    #[test]
    fn test_argument_resolving_2() {
        // Test the resolving of variadic and required variadic arguments
        let mut args = VecDeque::from(vec![
            "arg1".to_string(),
            "arg2".to_string(),
            "arg3".to_string(),
        ]);

        let arg = ArgumentType::Variadic.resolve(&mut args).unwrap();
        assert_eq!(
            arg,
            ResolvedArgument::Variadic(vec![
                "arg1".to_string(),
                "arg2".to_string(),
                "arg3".to_string()
            ])
        );
        assert_eq!(args, VecDeque::from(vec![]));

        let err = ArgumentType::RequiredVariadic
            .resolve(&mut args)
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "Required variadic argument must contain at least one value"
        );
        assert_eq!(args, VecDeque::from(vec![]));
    }
}
