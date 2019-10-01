use std::{collections::HashMap, error::Error, fmt};

use full_moon::ast::Ast;
use serde::{de::{Deserializer, DeserializeOwned}, Deserialize};

pub mod rules;

use rules::{Diagnostic, Rule, Severity};

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

impl fmt::Display for CheckerError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use CheckerErrorProblem::*;

        write!(formatter, "[{}] ", self.name)?;

        match &self.problem {
            ConfigDeserializeError(error) => write!(
                formatter,
                "Configuration was incorrectly formatted: {}",
                error
            ),
            RuleNewError(error) => write!(formatter, "{}", error),
        }
    }
}

impl Error for CheckerError {}

#[derive(Deserialize)]
#[serde(default)]
pub struct CheckerConfig<V> {
    pub config: HashMap<String, V>,
    pub rules: HashMap<String, RuleVariation>,
}

// #[derive(Default)] cannot be used since it binds V to Default
impl<V> Default for CheckerConfig<V> {
    fn default() -> Self {
        CheckerConfig {
            config: HashMap::new(),
            rules: HashMap::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
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
        pub struct Checker<V: 'static + DeserializeOwned> {
            config: CheckerConfig<V>,

            $(
                $rule_name: Option<$rule_path>,
            )+
        }

        impl<V: 'static + DeserializeOwned> Checker<V> {
            // TODO: Be more strict about config? Make sure all keys exist
            pub fn from_config(
                mut config: CheckerConfig<V>,
            ) -> Result<Self, CheckerError> where V: for<'de> Deserializer<'de> {
                Ok(Self {
                    $(
                        $rule_name: {
                            let rule_name = stringify!($rule_name);
                            let variation = config.rules.get(rule_name);

                            if variation != Some(&RuleVariation::Allow) {
                                Some(<$rule_path>::new({
                                    match config.config.remove(rule_name) {
                                        Some(entry_generic) => {
                                            <$rule_path as Rule>::Config::deserialize(entry_generic).map_err(|error| {
                                                CheckerError {
                                                    name: rule_name,
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
                                })?)
                            } else {
                                None
                            }
                        },
                    )+
                    config,
                })
            }

            pub fn test_on(&self, ast: &Ast<'static>) -> Vec<CheckerDiagnostic> {
                let mut diagnostics = Vec::new();

                $(
                    if let Some(rule) = &self.$rule_name {
                        diagnostics.extend(&mut rule.pass(ast).into_iter().map(|diagnostic| {
                            CheckerDiagnostic {
                                diagnostic,
                                severity: match self.config.rules.get(stringify!($rule_name)) {
                                    None => rule.severity(),
                                    Some(RuleVariation::Deny) => Severity::Error,
                                    Some(RuleVariation::Warn) => Severity::Warning,
                                    Some(RuleVariation::Allow) => unreachable!(),
                                }
                            }
                        }));
                    }
                )+

                diagnostics
            }
        }
    };
}

pub struct CheckerDiagnostic {
    pub diagnostic: Diagnostic,
    pub severity: Severity,
}

use_rules! {
    empty_if: rules::empty_if::EmptyIfLint,
}
