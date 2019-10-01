use super::*;
use std::convert::Infallible;

use full_moon::{
    ast::{self, Ast},
    node::Node,
    tokenizer::{Token, TokenReference, TokenKind},
    visitors::Visitor,
};
use serde::Deserialize;

#[derive(Clone, Copy, Deserialize)]
#[serde(default)]
pub struct EmptyIfLintConfig {
    comments_count: bool,
}

impl Default for EmptyIfLintConfig {
    fn default() -> Self {
        Self {
            comments_count: false,
        }
    }
}

pub struct EmptyIfLint {
    config: EmptyIfLintConfig,
}

impl Rule for EmptyIfLint {
    type Config = EmptyIfLintConfig;
    type Error = Infallible;

    fn new(config: EmptyIfLintConfig) -> Result<Self, Self::Error> {
        Ok(EmptyIfLint {
            config,
        })
    }

    fn pass(&self, ast: &Ast) -> Vec<Diagnostic> {
        let mut visitor = EmptyIfVisitor {
            comment_positions: Vec::new(),
            positions: Vec::new(),
        };

        visitor.visit_ast(&ast);

        let comment_positions = visitor.comment_positions.clone();

        visitor
            .positions
            .into_iter()
            .filter(|(position, _)| {
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
                    "empty_if",
                    match position.1 {
                        EmptyIfKind::If => "empty if block",
                        EmptyIfKind::ElseIf => "empty elseif block",
                        EmptyIfKind::Else => "empty else block",
                    }
                    .to_owned(),
                    Label::new(position.0),
                )
            })
            .collect()
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn rule_type(&self) -> RuleType {
        RuleType::Style
    }
}

struct EmptyIfVisitor {
    comment_positions: Vec<u32>,
    positions: Vec<((u32, u32), EmptyIfKind)>,
}

impl Visitor<'_> for EmptyIfVisitor {
    fn visit_if(&mut self, if_block: &ast::If<'_>) {
        if if_block.block().iter_stmts().next().is_none() {
            self.positions.push((
                if_block
                    .range()
                    .map(|(start, end)| (start.bytes() as u32, end.bytes() as u32))
                    .unwrap(),
                EmptyIfKind::If,
            ));
        }

        if let Some(else_ifs) = if_block.else_if() {
            let mut else_ifs = else_ifs.iter().peekable();

            while let Some(else_if) = else_ifs.next() {
                if else_if.block().iter_stmts().next().is_none() {
                    let next_token_position = match else_ifs.peek() {
                        Some(next_else_if) => next_else_if.start_position().unwrap().bytes() as u32,
                        None => {
                            if let Some(else_block) = if_block.else_token() {
                                else_block.start_position().unwrap().bytes() as u32
                            } else {
                                if_block.end_token().start_position().unwrap().bytes() as u32
                            }
                        }
                    };

                    self.positions.push((
                        (
                            else_if.start_position().unwrap().bytes() as u32,
                            next_token_position,
                        ),
                        EmptyIfKind::ElseIf,
                    ));
                }
            }
        }

        if let Some(else_block) = if_block.else_block() {
            if else_block.iter_stmts().next().is_none() {
                self.positions.push((
                    (
                        if_block.else_token().start_position().unwrap().bytes() as u32,
                        if_block.end_token().end_position().unwrap().bytes() as u32,
                    ),
                    EmptyIfKind::Else,
                ));
            }
        }
    }

    fn visit_token(&mut self, token: &TokenReference<'_>) {
        match token.token_kind() {
            TokenKind::MultiLineComment | TokenKind::SingleLineComment => {
                self.comment_positions.push(Token::end_position(token).bytes() as u32);
            },

            _ => {}
        }
    }
}

enum EmptyIfKind {
    If,
    ElseIf,
    Else,
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_empty_if() {
        test_lint(EmptyIfLint::new(
            EmptyIfLintConfig::default()
        ).unwrap(), "empty_if", "empty_if");
    }

    #[test]
    fn test_empty_if_comments() {
        test_lint(EmptyIfLint::new(
            EmptyIfLintConfig {
                comments_count: true,
            }
        ).unwrap(), "empty_if", "empty_if_comments");
    }
}
