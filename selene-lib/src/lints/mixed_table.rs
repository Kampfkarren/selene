use super::*;
use crate::ast_util::range;
use std::convert::Infallible;

use full_moon::{
    ast::{self, Ast},
    visitors::Visitor,
};

pub struct MixedTableLint;

impl Lint for MixedTableLint {
    type Config = ();
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Warning;
    const LINT_TYPE: LintType = LintType::Correctness;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(MixedTableLint)
    }

    fn pass(&self, ast: &Ast, _: &Context, _: &AstContext) -> Vec<Diagnostic> {
        let mut visitor = MixedTableVisitor::default();

        visitor.visit_ast(ast);

        let mut diagnostics = Vec::new();

        for mixed_table in visitor.mixed_tables {
            diagnostics.push(Diagnostic::new_complete(
                "mixed_table",
                "mixed tables should be avoided, as they can cause confusing and hard to debug issues such as during iteration or encoding".to_owned(),
                Label::new(mixed_table.range),
                vec!["help: change this table to either an array or dictionary".to_owned()],
                Vec::new(),
            ));
        }

        diagnostics
    }
}

#[derive(Default)]
struct MixedTableVisitor {
    mixed_tables: Vec<MixedTable>,
}

struct MixedTable {
    range: (usize, usize),
}

impl Visitor for MixedTableVisitor {
    fn visit_table_constructor(&mut self, node: &ast::TableConstructor) {
        let mut last_key_field_starting_range = 0;
        let mut last_no_key_field_starting_range = 0;

        for field in node.fields() {
            if let ast::Field::NoKey(_) = field {
                if last_key_field_starting_range > 0 {
                    self.mixed_tables.push(MixedTable {
                        range: (last_key_field_starting_range, range(field).1),
                    });
                    return;
                }
                last_no_key_field_starting_range = range(field).0;
            } else {
                if last_no_key_field_starting_range > 0 {
                    self.mixed_tables.push(MixedTable {
                        range: (last_no_key_field_starting_range, range(field).1),
                    });
                    return;
                }
                last_key_field_starting_range = range(field).0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_mixed_table() {
        test_lint(
            MixedTableLint::new(()).unwrap(),
            "mixed_table",
            "mixed_table",
        );
    }
}
