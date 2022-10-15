use super::*;
use crate::ast_util::{range, HasSideEffects};
use std::convert::Infallible;

use full_moon::{
    ast::{self, Ast},
    node::Node,
    visitors::Visitor,
};

pub struct IfsSameCondLint;

impl Lint for IfsSameCondLint {
    type Config = ();
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Error;
    const LINT_TYPE: LintType = LintType::Correctness;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(IfsSameCondLint)
    }

    fn pass(&self, ast: &Ast, _: &Context, _: &AstContext) -> Vec<Diagnostic> {
        let mut visitor = IfsSameCondVisitor {
            positions: Vec::new(),
        };

        visitor.visit_ast(ast);

        visitor
            .positions
            .drain(..)
            .map(|position| {
                Diagnostic::new_complete(
                    "ifs_same_cond",
                    "this `elseif` has the same condition as a previous if".to_owned(),
                    Label::new(position.0),
                    Vec::new(),
                    vec![Label::new_with_message(
                        position.1,
                        "note: same as this".to_owned(),
                    )],
                )
            })
            .collect()
    }
}

struct IfsSameCondVisitor {
    positions: Vec<((u32, u32), (u32, u32))>,
}

impl Visitor for IfsSameCondVisitor {
    fn visit_if(&mut self, if_block: &ast::If) {
        if let Some(else_ifs) = if_block.else_if() {
            let mut conditions = Vec::with_capacity(else_ifs.len() + 1);
            if !if_block.condition().has_side_effects() {
                conditions.push(if_block.condition());
            }

            'else_ifs: for else_if in else_ifs {
                let condition = else_if.condition();
                if !condition.has_side_effects() {
                    for other in &conditions {
                        if other.similar(&condition) {
                            self.positions.push((range(condition), range(other)));
                            continue 'else_ifs;
                        }
                    }

                    conditions.push(condition);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_ifs_same_cond() {
        test_lint(
            IfsSameCondLint::new(()).unwrap(),
            "ifs_same_cond",
            "ifs_same_cond",
        );
    }
}
