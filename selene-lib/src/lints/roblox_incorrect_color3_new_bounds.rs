use super::*;
use crate::ast_util::range;
use std::convert::Infallible;

use full_moon::{
    ast::{self, Ast},
    visitors::Visitor,
};

pub struct Color3BoundsLint;

impl Lint for Color3BoundsLint {
    type Config = ();
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Error;
    const LINT_TYPE: LintType = LintType::Correctness;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(Color3BoundsLint)
    }

    fn pass(&self, ast: &Ast, context: &Context, _: &AstContext) -> Vec<Diagnostic> {
        if !context.is_roblox() {
            return Vec::new();
        }

        let mut visitor = Color3BoundsVisitor::default();

        visitor.visit_ast(ast);

        visitor
            .positions
            .iter()
            .map(|position| {
                Diagnostic::new_complete(
                    "roblox_incorrect_color3_new_bounds",
                    "Color3.new only takes numbers from 0 to 1".to_owned(),
                    Label::new(*position),
                    vec!["help: did you mean to use Color3.fromRGB instead?".to_owned()],
                    Vec::new(),
                )
            })
            .collect()
    }
}

#[derive(Default)]
struct Color3BoundsVisitor {
    positions: Vec<(usize, usize)>,
}

impl Visitor for Color3BoundsVisitor {
    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        if_chain::if_chain! {
            if let ast::Prefix::Name(token) = call.prefix();
            if token.token().to_string() == "Color3";
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
                for argument in arguments {
                    if let Ok(number) = argument.to_string().parse::<f32>() {
                        if !(0.0..=1.0).contains(&number) {
                            self.positions.push(range(argument));
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_roblox_incorrect_color3_new_bounds() {
        test_lint(
            Color3BoundsLint::new(()).unwrap(),
            "roblox_incorrect_color3_new_bounds",
            "roblox_incorrect_color3_new_bounds",
        );
    }
}
