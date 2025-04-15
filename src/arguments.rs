//! Module for parsing and resolving recipe arguments.

use std::collections::VecDeque;
use serde::Deserialize;

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
    pub fn resolve(&self, args: &mut VecDeque<String>) -> Result<ResolvedArgument, &'static str> {
        match self {
            Self::Required => {
                let Some(value) = args.pop_front() else {
                    return Err("Required argument not provided");
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
                    return Err("Required variadic argument must contain at least one value");
                }
                // Take all remaining arguments
                let values: Vec<String> = args.drain(..).collect();
                Ok(ResolvedArgument::RequiredVariadic(values))
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argument_resolving_1() {
        // Test the resolving of required and optional arguments
        let mut args = VecDeque::from(vec!["arg1".to_string(), "arg2".to_string(), "arg3".to_string()]);

        let arg = ArgumentType::Optional.resolve(&mut args).unwrap();
        assert_eq!(arg, ResolvedArgument::Optional(Some("arg1".to_string())));
        assert_eq!(args, VecDeque::from(vec!["arg2".to_string(), "arg3".to_string()]));

        let arg = ArgumentType::Required.resolve(&mut args).unwrap();
        assert_eq!(arg, ResolvedArgument::Required("arg2".to_string()));
        assert_eq!(args, VecDeque::from(vec!["arg3".to_string()]));

        let arg = ArgumentType::Optional.resolve(&mut args).unwrap();
        assert_eq!(arg, ResolvedArgument::Optional(Some("arg3".to_string())));
        assert_eq!(args, VecDeque::from(vec![]));

        let arg = ArgumentType::Required.resolve(&mut args).unwrap_err();
        assert_eq!(arg, "Required argument not provided");
        assert_eq!(args, VecDeque::from(vec![]));

        let arg = ArgumentType::Optional.resolve(&mut args).unwrap();
        assert_eq!(arg, ResolvedArgument::Optional(None));
        assert_eq!(args, VecDeque::from(vec![]));
    }

    #[test]
    fn test_argument_resolving_2() {
        // Test the resolving of variadic and required variadic arguments
        let mut args = VecDeque::from(vec!["arg1".to_string(), "arg2".to_string(), "arg3".to_string()]);

        let arg = ArgumentType::Variadic.resolve(&mut args).unwrap();
        assert_eq!(arg, ResolvedArgument::Variadic(vec!["arg1".to_string(), "arg2".to_string(), "arg3".to_string()]));
        assert_eq!(args, VecDeque::from(vec![]));

        let arg = ArgumentType::RequiredVariadic.resolve(&mut args).unwrap_err();
        assert_eq!(arg, "Required variadic argument must contain at least one value");
        assert_eq!(args, VecDeque::from(vec![]));
    }
}
