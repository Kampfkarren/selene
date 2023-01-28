use super::*;
use crate::{
    ast_util::{
        is_function_call, is_vararg, range,
        scopes::{Reference, ScopeManager, Variable},
    },
    text::plural,
};
use std::{
    collections::HashMap,
    convert::Infallible,
    fmt::{self, Display},
};

use full_moon::{
    ast::{self, Ast},
    visitors::Visitor,
};
use id_arena::Id;

pub struct MismatchedArgCountLint;

impl Lint for MismatchedArgCountLint {
    type Config = ();
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Error;
    const LINT_TYPE: LintType = LintType::Correctness;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(MismatchedArgCountLint)
    }

    fn pass(&self, ast: &Ast, _: &Context, ast_context: &AstContext) -> Vec<Diagnostic> {
        // Firstly visit the AST so we can map the variables to their required parameter counts
        let mut definitions = HashMap::new();
        let mut definitions_visitor = MapFunctionDefinitionVisitor {
            scope_manager: &ast_context.scope_manager,
            definitions: &mut definitions,
        };
        definitions_visitor.visit_ast(ast);

        let mut visitor = MismatchedArgCountVisitor {
            mismatched_arg_counts: Vec::new(),
            scope_manager: &ast_context.scope_manager,
            definitions,
        };

        visitor.visit_ast(ast);

        visitor
            .mismatched_arg_counts
            .iter()
            .map(|mismatched_arg| {
                Diagnostic::new_complete(
                    "mismatched_arg_count",
                    mismatched_arg
                        .parameter_count
                        .to_message(mismatched_arg.num_provided),
                    Label::new_with_message(
                        mismatched_arg.call_range,
                        mismatched_arg.parameter_count.to_string(),
                    ),
                    Vec::new(),
                    mismatched_arg
                        .function_definition_ranges
                        .iter()
                        .map(|range| {
                            Label::new_with_message(
                                *range,
                                "note: function defined here".to_owned(),
                            )
                        })
                        .collect(),
                )
            })
            .collect()
    }
}

struct MismatchedArgCount {
    parameter_count: ParameterCount,
    num_provided: PassedArgumentCount,
    call_range: (usize, usize),
    function_definition_ranges: Vec<(usize, usize)>,
}

#[derive(Clone, Copy, Debug)]
enum ParameterCount {
    /// A fixed number of parameters are required: `function(a, b, c)`
    Fixed(usize),
    /// Some amount of fixed parameters are required, and the rest are variable: `function(a, b, ...)`
    Minimum(usize),
    /// A variable number of parameters can be provided: `function(...)`
    Variable,
}

impl ParameterCount {
    /// Calculates the number of required parameters that must be passed to a function
    fn from_function_body(function_body: &ast::FunctionBody) -> Self {
        let mut necessary_params = 0;

        for parameter in function_body.parameters() {
            #[cfg_attr(
                feature = "force_exhaustive_checks",
                deny(non_exhaustive_omitted_patterns)
            )]
            match parameter {
                ast::Parameter::Name(_) => necessary_params += 1,
                ast::Parameter::Ellipse(_) => {
                    if necessary_params == 0 {
                        return Self::Variable;
                    } else {
                        return Self::Minimum(necessary_params);
                    }
                }
                _ => {}
            }
        }

        Self::Fixed(necessary_params)
    }

    /// Checks the provided number of arguments to see if it satisfies the number of arguments required
    /// We will only lint an upper bound. If we have a function(a, b, c) and we call foo(a, b), this will
    /// pass the lint, since the `nil` could be implicitly provided.
    fn correct_num_args_provided(self, provided: PassedArgumentCount) -> bool {
        match self {
            ParameterCount::Fixed(required) => match provided {
                PassedArgumentCount::Fixed(provided) => provided <= required,
                // If we have function(a, b, c), but we provide foo(a, call()), we cannot infer anything
                // but if we provide foo(a, b, c, call()), we know we have too many
                PassedArgumentCount::Variable(atleast_provided) => atleast_provided <= required,
            },
            // function(a, b, ...) - if we call it through foo(a), b and the varargs could be implicitly nil.
            // there is no upper bound since foo(a, b, c, d) is valid - therefore any amount of arguments provided is valid
            ParameterCount::Minimum(_) => true,
            // Any amount of arguments could be provided
            ParameterCount::Variable => true,
        }
    }

    fn to_message(self, provided: PassedArgumentCount) -> String {
        match self {
            ParameterCount::Fixed(required) => {
                format!(
                    "this function takes {} {} but {} were supplied",
                    required,
                    plural(required, "argument", "arguments"),
                    provided
                )
            }
            ParameterCount::Minimum(required) => format!(
                "this function takes at least {} {} but {} were supplied",
                required,
                plural(required, "argument", "arguments"),
                provided
            ),
            ParameterCount::Variable => "a variable amount of arguments".to_owned(),
        }
    }

    fn overlap_with_other_parameter_count(self, other: ParameterCount) -> ParameterCount {
        match (self, other) {
            // If something takes `...`, then it'll always be correct no matter what.
            (ParameterCount::Variable, _) | (_, ParameterCount::Variable) => {
                ParameterCount::Variable
            }

            // Minimum always wins, since it allows for infinite parameters, and fixed will always match.
            // f(a, b, ...) vs. f(a) is Minimum(1), so that `f(1, 2, 3, 4)` passes.
            // f(a, b, c) vs. f(a, ...) is Minimum(1) for the same reason.
            (ParameterCount::Fixed(fixed), ParameterCount::Minimum(minimum))
            | (ParameterCount::Minimum(minimum), ParameterCount::Fixed(fixed)) => {
                ParameterCount::Minimum(minimum.min(fixed))
            }

            // Given `f(a, b)` and `f(c, d)`, just preserve the Fixed(2).
            // The complication comes with `f(a)` and `f(b, c)`, where we change to Minimum(1).
            (ParameterCount::Fixed(this_fixed), ParameterCount::Fixed(other_fixed)) => {
                if this_fixed == other_fixed {
                    ParameterCount::Fixed(this_fixed)
                } else {
                    ParameterCount::Fixed(this_fixed.max(other_fixed))
                }
            }

            // `f(a, b, ...)` vs. `f(a, ...)`. Same lints apply, just preserve the smaller minimum.
            (ParameterCount::Minimum(this_minimum), ParameterCount::Minimum(other_minimum)) => {
                ParameterCount::Minimum(this_minimum.min(other_minimum))
            }
        }
    }
}

impl Display for ParameterCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParameterCount::Fixed(required) => write!(
                f,
                "expected {} {}",
                required,
                plural(*required, "argument", "arguments")
            ),
            ParameterCount::Minimum(required) => {
                write!(
                    f,
                    "expected at least {} {}",
                    required,
                    plural(*required, "argument", "arguments")
                )
            }
            ParameterCount::Variable => write!(f, "expected any number of arguments"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum PassedArgumentCount {
    /// Passed a fixed amount of arguments, such as foo(a, b, c) or foo(a, call(), c) or foo(a, ..., c)
    Fixed(usize),
    /// Passed a variable of arguments - but we know the lower bound: e.g. foo(a, b, call()) or foo(a, b, ...)
    Variable(usize),
}

impl PassedArgumentCount {
    fn from_function_args(function_args: &ast::FunctionArgs) -> Self {
        match function_args {
            ast::FunctionArgs::Parentheses { arguments, .. } => {
                // We need to be wary of function calls or ... being the last argument passed
                // e.g. foo(a, b, call()) or foo(a, b, ...) - we don't know how many arguments were passed.
                // However, if the call is NOT the last argument, as per Lua semantics, it is only classed as one argument,
                // e.g. foo(a, call(), b) or foo(a, ..., c)

                let mut passed_argument_count = 0;

                for argument in arguments.pairs() {
                    passed_argument_count += 1;

                    if let ast::punctuated::Pair::End(expression) = argument {
                        if is_function_call(expression) || is_vararg(expression) {
                            return PassedArgumentCount::Variable(passed_argument_count);
                        }
                    }
                }

                Self::Fixed(passed_argument_count)
            }
            ast::FunctionArgs::String(_) => Self::Fixed(1),
            ast::FunctionArgs::TableConstructor(_) => Self::Fixed(1),
            _ => Self::Fixed(0),
        }
    }
}

impl Display for PassedArgumentCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PassedArgumentCount::Fixed(amount) => write!(f, "{amount} arguments"),
            PassedArgumentCount::Variable(amount) => write!(f, "at least {amount} arguments"),
        }
    }
}

/// A visitor used to map a variable to the necessary number of parameters required
struct MapFunctionDefinitionVisitor<'a> {
    scope_manager: &'a ScopeManager,
    definitions: &'a mut HashMap<Id<Variable>, ParameterCount>,
}

impl MapFunctionDefinitionVisitor<'_> {
    fn find_variable(&self, identifier: (usize, usize)) -> Option<Id<Variable>> {
        self.scope_manager
            .variables
            .iter()
            .find(|variable| variable.1.identifiers.contains(&identifier))
            .map(|variable| variable.0)
    }

    fn find_reference(&self, identifier: (usize, usize)) -> Option<&Reference> {
        self.scope_manager
            .references
            .iter()
            .find(|reference| reference.1.identifier == identifier)
            .map(|reference| reference.1)
    }

    /// Checks the provided variable to see if it is blacklisted, or it has already been stored.
    /// If so, we can no longer verify which function definition is correct for a specific function call
    /// so we bail out and blacklist it. This does not apply to locally assignment/reassigned variables (i.e. shadowing),
    /// as that is handled properly.
    /// If it is safe to use, the function body is stored.
    fn verify_assignment(&mut self, variable: Id<Variable>, function_body: &ast::FunctionBody) {
        let parameter_count = ParameterCount::from_function_body(function_body);

        self.definitions
            .entry(variable)
            .and_modify(|older_count| {
                *older_count = parameter_count.overlap_with_other_parameter_count(*older_count)
            })
            .or_insert(parameter_count);
    }
}

impl Visitor for MapFunctionDefinitionVisitor<'_> {
    fn visit_local_function(&mut self, function: &ast::LocalFunction) {
        let identifier = range(function.name());

        if let Some(id) = self.find_variable(identifier) {
            self.definitions
                .insert(id, ParameterCount::from_function_body(function.body()));
        }
    }

    fn visit_function_declaration(&mut self, function: &ast::FunctionDeclaration) {
        let identifier = range(function.name());

        if let Some(reference) = self.find_reference(identifier) {
            if let Some(variable) = reference.resolved {
                self.verify_assignment(variable, function.body())
            }
        }
    }

    fn visit_local_assignment(&mut self, local_assignment: &ast::LocalAssignment) {
        let assignment_expressions = local_assignment
            .names()
            .iter()
            .zip(local_assignment.expressions());

        for (name_token, expression) in assignment_expressions {
            if let ast::Expression::Value { value, .. } = expression {
                if let ast::Value::Function((_, function_body)) = &**value {
                    let identifier = range(name_token);

                    if let Some(id) = self.find_variable(identifier) {
                        self.definitions
                            .insert(id, ParameterCount::from_function_body(function_body));
                    }
                }
            }
        }
    }

    fn visit_assignment(&mut self, assignment: &ast::Assignment) {
        let assignment_expressions = assignment.variables().iter().zip(assignment.expressions());

        for (var, expression) in assignment_expressions {
            if let ast::Expression::Value { value, .. } = expression {
                if let ast::Value::Function((_, function_body)) = &**value {
                    let identifier = range(var);

                    if let Some(reference) = self.find_reference(identifier) {
                        if let Some(variable) = reference.resolved {
                            self.verify_assignment(variable, function_body)
                        }
                    }
                }
            }
        }
    }
}

struct MismatchedArgCountVisitor<'a> {
    mismatched_arg_counts: Vec<MismatchedArgCount>,
    scope_manager: &'a ScopeManager,
    definitions: HashMap<Id<Variable>, ParameterCount>,
}

impl MismatchedArgCountVisitor<'_> {
    // Split off since the formatter doesn't work inside if_chain.
    fn get_function_definiton_ranges(&self, defined_variable: Id<Variable>) -> Vec<(usize, usize)> {
        let variable = self.scope_manager.variables.get(defined_variable).unwrap();

        variable
            .definitions
            .iter()
            .copied()
            .chain(variable.references.iter().filter_map(|reference_id| {
                let reference = self.scope_manager.references.get(*reference_id)?;
                if reference.write.is_some() {
                    Some(reference.identifier)
                } else {
                    None
                }
            }))
            .collect()
    }
}

impl Visitor for MismatchedArgCountVisitor<'_> {
    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        if_chain::if_chain! {
            // Check that we're using a named function call, with an anonymous call suffix
            if let ast::Prefix::Name(name) = call.prefix();
            if let Some(ast::Suffix::Call(ast::Call::AnonymousCall(args))) = call.suffixes().next();

            // Find the corresponding function definition
            let identifier = range(name);
            if let Some((_, reference)) = self.scope_manager.references.iter().find(|reference| reference.1.identifier == identifier);
            if let Some(defined_variable) = reference.resolved;
            if let Some(parameter_count) = self.definitions.get(&defined_variable);

            // Count the number of arguments provided
            let num_args_provided = PassedArgumentCount::from_function_args(args);
            if !parameter_count.correct_num_args_provided(num_args_provided);

            then {
                self.mismatched_arg_counts.push(MismatchedArgCount {
                    num_provided: num_args_provided,
                    parameter_count: *parameter_count,
                    call_range: range(call),
                    function_definition_ranges: self.get_function_definiton_ranges(defined_variable),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_mismatched_arg_count() {
        test_lint(
            MismatchedArgCountLint::new(()).unwrap(),
            "mismatched_arg_count",
            "mismatched_arg_count",
        );
    }

    #[test]
    fn test_vararg_function_def() {
        test_lint(
            MismatchedArgCountLint::new(()).unwrap(),
            "mismatched_arg_count",
            "variable_function_def",
        );
    }

    #[test]
    fn test_call_side_effects() {
        test_lint(
            MismatchedArgCountLint::new(()).unwrap(),
            "mismatched_arg_count",
            "call_side_effects",
        );
    }

    #[test]
    fn test_args_alt_definition() {
        test_lint(
            MismatchedArgCountLint::new(()).unwrap(),
            "mismatched_arg_count",
            "alternative_function_definition",
        );
    }

    #[test]
    fn test_args_shadowing_variables() {
        test_lint(
            MismatchedArgCountLint::new(()).unwrap(),
            "mismatched_arg_count",
            "shadowing_variables",
        );
    }

    #[test]
    fn test_args_reassigned_variables() {
        test_lint(
            MismatchedArgCountLint::new(()).unwrap(),
            "mismatched_arg_count",
            "reassigned_variables",
        );
    }

    #[test]
    fn test_args_reassigned_variables_2() {
        test_lint(
            MismatchedArgCountLint::new(()).unwrap(),
            "mismatched_arg_count",
            "reassigned_variables_2",
        );
    }

    #[test]
    fn test_definition_location() {
        test_lint(
            MismatchedArgCountLint::new(()).unwrap(),
            "mismatched_arg_count",
            "definition_location",
        );
    }

    #[test]
    fn test_multiple_definition_locations() {
        test_lint(
            MismatchedArgCountLint::new(()).unwrap(),
            "mismatched_arg_count",
            "multiple_definition_locations",
        );
    }
}
