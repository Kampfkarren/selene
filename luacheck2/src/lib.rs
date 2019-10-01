use std::{collections::HashMap, error::Error};

use serde::de::{Deserialize, Deserializer};

pub mod rules;

use rules::Rule;

// TODO: Implement Display, Error
#[derive(Debug)]
pub struct CheckerError {
    name: &'static str,
    problem: CheckerErrorProblem,
}

#[derive(Debug)]
pub enum CheckerErrorProblem {
    ConfigDeserializeError(Box<dyn Error>),
    RuleNewError(Box<dyn Error>),
}

macro_rules! use_rules {
    {
        $(
            $rule_name:ident: $rule_path:ty,
        )+
    } => {
        struct Checker {
            $(
                $rule_name: $rule_path,
            )+
        }

        impl Checker {
            // TODO: Be more strict about config? Make sure all keys exist
            fn from_config<V: 'static>(
                mut config: HashMap<String, V>,
            ) -> Result<Self, CheckerError> where V: for<'de> Deserialize<'de> + for<'de> Deserializer<'de> {
                Ok(Self {
                    $(
                        $rule_name: <$rule_path>::new({
                            match config.remove(stringify!($rule_name)) {
                                Some(entry_generic) => {
                                    <$rule_path as Rule>::Config::deserialize(entry_generic).map_err(|error| {
                                        CheckerError {
                                            name: stringify!($rule_name),
                                            problem: CheckerErrorProblem::ConfigDeserializeError(Box::new(error)),
                                        }
                                    })?
                                }

                                None => {
                                    <$rule_path as Rule>::Config::default()
                                }
                            }
                        }).map_err(|error| {
                            CheckerError {
                                name: stringify!($rule_name),
                                problem: CheckerErrorProblem::RuleNewError(Box::new(error)),
                            }
                        })?,
                    )+
                })
            }
        }
    };
}

use_rules! {
    empty_if: rules::empty_if::EmptyIfLint,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create() {
        Checker::from_config::<serde_json::Value>(HashMap::new()).unwrap();
    }
}
