use super::*;
use crate::ast_util::range;
use std::convert::Infallible;

use full_moon::{
    ast::{self, Ast},
    visitors::Visitor,
};

pub struct UDim2ArgCountLint;

impl Lint for UDim2ArgCountLint {
    type Config = ();
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Warning;
    const LINT_TYPE: LintType = LintType::Correctness;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(UDim2ArgCountLint)
    }

    fn pass(&self, ast: &Ast, context: &Context, _: &AstContext) -> Vec<Diagnostic> {
        if !context.is_roblox() {
            return Vec::new();
        }

        let mut visitor = UDim2CountVisitor::default();

        visitor.visit_ast(ast);

        visitor
            .args
            .iter()
            .map(|mismatch| {
                Diagnostic::new_complete(
                    "roblox_suspicious_udim2_new",
                    format!(
                        "UDim2.new takes 4 numbers, but {} were provided.",
                        mismatch.args_provided
                    )
                    .to_owned(),
                    Label::new(mismatch.call_range),
                    vec![
                        "help: did you mean to use UDim2.fromScale or UDim2.fromOffset instead?"
                            .to_owned(),
                    ],
                    Vec::new(),
                )
            })
            .collect()
    }
}

#[derive(Default)]
struct UDim2CountVisitor {
    args: Vec<MismatchedArgCount>,
}

struct MismatchedArgCount {
    args_provided: usize,
    call_range: (usize, usize),
}

impl Visitor for UDim2CountVisitor {
    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        if_chain::if_chain! {
            if let ast::Prefix::Name(token) = call.prefix();
            if token.token().to_string() == "UDim2";
            let mut suffixes = call.suffixes().collect::<Vec<_>>();

            if suffixes.len() == 2; // .new and ()
            let call_suffix = suffixes.pop().unwrap();
            let index_suffix = suffixes.pop().unwrap();

            if let ast::Suffix::Index(ast::Index::Dot { name, .. }) = index_suffix;
            if name.token().to_string() == "new";

            if let ast::Suffix::Call(ast::Call::AnonymousCall(
                ast::FunctionArgs::Parentheses { arguments, .. }
            )) = call_suffix;

            then {
                let args_provided = arguments.len();
                let numbers_passed = arguments.iter().filter(|expression| {
                    match expression {
                        ast::Expression::Value { value, .. } => matches!(&**value, ast::Value::Number(_)),
                        _ => false,
                    }
                }).count();

                if args_provided == 2 && numbers_passed != 2 {
                    return;
                };

                if args_provided < 4 {
                    self.args.push(MismatchedArgCount {
                        args_provided,
                        call_range: range(call),
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_roblox_suspicious_udim2_new() {
        test_lint(
            UDim2ArgCountLint::new(()).unwrap(),
            "roblox_suspicious_udim2_new",
            "roblox_suspicious_udim2_new",
        );
    }
}
