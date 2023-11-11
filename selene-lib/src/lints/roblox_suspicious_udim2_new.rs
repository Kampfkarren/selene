use super::*;
use crate::ast_util::range;
use std::convert::Infallible;

use full_moon::{
    ast::{self, Ast},
    visitors::Visitor,
};

pub struct SuspiciousUDim2NewLint;

fn create_diagnostic(mismatch: &MismatchedArgCount) -> Diagnostic {
    let code = "roblox_suspicious_udim2_new";
    let message = format!(
        "UDim2.new takes 4 numbers, but {} {} provided.",
        mismatch.args_provided,
        if mismatch.args_provided == 1 {
            "was"
        } else {
            "were"
        }
    );
    let primary_label = Label::new(mismatch.call_range);

    if mismatch.args_provided <= 2 && mismatch.args_are_numbers {
        Diagnostic::new_complete(
            code,
            message,
            primary_label,
            vec![if mismatch.args_are_between_0_and_1 {
                "did you mean to use UDim2.fromScale instead?"
            } else {
                "did you mean to use UDim2.fromOffset instead?"
            }
            .to_owned()],
            Vec::new(),
        )
    } else {
        Diagnostic::new(code, message, primary_label)
    }
}

impl Lint for SuspiciousUDim2NewLint {
    type Config = ();
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Warning;
    const LINT_TYPE: LintType = LintType::Correctness;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(SuspiciousUDim2NewLint)
    }

    fn pass(&self, ast: &Ast, context: &Context, _: &AstContext) -> Vec<Diagnostic> {
        if !context.is_roblox() {
            return Vec::new();
        }

        let mut visitor = UDim2CountVisitor::default();

        visitor.visit_ast(ast);

        visitor.args.iter().map(create_diagnostic).collect()
    }
}

#[derive(Default)]
struct UDim2CountVisitor {
    args: Vec<MismatchedArgCount>,
}

struct MismatchedArgCount {
    args_provided: usize,
    call_range: (usize, usize),
    args_are_between_0_and_1: bool,
    args_are_numbers: bool,
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

                if args_provided >= 4 {
                    return;
                }

                let numbers_passed = arguments.iter().filter(|expression| {
                    match expression {
                        ast::Expression::Value { value, .. } => matches!(&**value, ast::Value::Number(_)),
                        _ => false,
                    }
                }).count();

                // Prevents false positives for UDim2.new(UDim.new(), UDim.new())
                if args_provided == 2 && numbers_passed == 0 {
                    return;
                };

                self.args.push(MismatchedArgCount {
                    call_range: range(call),
                    args_provided,
                    args_are_between_0_and_1: arguments.iter().all(|argument| {
                        match argument.to_string().parse::<f32>() {
                            Ok(number) => (0.0..=1.0).contains(&number),
                            Err(_) => false,
                        }
                    }),
                    args_are_numbers: numbers_passed == args_provided,
                });
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
            SuspiciousUDim2NewLint::new(()).unwrap(),
            "roblox_suspicious_udim2_new",
            "roblox_suspicious_udim2_new",
        );
    }
}
