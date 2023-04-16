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

    const SEVERITY: Severity = Severity::Error;
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
                    "roblox_mismatched_udim2_new_arg_count",
                    format!(
                        "UDim2.new takes 4 numbers, but {} were provided.",
                        mismatch.num_provided
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
    num_provided: usize,
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
                let num_provided = arguments.len();
                if num_provided != 4 {
                    self.args.push(MismatchedArgCount {
                        num_provided,
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
    fn test_roblox_mismatched_udim2_new_arg_count() {
        test_lint(
            UDim2ArgCountLint::new(()).unwrap(),
            "roblox_mismatched_udim2_new_arg_count",
            "roblox_mismatched_udim2_new_arg_count",
        );
    }
}
