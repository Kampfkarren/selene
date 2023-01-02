#![recursion_limit = "1000"]
#![cfg_attr(
    feature = "force_exhaustive_checks",
    feature(non_exhaustive_omitted_patterns_lint)
)]
use std::{
    collections::HashMap,
    error::Error,
    fmt,
    path::Path,
    sync::{Arc, Mutex},
};

use full_moon::ast::Ast;
use serde::{
    de::{DeserializeOwned, Deserializer},
    Deserialize,
};

mod ast_util;
mod lint_filtering;
pub mod lints;
pub mod logs;
mod plugins;
mod possible_std;
pub mod standard_library;
mod text;

#[cfg(test)]
mod test_util;

#[cfg(test)]
mod test_full_runs;

use lints::{AstContext, Context, Diagnostic, Lint, Severity};
use standard_library::StandardLibrary;

#[derive(Debug)]
pub enum CheckerError {
    ConfigDeserializeError {
        name: &'static str,
        problem: Box<dyn Error>,
    },

    InvalidPlugin(eyre::Error),

    LintNewError {
        name: &'static str,
        problem: Box<dyn Error>,
    },
}

impl fmt::Display for CheckerError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            CheckerError::ConfigDeserializeError { name, problem } => write!(
                formatter,
                "[{name}] Configuration was incorrectly formatted: {problem}",
            ),

            CheckerError::InvalidPlugin(error) => {
                write!(formatter, "Couldn't load plugin: {error:#}")
            }

            CheckerError::LintNewError { name, problem } => write!(formatter, "[{name}] {problem}"),
        }
    }
}

impl Error for CheckerError {}

#[derive(Deserialize)]
#[serde(default)]
#[serde(rename_all = "kebab-case")]
pub struct CheckerConfig<V> {
    pub config: HashMap<String, V>,
    // PLUGIN TODO: Make sure we don't specify multiple of the same hub
    pub plugins: Vec<plugins::config::PluginConfig>,
    #[serde(alias = "rules")]
    pub lints: HashMap<String, LintVariation>,
    pub std: Option<String>,
    pub exclude: Vec<String>,

    // Not locked behind Roblox feature so that selene.toml for Roblox will
    // run even without it.
    pub roblox_std_source: RobloxStdSource,
}

impl<V> CheckerConfig<V> {
    // PLUGIN TODO: Don't allow escaping base
    pub fn absolutize_paths(&mut self, base: &Path) {
        for plugin in &mut self.plugins {
            plugin.source = base.join(&plugin.source);
        }
    }

    // Necessary because #[derive(Default)] would bind V: Default
    pub fn std(&self) -> &str {
        self.std.as_deref().unwrap_or("lua51")
    }
}

impl<V> Default for CheckerConfig<V> {
    fn default() -> Self {
        CheckerConfig {
            config: HashMap::new(),

            lints: HashMap::new(),
            plugins: Vec::new(),
            std: None,
            exclude: Vec::new(),

            roblox_std_source: RobloxStdSource::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LintVariation {
    Allow,
    Deny,
    Warn,
}

impl LintVariation {
    pub fn to_severity(self) -> Severity {
        match self {
            LintVariation::Allow => Severity::Allow,
            LintVariation::Deny => Severity::Error,
            LintVariation::Warn => Severity::Warning,
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

pub struct Checker<V: 'static + DeserializeOwned> {
    config: CheckerConfig<V>,
    context: Context,
    plugins: Vec<plugins::LuaPlugin>,

    lints: Lints,
}

impl<V: 'static + DeserializeOwned> Checker<V> {
    // TODO: Be more strict about config? Make sure all keys exist
    pub fn new(
        mut config: CheckerConfig<V>,
        standard_library: StandardLibrary,
    ) -> Result<Self, CheckerError>
    where
        V: for<'de> Deserializer<'de>,
    {
        Ok(Self {
            context: Context {
                standard_library,
                user_set_standard_library: config
                    .std
                    .as_ref()
                    .map(|std_text| std_text.split('+').map(ToOwned::to_owned).collect()),
            },

            plugins: create_plugins_from_config(&config)?,

            lints: Lints::new(&mut config)?,

            config,
        })
    }

    pub fn test_on(&self, ast: &Ast) -> Vec<CheckerDiagnostic> {
        let ast_context = AstContext::from_ast(ast);
        let mut diagnostics = self.lints.test_on(ast, self, &ast_context, &self.context);

        self.run_plugins(&mut diagnostics, ast, &ast_context);

        diagnostics = lint_filtering::filter_diagnostics(
            ast,
            diagnostics,
            self.get_lint_severity(&self.lints.invalid_lint_filter, "invalid_lint_filter"),
        );

        diagnostics
    }

    pub fn get_lint_severity<L: Lint>(&self, _lint: &L, name: &'static str) -> Severity {
        match self.config.lints.get(name) {
            Some(variation) => variation.to_severity(),
            None => L::SEVERITY,
        }
    }

    fn run_plugins(
        &self,
        diagnostics: &mut Vec<CheckerDiagnostic>,
        ast: &Ast,
        ast_context: &AstContext,
    ) {
        if self.plugins.is_empty() {
            return;
        }

        let lua_ast = Arc::new(Mutex::new(full_moon_lua_types::Ast::from(ast)));

        for plugin in &self.plugins {
            let plugin_name = plugin.full_name();

            let plugin_pass = {
                profiling::scope!(plugin_name);
                plugin.pass(Arc::clone(&lua_ast), &self.context, ast_context)
            };

            match plugin_pass {
                Ok(plugin_diagnostics) => {
                    diagnostics.extend(&mut plugin_diagnostics.into_iter().map(|diagnostic| {
                        CheckerDiagnostic {
                            diagnostic,
                            severity: match self.config.lints.get(&plugin_name) {
                                Some(variation) => variation.to_severity(),
                                None => plugin.severity(),
                            },
                        }
                    }));
                }

                Err(error) => {
                    diagnostics.push(CheckerDiagnostic {
                        diagnostic: Diagnostic::new(
                            plugin_name,
                            format!("error running plugin: {error}"),
                            // PLUGIN TODO: Support not pointing at anything in particular (and allow lint(message) to do the same)
                            lints::Label::new((0, 0)),
                        ),
                        severity: Severity::Error,
                    });
                }
            }
        }
    }
}

macro_rules! use_lints {
    {
        $(
            $lint_name:ident: $lint_path:ty,
        )+

        $(
            #[$meta:meta]
            {
                $($meta_lint_name:ident: $meta_lint_path:ty,)+
            },
        )+
    } => {
        lazy_static::lazy_static! {
            static ref ALL_LINTS: Vec<&'static str> = vec![
                $(
                    stringify!($lint_name),
                )+

                $(
                    $(
                        #[$meta]
                        stringify!($meta_lint_name),
                    )+
                )+
            ];
        }

        pub struct Lints {
            $(
                pub $lint_name: $lint_path,
            )+

            $(
                $(
                    #[$meta]
                    pub $meta_lint_name: $meta_lint_path,
                )+
            )+
        }

        impl Lints {
            fn new<V: 'static + DeserializeOwned>(config: &mut CheckerConfig<V>) -> Result<Self, CheckerError>
            where
                V: for<'de> Deserializer<'de>,
            {
                macro_rules! lint_field {
                    ($name:ident, $path:ty) => {{
                        let lint_name = stringify!($name);

                        let lint = <$path>::new({
                            match config.config.remove(lint_name) {
                                Some(entry_generic) => {
                                    <$path as Lint>::Config::deserialize(entry_generic).map_err(|error| {
                                        CheckerError::ConfigDeserializeError {
                                            name: lint_name,
                                            problem: Box::new(error),
                                        }
                                    })?
                                }

                                None => {
                                    <$path as Lint>::Config::default()
                                }
                            }
                        }).map_err(|error| {
                            CheckerError::LintNewError {
                                name: stringify!($name),
                                problem: Box::new(error),
                            }
                        })?;

                        lint
                    }};
                }

                Ok(Self {
                    $(
                        $lint_name: {
                            lint_field!($lint_name, $lint_path)
                        },
                    )+
                    $(
                        $(
                            #[$meta]
                            $meta_lint_name: {
                                lint_field!($meta_lint_name, $meta_lint_path)
                            },
                        )+
                    )+
                })
            }

            fn test_on<V: 'static + DeserializeOwned>(&self, ast: &Ast, checker: &Checker<V>, ast_context: &AstContext, context: &Context) -> Vec<CheckerDiagnostic> {
                let mut diagnostics = Vec::new();

                macro_rules! check_lint {
                    ($name:ident) => {
                        let lint = &self.$name;

                        let lint_pass = {
                            profiling::scope!(&format!("lint: {}", stringify!($name)));
                            lint.pass(ast, context, ast_context)
                        };

                        diagnostics.extend(&mut lint_pass.into_iter().map(|diagnostic| {
                            CheckerDiagnostic {
                                diagnostic,
                                severity: checker.get_lint_severity(lint, stringify!($name)),
                            }
                        }));
                    };
                }

                $(
                    check_lint!($lint_name);
                )+

                $(
                    $(
                        #[$meta]
                        {
                            check_lint!($meta_lint_name);
                        }
                    )+
                )+

                diagnostics
            }
        }
    };
}

// PLUGIN TODO: Async to support multiple downloads at once?
fn create_plugins_from_config<V>(
    config: &CheckerConfig<V>,
) -> Result<Vec<plugins::LuaPlugin>, CheckerError> {
    let mut plugins = Vec::new();

    for plugin_config in &config.plugins {
        plugins.append(
            &mut plugins::load_plugins_from_config(plugin_config)
                .map_err(CheckerError::InvalidPlugin)?,
        );
    }

    Ok(plugins)
}

#[derive(Debug)]
pub struct CheckerDiagnostic {
    pub diagnostic: Diagnostic,
    pub severity: Severity,
}

pub fn lint_exists(name: &str) -> bool {
    ALL_LINTS.contains(&name)
}

use_lints! {
    almost_swapped: lints::almost_swapped::AlmostSwappedLint,
    bad_string_escape: lints::bad_string_escape::BadStringEscapeLint,
    compare_nan: lints::compare_nan::CompareNanLint,
    constant_table_comparison: lints::constant_table_comparison::ConstantTableComparisonLint,
    deprecated: lints::deprecated::DeprecatedLint,
    divide_by_zero: lints::divide_by_zero::DivideByZeroLint,
    duplicate_keys: lints::duplicate_keys::DuplicateKeysLint,
    empty_if: lints::empty_if::EmptyIfLint,
    global_usage: lints::global_usage::GlobalLint,
    high_cyclomatic_complexity: lints::high_cyclomatic_complexity::HighCyclomaticComplexityLint,
    if_same_then_else: lints::if_same_then_else::IfSameThenElseLint,
    ifs_same_cond: lints::ifs_same_cond::IfsSameCondLint,
    incorrect_standard_library_use: lints::standard_library::StandardLibraryLint,
    invalid_lint_filter: lints::invalid_lint_filter::InvalidLintFilterLint,
    manual_table_clone: lints::manual_table_clone::ManualTableCloneLint,
    mismatched_arg_count: lints::mismatched_arg_count::MismatchedArgCountLint,
    multiple_statements: lints::multiple_statements::MultipleStatementsLint,
    must_use: lints::must_use::MustUseLint,
    parenthese_conditions: lints::parenthese_conditions::ParentheseConditionsLint,
    shadowing: lints::shadowing::ShadowingLint,
    suspicious_reverse_loop: lints::suspicious_reverse_loop::SuspiciousReverseLoopLint,
    type_check_inside_call: lints::type_check_inside_call::TypeCheckInsideCallLint,
    unbalanced_assignments: lints::unbalanced_assignments::UnbalancedAssignmentsLint,
    undefined_variable: lints::undefined_variable::UndefinedVariableLint,
    unscoped_variables: lints::unscoped_variables::UnscopedVariablesLint,
    unused_variable: lints::unused_variable::UnusedVariableLint,

    #[cfg(feature = "roblox")]
    {
        roblox_incorrect_color3_new_bounds: lints::roblox_incorrect_color3_new_bounds::Color3BoundsLint,
        roblox_incorrect_roact_usage: lints::roblox_incorrect_roact_usage::IncorrectRoactUsageLint,
    },
}
