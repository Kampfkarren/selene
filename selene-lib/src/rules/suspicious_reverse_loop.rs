use super::*;
use std::{convert::Infallible, str};

use full_moon::{
    ast::{self, Ast},
    node::Node,
    visitors::Visitor,
};

pub struct SuspiciousReverseLoopLint;

impl Rule for SuspiciousReverseLoopLint {
    type Config = ();
    type Error = Infallible;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(SuspiciousReverseLoopLint)
    }

    fn pass(&self, ast: &Ast, _: &Context) -> Vec<Diagnostic> {
        let mut visitor = SuspiciousReverseLoopVisitor {
            positions: Vec::new(),
        };

        visitor.visit_ast(&ast);

        visitor
            .positions
            .iter()
            .map(|position| {
                Diagnostic::new_complete(
                    "suspicious_reverse_loop",
                    "this loop will only ever run once at most".to_owned(),
                    Label::new(*position),
                    vec!["help: try adding `, -1` after `1`".to_owned()],
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

struct SuspiciousReverseLoopVisitor {
    positions: Vec<(usize, usize)>,
}

impl Visitor for SuspiciousReverseLoopVisitor {
    fn visit_numeric_for(&mut self, node: &ast::NumericFor) {
        if_chain::if_chain! {
            if node.step().is_none();
            if let ast::Expression::UnaryOperator { unop, .. } = node.start();
            if let ast::UnOp::Hash(_) = unop;
            if let ast::Expression::Value { value, .. } = node.end();
            if let ast::Value::Number(number) = &**value;
            if str::parse::<f32>(&number.token().to_string()).ok() <= Some(1.0);
            then {
                self.positions.push((
                    node.start().start_position().unwrap().bytes(),
                    node.end().end_position().unwrap().bytes(),
                ));
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_suspicious_reverse_loop() {
        test_lint(
            SuspiciousReverseLoopLint::new(()).unwrap(),
            "suspicious_reverse_loop",
            "suspicious_reverse_loop",
        );
    }
}
