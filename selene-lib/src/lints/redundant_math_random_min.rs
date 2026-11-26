use super::*;
use crate::ast_util::{range, scopes::ScopeManager};
use std::convert::Infallible;

use full_moon::{
    ast::{self, Ast},
    visitors::Visitor,
};

pub struct RedundantMathRandomMinLint;

impl Lint for RedundantMathRandomMinLint {
    type Config = ();
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Warning;
    const LINT_TYPE: LintType = LintType::Style;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(RedundantMathRandomMinLint)
    }

    fn pass(&self, ast: &Ast, _: &Context, ast_context: &AstContext) -> Vec<Diagnostic> {
        let mut visitor = RedundantMathRandomMinVisitor {
            positions: Vec::new(),
            scope_manager: &ast_context.scope_manager,
        };

        visitor.visit_ast(ast);

        visitor
            .positions
            .into_iter()
            .map(|(position, upper)| {
                Diagnostic::new_complete(
                    "redundant_math_random_min",
                    "`math.random(1, n)` is the same as `math.random(n)`".to_owned(),
                    Label::new(position),
                    vec![format!("try: `math.random({upper})`")],
                    Vec::new(),
                )
            })
            .collect()
    }
}

struct RedundantMathRandomMinVisitor<'a> {
    positions: Vec<((usize, usize), String)>,
    scope_manager: &'a ScopeManager,
}

impl Visitor for RedundantMathRandomMinVisitor<'_> {
    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        if_chain::if_chain! {
            if let ast::Prefix::Name(token) = call.prefix();
            if token.token().to_string() == "math";

            // Make sure `math` is the global library, not a local the user defined
            if !matches!(
                self.scope_manager.reference_at_byte(token.token().start_position().bytes()),
                Some(reference) if reference.resolved.is_some()
            );

            let mut suffixes = call.suffixes();
            if let Some(ast::Suffix::Index(ast::Index::Dot { name, .. })) = suffixes.next();
            if name.token().to_string() == "random";

            if let Some(ast::Suffix::Call(ast::Call::AnonymousCall(function_args))) = suffixes.next();
            if suffixes.next().is_none();
            if let ast::FunctionArgs::Parentheses { arguments, .. } = &**function_args;
            if arguments.len() == 2;

            let mut arguments = arguments.iter();
            if let Some(ast::Expression::Number(min)) = arguments.next();
            if min.token().to_string().parse::<f64>() == Ok(1.0);
            if let Some(upper) = arguments.next();

            then {
                self.positions.push((range(call), upper.to_string().trim().to_owned()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_redundant_math_random_min() {
        test_lint(
            RedundantMathRandomMinLint::new(()).unwrap(),
            "redundant_math_random_min",
            "redundant_math_random_min",
        );
    }
}
