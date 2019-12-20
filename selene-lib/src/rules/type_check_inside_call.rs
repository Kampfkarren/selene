use super::*;
use crate::ast_util::{is_type_function, range};
use std::convert::Infallible;

use full_moon::{
    ast::{self, Ast},
    visitors::Visitor,
};

pub struct TypeCheckInsideCallLint;

impl Rule for TypeCheckInsideCallLint {
    type Config = ();
    type Error = Infallible;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(TypeCheckInsideCallLint)
    }

    fn pass(&self, ast: &Ast, context: &Context) -> Vec<Diagnostic> {
        let mut visitor = TypeCheckInsideCallVisitor {
            positions: Vec::new(),
            roblox: context.is_roblox(),
        };

        visitor.visit_ast(&ast);

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

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn rule_type(&self) -> RuleType {
        RuleType::Correctness
    }
}

struct TypeCheckInsideCallVisitor {
    positions: Vec<(usize, usize)>,
    roblox: bool,
}

impl Visitor<'_> for TypeCheckInsideCallVisitor {
    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        if_chain::if_chain! {
            // Check that we're using type or typeof
            if let ast::Prefix::Name(name) = call.prefix();
            if is_type_function(&name.to_string(), self.roblox);

            // Check that we're calling it with an argument
            if let ast::Suffix::Call(call) = call.iter_suffixes().next().unwrap();
            if let ast::Call::AnonymousCall(args) = call;
            if let ast::FunctionArgs::Parentheses { arguments, .. } = args;
            if let Some(argument) = arguments.iter().next();

            // Check that the argument is in the form of x == y
            if let ast::Expression::Value { binop: rhs, .. } = argument;
            if let Some(rhs) = rhs;
            if let ast::BinOp::TwoEqual(_) = rhs.bin_op();

            // Check that rhs is a constant string
            if let ast::Expression::Value { binop: rhs, value } = rhs.rhs();
            if rhs.is_none();
            if let ast::Value::String(_) = &**value;

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
