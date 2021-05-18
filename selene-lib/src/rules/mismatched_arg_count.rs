use super::*;
use crate::ast_util::{
    range,
    scopes::{ScopeManager, Variable},
    HasSideEffects,
};
use std::{
    collections::HashMap,
    convert::Infallible,
    fmt::{self, Display},
    usize,
};

use full_moon::{
    ast::{self, Ast},
    visitors::Visitor,
};
use id_arena::Id;

pub struct MismatchedArgCountLint;

impl Rule for MismatchedArgCountLint {
    type Config = ();
    type Error = Infallible;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(MismatchedArgCountLint)
    }

    fn pass(&self, ast: &Ast, _context: &Context) -> Vec<Diagnostic> {
        let scope_manager = ScopeManager::new(ast);

        // Firstly visit the AST so we can map the variables to their required parameter counts
        let mut definitions = HashMap::new();
        let mut definitions_visitor = MapFunctionDefinitionVisitor {
            scope_manager: &scope_manager,
            definitions: &mut definitions,
        };
        definitions_visitor.visit_ast(&ast);

        let mut visitor = MismatchedArgCountVisitor {
            mismatched_arg_counts: Vec::new(),
            scope_manager,
            definitions,
        };

        visitor.visit_ast(&ast);

        visitor
            .mismatched_arg_counts
            .iter()
            .map(|mismatched_arg| {
                Diagnostic::new_complete(
                    "mismatched_arg_count",
                    format!(
                        "this function takes {}",
                        mismatched_arg
                            .parameter_count
                            .to_message(mismatched_arg.num_provided)
                    ),
                    Label::new_with_message(
                        mismatched_arg.call_range,
                        mismatched_arg.parameter_count.to_string(),
                    ),
                    Vec::new(),
                    vec![Label::new_with_message(
                        mismatched_arg.function_definition_range,
                        "note: function defined here".to_owned(),
                    )],
                )
            })
            .collect()
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn rule_type(&self) -> RuleType {
        RuleType::Correctness
    }
}

struct MismatchedArgCount {
    parameter_count: ParameterCount,
    num_provided: PassedArgumentCount,
    call_range: (usize, usize),
    function_definition_range: (usize, usize),
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
    pub fn from_function_body(function_body: &ast::FunctionBody) -> Self {
        let mut necessary_params = 0;

        for parameter in function_body.parameters() {
            match parameter {
                ast::Parameter::Name(_) => necessary_params += 1,
                ast::Parameter::Ellipse(_) => {
                    if necessary_params == 0 {
                        return Self::Variable;
                    } else {
                        return Self::Minimum(necessary_params);
                    }
                }
            }
        }

        Self::Fixed(necessary_params)
    }

    /// Checks the provided number of arguments to see if it satisfies the number of arguments required
    /// We will only lint an upper bound. If we have a function(a, b, c) and we call foo(a, b), this will
    /// pass the lint, since the `nil` could be implicitly provided.
    pub fn correct_num_args_provided(&self, provided: PassedArgumentCount) -> bool {
        match self {
            ParameterCount::Fixed(required) => match provided {
                PassedArgumentCount::Fixed(provided) => provided <= *required,
                // If we have function(a, b, c), but we provide foo(a, call()), we cannot infer anything
                // but if we provide foo(a, b, c, call()), we know we have too many
                PassedArgumentCount::Variable(atleast_provided) => atleast_provided <= *required,
            },
            // function(a, b, ...) - if we call it through foo(a), b and the varargs could be implicitly nil.
            // there is no upper bound since foo(a, b, c, d) is valid - therefore any amount of arguments provided is valid
            ParameterCount::Minimum(_) => true,
            // Any amount of arguments could be provided
            ParameterCount::Variable => true,
        }
    }

    pub fn to_message(&self, provided: PassedArgumentCount) -> String {
        match self {
            ParameterCount::Fixed(required) => {
                format!("{} arguments but {} were supplied", required, provided)
            }
            ParameterCount::Minimum(required) => format!(
                "atleast {} arguments but {} were supplied",
                required, provided
            ),
            ParameterCount::Variable => "a variable amount of arguments".to_owned(),
        }
    }
}
impl Display for ParameterCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParameterCount::Fixed(required) => write!(f, "expected {} arguments", required),
            ParameterCount::Minimum(required) => {
                write!(f, "expected atleast {} arguments", required)
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
impl Display for PassedArgumentCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PassedArgumentCount::Fixed(amount) => write!(f, "{} arguments", amount),
            PassedArgumentCount::Variable(amount) => write!(f, "atleast {} arguments", amount),
        }
    }
}

fn function_call_argument_count(function_args: &ast::FunctionArgs) -> PassedArgumentCount {
    match function_args {
        ast::FunctionArgs::Parentheses { arguments, .. } => {
            // We need to be wary of items with side effects, such as function calls or ... being the last argument passed
            // e.g. foo(a, b, call()) or foo(a, b, ...) - we don't know how many arguments were passed.
            // However, if the call is NOT the last argument, as per Lua semantics, it is only classed as one argument, and no side effects occur
            // e.g. foo(a, call(), b) or foo(a, ..., c)

            let mut passed_argument_count = 0;

            for argument in arguments.pairs() {
                match argument {
                    ast::punctuated::Pair::Punctuated(_, _) => passed_argument_count += 1,
                    ast::punctuated::Pair::End(expression) => {
                        passed_argument_count += 1;
                        if expression.has_side_effects() {
                            return PassedArgumentCount::Variable(passed_argument_count);
                        }
                    }
                }
            }

            PassedArgumentCount::Fixed(passed_argument_count)
        }
        ast::FunctionArgs::String(_) => PassedArgumentCount::Fixed(1),
        ast::FunctionArgs::TableConstructor(_) => PassedArgumentCount::Fixed(1),
    }
}

/// A visitor used to map a variable to the necessary number of parameters required
struct MapFunctionDefinitionVisitor<'a> {
    scope_manager: &'a ScopeManager,
    definitions: &'a mut HashMap<Id<Variable>, ParameterCount>,
}
impl Visitor<'_> for MapFunctionDefinitionVisitor<'_> {
    fn visit_local_function(&mut self, function: &ast::LocalFunction<'_>) {
        let identifier = range(function.name());
        let variable = self
            .scope_manager
            .variables
            .iter()
            .find(|v| v.1.identifiers.contains(&identifier));

        if let Some((id, _)) = variable {
            self.definitions
                .insert(id, ParameterCount::from_function_body(function.func_body()));
        }
    }

    fn visit_function_declaration(&mut self, function: &ast::FunctionDeclaration<'_>) {
        let identifier = range(function.name());
        let variable = self
            .scope_manager
            .variables
            .iter()
            .find(|v| v.1.identifiers.contains(&identifier));

        if let Some((id, _)) = variable {
            self.definitions
                .insert(id, ParameterCount::from_function_body(function.body()));
        }
    }

    fn visit_local_assignment(&mut self, local_assignment: &ast::LocalAssignment) {
        let mut expressions = local_assignment.expr_list().iter();

        for name_token in local_assignment.name_list() {
            let expression = expressions.next();

            if let Some(ast::Expression::Value { value, .. }) = expression {
                if let ast::Value::Function((_, function_body)) = &**value {
                    let identifier = range(name_token);
                    let variable = self
                        .scope_manager
                        .variables
                        .iter()
                        .find(|v| v.1.identifiers.contains(&identifier));

                    if let Some((id, _)) = variable {
                        self.definitions
                            .insert(id, ParameterCount::from_function_body(&function_body));
                    }
                }
            }
        }
    }

    fn visit_assignment(&mut self, assignment: &ast::Assignment) {
        let mut expressions = assignment.expr_list().iter();

        for var in assignment.var_list() {
            let expression = expressions.next();

            if let Some(ast::Expression::Value { value, .. }) = expression {
                if let ast::Value::Function((_, function_body)) = &**value {
                    let identifier = range(var);
                    let variable = self
                        .scope_manager
                        .variables
                        .iter()
                        .find(|v| v.1.identifiers.contains(&identifier));

                    if let Some((id, _)) = variable {
                        self.definitions
                            .insert(id, ParameterCount::from_function_body(&function_body));
                    }
                }
            }
        }
    }
}

struct MismatchedArgCountVisitor {
    mismatched_arg_counts: Vec<MismatchedArgCount>,
    scope_manager: ScopeManager,
    definitions: HashMap<Id<Variable>, ParameterCount>,
}

impl Visitor<'_> for MismatchedArgCountVisitor {
    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        if_chain::if_chain! {
            // Check that we're using a named function call, with an anonymous call suffix
            if let ast::Prefix::Name(name) = call.prefix();
            if let ast::Suffix::Call(ast::Call::AnonymousCall(args)) = call.iter_suffixes().next().unwrap();

            // Find the corresponding function definition
            let identifier = range(name);
            if let Some((_, reference)) = self.scope_manager.references.iter().find(|r| r.1.identifier == identifier);
            if let Some(defined_variable) = reference.resolved;
            if let Some(parameter_count) = self.definitions.get(&defined_variable);

            // Count the number of arguments provided
            let num_args_provided = function_call_argument_count(args);
            if !parameter_count.correct_num_args_provided(num_args_provided);

            then {
                self.mismatched_arg_counts.push(MismatchedArgCount {
                    num_provided: num_args_provided,
                    parameter_count: *parameter_count,
                    call_range: identifier,
                    function_definition_range: self.scope_manager.variables.get(defined_variable).unwrap().identifiers[0],
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
    fn test_mismatched_vararg_function_def() {
        test_lint(
            MismatchedArgCountLint::new(()).unwrap(),
            "mismatched_arg_count",
            "variable_function_def",
        );
    }

    #[test]
    fn test_mismatched_call_side_effects() {
        test_lint(
            MismatchedArgCountLint::new(()).unwrap(),
            "mismatched_arg_count",
            "call_side_effects",
        );
    }

    #[test]
    fn test_mismatched_args_alt_definition() {
        test_lint(
            MismatchedArgCountLint::new(()).unwrap(),
            "mismatched_arg_count",
            "alternative_function_definition",
        );
    }

    #[test]
    fn test_mismatched_args_shadowing_definition() {
        test_lint(
            MismatchedArgCountLint::new(()).unwrap(),
            "mismatched_arg_count",
            "shadowing_variables",
        );
    }

    #[test]
    fn test_mismatched_args_reassigned_definition() {
        test_lint(
            MismatchedArgCountLint::new(()).unwrap(),
            "mismatched_arg_count",
            "reassigned_variables",
        );
    }
}
