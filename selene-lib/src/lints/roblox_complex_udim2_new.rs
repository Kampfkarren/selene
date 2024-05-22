use super::*;
use crate::ast_util::range;
use std::convert::Infallible;

use full_moon::{
    ast::{self, Ast},
    visitors::Visitor,
};

pub struct ComplexUDim2NewLint;

fn create_diagnostic(mismatch: &UDim2ComplexArgs) -> Diagnostic {
    let code = "roblox_complex_udim2_new";
    let primary_label = Label::new(mismatch.call_range);

    if mismatch.no_scale {
        Diagnostic::new_complete(
            code,
            "this UDim2.new call only sets offset, and can be simplified using UDim2.fromOffset"
                .to_owned(),
            primary_label,
            vec![format!(
                "try: UDim2.fromOffset({}, {})",
                mismatch.arg_0, mismatch.arg_1
            )],
            Vec::new(),
        )
    } else {
        Diagnostic::new_complete(
            code,
            "this UDim2.new call only sets scale, and can be simplified using UDim2.fromScale"
                .to_owned(),
            primary_label,
            vec![format!(
                "try: UDim2.fromScale({}, {})",
                mismatch.arg_0, mismatch.arg_1
            )],
            Vec::new(),
        )
    }
}

impl Lint for ComplexUDim2NewLint {
    type Config = ();
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Warning;
    const LINT_TYPE: LintType = LintType::Style;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(ComplexUDim2NewLint)
    }

    fn pass(&self, ast: &Ast, context: &Context, _: &AstContext) -> Vec<Diagnostic> {
        if !context.is_roblox() {
            return Vec::new();
        }

        let mut visitor = UDim2NewVisitor::default();

        visitor.visit_ast(ast);

        visitor.args.iter().map(create_diagnostic).collect()
    }
}

#[derive(Default)]
struct UDim2NewVisitor {
    args: Vec<UDim2ComplexArgs>,
}

struct UDim2ComplexArgs {
    call_range: (usize, usize),
    arg_0: f32,
    arg_1: f32,
    no_scale: bool,
}

impl Visitor for UDim2NewVisitor {
    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        if_chain::if_chain! {
            if let ast::Prefix::Name(token) = call.prefix();
            if token.token().to_string() == "UDim2";
            let mut suffixes = call.suffixes().collect::<Vec<_>>();

            if suffixes.len() == 2; // .new and ()
            let call_suffix = suffixes.pop().unwrap();
            let index_suffix = suffixes.pop().unwrap();

            if let ast::Suffix::Index(ast::Index::Dot { name, .. }) = index_suffix;
            if name.token().to_string() == "new";

            if let ast::Suffix::Call(ast::Call::AnonymousCall(
                ast::FunctionArgs::Parentheses { arguments, .. }
            )) = call_suffix;

            then {
                let args_provided = arguments.len();
                if args_provided != 4 {
                    return;
                }


                let numbers_passed = arguments.iter().filter(|expression| {
                    matches!(expression, ast::Expression::Number(_))
                }).count();
                if numbers_passed != 4 {
                    return;
                }

                let mut iter = arguments.iter();
                let x_scale = match iter.next().unwrap().to_string().parse::<f32>() {
                    Ok(value) => value,
                    Err(_) => return,
                };
                let x_offset = match iter.next().unwrap().to_string().parse::<f32>() {
                    Ok(value) => value,
                    Err(_) => return,
                };
                let y_scale = match iter.next().unwrap().to_string().parse::<f32>() {
                    Ok(value) => value,
                    Err(_) => return,
                };
                let y_offset = match iter.next().unwrap().to_string().parse::<f32>() {
                    Ok(value) => value,
                    Err(_) => return,
                };

                let no_scale = x_scale == 0.0 && y_scale == 0.0;
                let no_offset = x_offset == 0.0 && y_offset == 0.0;

                let arg_0;
                let arg_1;
                if no_scale && no_offset
                {
                    // Skip any lint
                    return;
                }
                else if no_scale
                {
                    arg_0 = x_offset;
                    arg_1 = y_offset;
                }
                else if no_offset {
                    arg_0 = x_scale;
                    arg_1 = y_scale;
                }
                else
                {
                    return;
                }

                self.args.push(UDim2ComplexArgs {
                    call_range: range(call),
                    arg_0: arg_0,
                    arg_1: arg_1,
                    no_scale: no_scale,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_roblox_complex_udim2_new() {
        test_lint(
            ComplexUDim2NewLint::new(()).unwrap(),
            "roblox_complex_udim2_new",
            "roblox_complex_udim2_new",
        );
    }
}
