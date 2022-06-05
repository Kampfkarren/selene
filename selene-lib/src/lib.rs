#![recursion_limit = "1000"]
#![cfg_attr(
    feature = "force_exhaustive_checks",
    feature(non_exhaustive_omitted_patterns_lint)
)]
use std::{collections::HashMap, error::Error, fmt};

use full_moon::ast::Ast;
use serde::{
    de::{DeserializeOwned, Deserializer},
    Deserialize,
};

mod ast_util;
mod lint_filtering;
pub mod rules;
pub mod standard_library;
mod util;

#[cfg(test)]
mod test_util;

use rules::{Context, Diagnostic, Rule, Severity};
use standard_library::StandardLibrary;

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
#[serde(rename_all = "kebab-case")]
pub struct CheckerConfig<V> {
    pub config: HashMap<String, V>,
    pub rules: HashMap<String, RuleVariation>,
    pub std: String,

    // Not locked behind Roblox feature so that selene.toml for Roblox will
    // run even without it.
    pub roblox_std_source: RobloxStdSource,
}

// #[derive(Default)] cannot be used since it binds V to Default
impl<V> Default for CheckerConfig<V> {
    fn default() -> Self {
        CheckerConfig {
            config: HashMap::new(),
            rules: HashMap::new(),
            std: "lua51".to_owned(),
            roblox_std_source: RobloxStdSource::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuleVariation {
    Allow,
    Deny,
    Warn,
}

impl RuleVariation {
    pub fn to_severity(self) -> Severity {
        match self {
            RuleVariation::Allow => Severity::Allow,
            RuleVariation::Deny => Severity::Error,
            RuleVariation::Warn => Severity::Warning,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RobloxStdSource {
    Floating,
    Pinned,
}

impl Default for RobloxStdSource {
    fn default() -> Self {
        Self::Floating
    }
}

macro_rules! use_rules {
    {
        $(
            $rule_name:ident: $rule_path:ty,
        )+

        $(
            #[$meta:meta]
            {
                $($meta_rule_name:ident: $meta_rule_path:ty,)+
            },
        )+
    } => {
        lazy_static::lazy_static! {
            static ref ALL_RULES: Vec<&'static str> = vec![
                $(
                    stringify!($rule_name),
                )+

                $(
                    $(
                        #[$meta]
                        stringify!($meta_rule_name),
                    )+
                )+
            ];
        }

        pub struct Checker<V: 'static + DeserializeOwned> {
            config: CheckerConfig<V>,
            context: Context,

            $(
                $rule_name: $rule_path,
            )+

            $(
                $(
                    #[$meta]
                    $meta_rule_name: $meta_rule_path,
                )+
            )+
        }

        impl<V: 'static + DeserializeOwned> Checker<V> {
            // TODO: Be more strict about config? Make sure all keys exist
            pub fn new(
                mut config: CheckerConfig<V>,
                standard_library: StandardLibrary,
            ) -> Result<Self, CheckerError> where V: for<'de> Deserializer<'de> {
                macro_rules! rule_field {
                    ($name:ident, $path:ty) => {{
                        let rule_name = stringify!($name);

                        let rule = <$path>::new({
                            match config.config.remove(rule_name) {
                                Some(entry_generic) => {
                                    <$path as Rule>::Config::deserialize(entry_generic).map_err(|error| {
                                        CheckerError {
                                            name: rule_name,
                                            problem: CheckerErrorProblem::ConfigDeserializeError(Box::new(error)),
                                        }
                                    })?
                                }

                                None => {
                                    <$path as Rule>::Config::default()
                                }
                            }
                        }).map_err(|error| {
                            CheckerError {
                                name: stringify!($name),
                                problem: CheckerErrorProblem::RuleNewError(Box::new(error)),
                            }
                        })?;

                        rule
                    }};
                }

                Ok(Self {
                    $(
                        $rule_name: {
                            rule_field!($rule_name, $rule_path)
                        },
                    )+
                    $(
                        $(
                            #[$meta]
                            $meta_rule_name: {
                                rule_field!($meta_rule_name, $meta_rule_path)
                            },
                        )+
                    )+
                    config,
                    context: Context {
                        standard_library,
                    },
                })
            }

            pub fn test_on(&self, ast: &Ast) -> Vec<CheckerDiagnostic> {
                let mut diagnostics = Vec::new();

                macro_rules! check_rule {
                    ($name:ident) => {
                        let rule = &self.$name;
                        diagnostics.extend(&mut rule.pass(ast, &self.context).into_iter().map(|diagnostic| {
                            CheckerDiagnostic {
                                diagnostic,
                                severity: self.get_lint_severity(rule, stringify!($name)),
                            }
                        }));
                    };
                }

                $(
                    check_rule!($rule_name);
                )+

                $(
                    $(
                        #[$meta]
                        {
                            check_rule!($meta_rule_name);
                        }
                    )+
                )+

                diagnostics = lint_filtering::filter_diagnostics(
                    ast,
                    diagnostics,
                    self.get_lint_severity(&self.invalid_lint_filter, "invalid_lint_filter"),
                );

                diagnostics
            }

            fn get_lint_severity<R: Rule>(&self, lint: &R, name: &'static str) -> Severity {
                match self.config.rules.get(name) {
                    Some(variation) => variation.to_severity(),
                    None => lint.severity(),
                }
            }
        }
    };
}

#[derive(Debug)]
pub struct CheckerDiagnostic {
    pub diagnostic: Diagnostic,
    pub severity: Severity,
}

pub fn rule_exists(name: &str) -> bool {
    ALL_RULES.contains(&name)
}

use_rules! {
    almost_swapped: rules::almost_swapped::AlmostSwappedLint,
    bad_string_escape: rules::bad_string_escape::BadStringEscapeLint,
    compare_nan: rules::compare_nan::CompareNanLint,
    divide_by_zero: rules::divide_by_zero::DivideByZeroLint,
    duplicate_keys: rules::duplicate_keys::DuplicateKeysLint,
    empty_if: rules::empty_if::EmptyIfLint,
    global_usage: rules::global_usage::GlobalLint,
    if_same_then_else: rules::if_same_then_else::IfSameThenElseLint,
    ifs_same_cond: rules::ifs_same_cond::IfsSameCondLint,
    incorrect_standard_library_use: rules::standard_library::StandardLibraryLint,
    invalid_lint_filter: rules::invalid_lint_filter::InvalidLintFilterLint,
    mismatched_arg_count: rules::mismatched_arg_count::MismatchedArgCountLint,
    multiple_statements: rules::multiple_statements::MultipleStatementsLint,
    parenthese_conditions: rules::parenthese_conditions::ParentheseConditionsLint,
    shadowing: rules::shadowing::ShadowingLint,
    suspicious_reverse_loop: rules::suspicious_reverse_loop::SuspiciousReverseLoopLint,
    type_check_inside_call: rules::type_check_inside_call::TypeCheckInsideCallLint,
    unbalanced_assignments: rules::unbalanced_assignments::UnbalancedAssignmentsLint,
    undefined_variable: rules::undefined_variable::UndefinedVariableLint,
    unscoped_variables: rules::unscoped_variables::UnscopedVariablesLint,
    unused_variable: rules::unused_variable::UnusedVariableLint,

    #[cfg(feature = "roblox")]
    {
        roblox_incorrect_color3_new_bounds: rules::roblox_incorrect_color3_new_bounds::Color3BoundsLint,
        roblox_incorrect_roact_usage: rules::roblox_incorrect_roact_usage::IncorrectRoactUsageLint,
    },
}
