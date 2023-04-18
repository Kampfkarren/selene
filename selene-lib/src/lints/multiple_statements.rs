use super::*;
use std::{collections::HashSet, convert::Infallible};

use full_moon::{
    ast::{self, Ast},
    node::Node,
    visitors::Visitor,
};
use serde::Deserialize;

#[derive(Clone, Copy, Default, Deserialize)]
pub struct MultipleStatementsConfig {
    one_line_if: OneLineIf,
}

pub struct MultipleStatementsLint {
    config: MultipleStatementsConfig,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OneLineIf {
    Allow,
    Deny,
    #[default]
    BreakReturnOnly,
}

impl Lint for MultipleStatementsLint {
    type Config = MultipleStatementsConfig;
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Warning;
    const LINT_TYPE: LintType = LintType::Style;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(MultipleStatementsLint { config })
    }

    fn pass(&self, ast: &Ast, _: &Context, _: &AstContext) -> Vec<Diagnostic> {
        let mut visitor = MultipleStatementsVisitor {
            config: self.config,
            ..MultipleStatementsVisitor::default()
        };

        visitor.visit_ast(ast);

        visitor
            .positions
            .iter()
            .map(|position| {
                Diagnostic::new(
                    "multiple_statements",
                    "only one statement per line is allowed".to_owned(),
                    Label::new(*position),
                )
            })
            .collect()
    }
}

#[derive(Default)]
struct MultipleStatementsVisitor {
    config: MultipleStatementsConfig,
    if_lines: HashSet<usize>,
    lines_with_stmt: HashSet<usize>,
    positions: Vec<(usize, usize)>,
}

impl MultipleStatementsVisitor {
    fn prepare_if(&mut self, if_block: &ast::If) {
        let line = if_block.then_token().end_position().unwrap().line();

        if self.config.one_line_if != OneLineIf::Deny {
            if self.config.one_line_if == OneLineIf::BreakReturnOnly
                && (if_block.block().stmts().next().is_some()
                    || if_block.block().last_stmt().is_none())
            {
                return;
            }

            self.if_lines.insert(line);
        }
    }

    fn lint_stmt<N: Node>(&mut self, stmt: N) {
        let line = stmt.end_position().unwrap().line();

        if self.lines_with_stmt.contains(&line) {
            let range = stmt.range().unwrap();
            self.positions.push((range.0.bytes(), range.1.bytes()));
        } else if self.if_lines.contains(&line) {
            self.if_lines.remove(&line);
        } else {
            self.lines_with_stmt.insert(line);
        }
    }
}

impl Visitor for MultipleStatementsVisitor {
    fn visit_last_stmt(&mut self, stmt: &ast::LastStmt) {
        self.lint_stmt(stmt);
    }

    fn visit_stmt(&mut self, stmt: &ast::Stmt) {
        if let ast::Stmt::If(if_block) = stmt {
            self.prepare_if(if_block);
        }

        self.lint_stmt(stmt);
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_multiple_statements() {
        test_lint(
            MultipleStatementsLint::new(MultipleStatementsConfig::default()).unwrap(),
            "multiple_statements",
            "multiple_statements",
        );
    }

    #[test]
    fn test_one_line_if_deny() {
        test_lint(
            MultipleStatementsLint::new(MultipleStatementsConfig {
                one_line_if: OneLineIf::Deny,
            })
            .unwrap(),
            "multiple_statements",
            "one_line_if_deny",
        );
    }

    #[test]
    fn test_one_line_if_allow() {
        test_lint(
            MultipleStatementsLint::new(MultipleStatementsConfig {
                one_line_if: OneLineIf::Allow,
            })
            .unwrap(),
            "multiple_statements",
            "one_line_if_allow",
        );
    }

    #[test]
    fn test_one_line_if_break_return_only() {
        test_lint(
            MultipleStatementsLint::new(MultipleStatementsConfig {
                one_line_if: OneLineIf::BreakReturnOnly,
            })
            .unwrap(),
            "multiple_statements",
            "one_line_if_break_return_only",
        );
    }
}
