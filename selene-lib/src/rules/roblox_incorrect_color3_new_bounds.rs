use super::*;
use crate::ast_util::range;
use std::convert::Infallible;

use full_moon::{
    ast::{self, Ast},
    visitors::Visitor,
};

pub struct Color3BoundsLint;

impl Rule for Color3BoundsLint {
    type Config = ();
    type Error = Infallible;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(Color3BoundsLint)
    }

    fn pass(&self, ast: &Ast, context: &Context) -> Vec<Diagnostic> {
        if !context.is_roblox() {
            return Vec::new();
        }

        let mut visitor = Color3BoundsVisitor::default();

        visitor.visit_ast(&ast);

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

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn rule_type(&self) -> RuleType {
        RuleType::Correctness
    }
}

#[derive(Default)]
struct Color3BoundsVisitor {
    positions: Vec<(usize, usize)>,
}

impl Visitor<'_> for Color3BoundsVisitor {
    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        if_chain::if_chain! {
            if let ast::Prefix::Name(token) = call.prefix();
            if token.to_string() == "Color3";
            let mut suffixes = call.iter_suffixes().collect::<Vec<_>>();

            if suffixes.len() == 2; // .new and ()
            let call_suffix = suffixes.pop().unwrap();
            let index_suffix = suffixes.pop().unwrap();

            if let ast::Suffix::Index(index) = index_suffix;
            if let ast::Index::Dot { name, .. } = index;
            if name.to_string() == "new";

            if let ast::Suffix::Call(call) = call_suffix;
            if let ast::Call::AnonymousCall(args) = call;
            if let ast::FunctionArgs::Parentheses { arguments, .. } = args;

            then {
                for argument in arguments {
                    // Check if the argument is a constant number
                    if_chain::if_chain! {
                        if let ast::Expression::Value { value, binop, .. } = argument;
                        if binop.is_none();
                        if let ast::Value::Number(value) = &**value;
                        if let Ok(number) = value.to_string().parse::<f32>();
                        if number > 1.0;
                        then {
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
