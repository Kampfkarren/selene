use super::*;
use crate::ast_util::range;
use std::convert::Infallible;

use full_moon::{
    ast::{self, Ast},
    tokenizer::{Token, TokenKind},
    visitors::Visitor,
};
use serde::Deserialize;

#[derive(Clone, Copy, Default, Deserialize)]
#[serde(default)]
pub struct EmptyLoopLintConfig {
    comments_count: bool,
}

pub struct EmptyLoopLint {
    config: EmptyLoopLintConfig,
}

impl Lint for EmptyLoopLint {
    type Config = EmptyLoopLintConfig;
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Warning;
    const LINT_TYPE: LintType = LintType::Style;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(EmptyLoopLint { config })
    }

    fn pass(&self, ast: &Ast, _: &Context, _: &AstContext) -> Vec<Diagnostic> {
        let mut visitor = EmptyLoopVisitor {
            comment_positions: Vec::new(),
            positions: Vec::new(),
        };

        visitor.visit_ast(ast);

        let comment_positions = visitor.comment_positions.clone();

        visitor
            .positions
            .into_iter()
            .filter(|position| {
                // OPTIMIZE: This is O(n^2), can we optimize this?
                if self.config.comments_count {
                    !comment_positions.iter().any(|comment_position| {
                        position.0 <= *comment_position && position.1 >= *comment_position
                    })
                } else {
                    true
                }
            })
            .map(|position| {
                Diagnostic::new(
                    "empty_loop",
                    "empty loop block".to_owned(),
                    Label::new(position),
                )
            })
            .collect()
    }
}

struct EmptyLoopVisitor {
    comment_positions: Vec<u32>,
    positions: Vec<(u32, u32)>,
}

fn block_is_empty(block: &ast::Block) -> bool {
    block.last_stmt().is_none() && block.stmts().next().is_none()
}

impl Visitor for EmptyLoopVisitor {
    fn visit_generic_for(&mut self, node: &ast::GenericFor) {
        if block_is_empty(node.block()) {
            self.positions.push(range(node));
        }
    }

    fn visit_numeric_for(&mut self, node: &ast::NumericFor) {
        if block_is_empty(node.block()) {
            self.positions.push(range(node));
        }
    }

    fn visit_while(&mut self, node: &ast::While) {
        if block_is_empty(node.block()) {
            self.positions.push(range(node));
        }
    }

    fn visit_repeat(&mut self, node: &ast::Repeat) {
        if block_is_empty(node.block()) {
            self.positions.push(range(node));
        }
    }

    fn visit_token(&mut self, token: &Token) {
        match token.token_kind() {
            TokenKind::MultiLineComment | TokenKind::SingleLineComment => {
                self.comment_positions
                    .push(Token::end_position(token).bytes() as u32);
            }

            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_empty_loop() {
        test_lint(
            EmptyLoopLint::new(EmptyLoopLintConfig::default()).unwrap(),
            "empty_loop",
            "empty_loop",
        );
    }

    #[test]
    fn test_empty_loop_comments() {
        test_lint(
            EmptyLoopLint::new(EmptyLoopLintConfig {
                comments_count: true,
            })
            .unwrap(),
            "empty_loop",
            "empty_loop_comments",
        );
    }
}
