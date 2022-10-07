use super::*;
use crate::ast_util::range;
use std::convert::Infallible;

use full_moon::{
    ast::{self, Ast},
    node::Node,
    visitors::Visitor,
};

pub struct IfSameThenElseLint;

impl Lint for IfSameThenElseLint {
    type Config = ();
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Error;
    const LINT_TYPE: LintType = LintType::Correctness;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(IfSameThenElseLint)
    }

    fn pass(&self, ast: &Ast, _: &Context, _: &AstContext) -> Vec<Diagnostic> {
        let mut visitor = IfSameThenElseVisitor {
            positions: Vec::new(),
        };

        visitor.visit_ast(ast);

        visitor
            .positions
            .drain(..)
            .map(|position| {
                Diagnostic::new_complete(
                    "if_same_then_else",
                    "this has the same block as a previous if".to_owned(),
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

struct IfSameThenElseVisitor {
    positions: Vec<((u32, u32), (u32, u32))>,
}

impl Visitor for IfSameThenElseVisitor {
    fn visit_if(&mut self, if_block: &ast::If) {
        let else_ifs = if_block
            .else_if()
            .map(|else_ifs| else_ifs.iter().collect())
            .unwrap_or_else(Vec::new);

        let mut blocks = Vec::with_capacity(2 + else_ifs.len());
        blocks.push(if_block.block());

        'blocks: for block in else_ifs
            .iter()
            .map(|else_if| else_if.block())
            .chain(if_block.else_block())
        {
            if block.stmts().next().is_none() {
                continue;
            }

            for other in &blocks {
                if other.similar(&block) {
                    self.positions.push((range(block), range(other)));
                    continue 'blocks;
                }
            }

            blocks.push(block);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_if_same_then_else() {
        test_lint(
            IfSameThenElseLint::new(()).unwrap(),
            "if_same_then_else",
            "if_same_then_else",
        );
    }
}
