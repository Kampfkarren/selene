use super::*;
use crate::ast_util::{is_type_function, range};
use std::convert::Infallible;

use full_moon::{
    ast::{self, Ast},
    visitors::Visitor,
};

pub struct TypeCheckInsideCallLint;

impl Lint for TypeCheckInsideCallLint {
    type Config = ();
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Error;
    const LINT_TYPE: LintType = LintType::Correctness;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(TypeCheckInsideCallLint)
    }

    fn pass(&self, ast: &Ast, context: &Context, _: &AstContext) -> Vec<Diagnostic> {
        let mut visitor = TypeCheckInsideCallVisitor {
            positions: Vec::new(),
            roblox: context.is_roblox(),
        };

        visitor.visit_ast(ast);

        visitor
            .positions
            .iter()
            .map(|position| {
                Diagnostic::new_complete(
                    "type_check_inside_call",
                    "you are checking the type inside the call, not outside".to_owned(),
                    Label::new(*position),
                    vec!["note: this will always return `boolean`".to_owned()],
                    Vec::new(),
                )
            })
            .collect()
    }
}

struct TypeCheckInsideCallVisitor {
    positions: Vec<(usize, usize)>,
    roblox: bool,
}

impl Visitor for TypeCheckInsideCallVisitor {
    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        if_chain::if_chain! {
            // Check that we're using type or typeof
            if let ast::Prefix::Name(name) = call.prefix();
            if is_type_function(&name.to_string(), self.roblox);

            // Check that we're calling it with an argument
            if let ast::Suffix::Call(call) = call.suffixes().next().unwrap();
            if let ast::Call::AnonymousCall(ast::FunctionArgs::Parentheses { arguments, .. }) = call;

            // Check that the argument, if it's there, is in the form of x == y
            if let Some(ast::Expression::BinaryOperator { binop: ast::BinOp::TwoEqual(_), rhs, .. })
                = arguments.iter().next();

            // Check that rhs is a constant string
            if let ast::Expression::Value { value: rhs_value, .. } = &**rhs;
            if let ast::Value::String(_) = &**rhs_value;

            then {
                self.positions.push(range(call));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_type_check_inside_call() {
        test_lint(
            TypeCheckInsideCallLint::new(()).unwrap(),
            "type_check_inside_call",
            "type_check_inside_call",
        );
    }
}
