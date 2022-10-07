use crate::standard_library::{Field, FieldKind};

use super::*;
use std::convert::Infallible;

pub struct MustUseLint;

impl Lint for MustUseLint {
    type Config = ();
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Warning;
    const LINT_TYPE: LintType = LintType::Correctness;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(MustUseLint)
    }

    fn pass(
        &self,
        _: &Ast,
        Context {
            standard_library, ..
        }: &Context,
        AstContext { scope_manager, .. }: &AstContext,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (_, function_call_stmt) in scope_manager.function_calls.iter() {
            let function_behavior =
                match standard_library.find_global(&function_call_stmt.call_name_path) {
                    Some(Field {
                        field_kind: FieldKind::Function(function_behavior),
                        ..
                    }) => function_behavior,
                    _ => continue,
                };

            if !function_behavior.must_use {
                continue;
            }

            diagnostics.push(Diagnostic::new(
                "must_use",
                format!(
                    "unused return value of `{}` must be used",
                    // TODO: This is wrong for methods
                    function_call_stmt.call_name_path.join(".")
                ),
                Label::new(function_call_stmt.call_prefix_range),
            ));
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_must_use() {
        test_lint(MustUseLint::new(()).unwrap(), "must_use", "must_use");
    }
}
