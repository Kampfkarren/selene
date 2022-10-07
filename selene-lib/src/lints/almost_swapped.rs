use super::*;
use crate::ast_util::{purge_trivia, range, HasSideEffects};
use std::convert::Infallible;

use full_moon::{
    ast::{self, Ast},
    visitors::Visitor,
};

pub struct AlmostSwappedLint;

impl Lint for AlmostSwappedLint {
    type Config = ();
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Error;
    const LINT_TYPE: LintType = LintType::Correctness;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(AlmostSwappedLint)
    }

    fn pass(&self, ast: &Ast, _: &Context, _: &AstContext) -> Vec<Diagnostic> {
        let mut visitor = AlmostSwappedVisitor {
            almost_swaps: Vec::new(),
        };

        visitor.visit_ast(ast);

        visitor
            .almost_swaps
            .iter()
            .map(|almost_swap| {
                Diagnostic::new_complete(
                    "almost_swapped",
                    format!(
                        "this looks like you are trying to swap `{}` and `{}`",
                        (almost_swap.names.0),
                        (almost_swap.names.1),
                    ),
                    Label::new(almost_swap.range),
                    vec![format!(
                        "try: `{name1}, {name2} = {name2}, {name1}`",
                        name1 = almost_swap.names.0,
                        name2 = almost_swap.names.1,
                    )],
                    Vec::new(),
                )
            })
            .collect()
    }
}

struct AlmostSwappedVisitor {
    almost_swaps: Vec<AlmostSwap>,
}

struct AlmostSwap {
    names: (String, String),
    range: (usize, usize),
}

impl Visitor for AlmostSwappedVisitor {
    fn visit_block(&mut self, block: &ast::Block) {
        let mut last_swap: Option<AlmostSwap> = None;

        for stmt in block.stmts() {
            if let ast::Stmt::Assignment(assignment) = stmt {
                let expressions = assignment.expressions();
                let variables = assignment.variables();

                if variables.len() == 1 && expressions.len() == 1 {
                    let expr = expressions.into_iter().next().unwrap();
                    let var = variables.into_iter().next().unwrap();

                    if !var.has_side_effects() {
                        let expr_end = range(expr).1;

                        let expr_text = purge_trivia(expr).to_string().trim().to_owned();
                        let var_text = purge_trivia(var).to_string().trim().to_owned();

                        if let Some(last_swap) = last_swap.take() {
                            if last_swap.names.0 == expr_text && last_swap.names.1 == var_text {
                                self.almost_swaps.push(AlmostSwap {
                                    names: last_swap.names.to_owned(),
                                    range: (last_swap.range.0, expr_end),
                                });
                            }
                        } else {
                            last_swap = Some(AlmostSwap {
                                names: (var_text, expr_text),
                                range: range(stmt),
                            });
                        }

                        continue;
                    }
                }
            }

            last_swap = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_almost_swapped() {
        test_lint(
            AlmostSwappedLint::new(()).unwrap(),
            "almost_swapped",
            "almost_swapped",
        );
    }

    #[test]
    fn test_almost_swapped_panic() {
        test_lint(
            AlmostSwappedLint::new(()).unwrap(),
            "almost_swapped",
            "panic",
        );
    }
}
