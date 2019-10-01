use std::{collections::HashMap, error::Error};

use full_moon::ast::Ast;
use serde::{Deserialize, de::Deserializer};

pub mod rules;

use rules::{Diagnostic, Rule};

// TODO: Implement Display, Error
#[derive(Debug)]
pub struct CheckerError {
    pub name: &'static str,
    pub problem: CheckerErrorProblem,
}

#[derive(Debug)]
pub enum CheckerErrorProblem {
    ConfigDeserializeError(Box<dyn Error>),
    RuleNewError(Box<dyn Error>),
}

#[derive(Default, Deserialize)]
#[serde(default)]
pub struct CheckerConfig<V> {
    pub config: HashMap<String, V>,
    pub rules: HashMap<String, RuleVariation>,
}

#[derive(Deserialize)]
#[serde(rename = "lowercase")]
pub enum RuleVariation {
    Allow,
    Deny,
    Warn,
}

macro_rules! use_rules {
    {
        $(
            $rule_name:ident: $rule_path:ty,
        )+
    } => {
        pub struct Checker {
            $(
                $rule_name: $rule_path,
            )+
        }

        impl Checker {
            // TODO: Be more strict about config? Make sure all keys exist
            pub fn from_config<V: 'static>(
                mut config: CheckerConfig<V>,
            ) -> Result<Self, CheckerError> where V: for<'de> Deserialize<'de> + for<'de> Deserializer<'de> {
                Ok(Self {
                    $(
                        $rule_name: <$rule_path>::new({
                            match config.config.remove(stringify!($rule_name)) {
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

            pub fn test_on(&self, ast: &Ast<'static>) -> Vec<Diagnostic> {
                let mut diagnostics = Vec::new();

                $(
                    diagnostics.extend(&mut (&self.$rule_name).pass(ast).into_iter());
                )+

                diagnostics
            }
        }
    };
}

use_rules! {
    empty_if: rules::empty_if::EmptyIfLint,
}
