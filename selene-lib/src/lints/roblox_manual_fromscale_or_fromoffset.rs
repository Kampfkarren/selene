use super::*;
use crate::ast_util::range;
use std::convert::Infallible;

use full_moon::{
    ast::{self, Ast},
    visitors::Visitor,
};

pub struct ManualFromScaleOrFromOffsetLint;

fn create_diagnostic(args: &UDim2ComplexArgs) -> Diagnostic {
    let code = "roblox_manual_fromscale_or_fromoffset";
    let primary_label = Label::new(args.call_range);

    match args.complexity_type {
        UDim2ConstructorType::OffsetOnly => Diagnostic::new_complete(
            code,
            "this UDim2.new call only sets offset, and can be simplified using UDim2.fromOffset"
                .to_owned(),
            primary_label,
            vec![format!(
                "try: UDim2.fromOffset({}, {})",
                args.arg_0, args.arg_1
            )],
            Vec::new(),
        ),
        UDim2ConstructorType::ScaleOnly => Diagnostic::new_complete(
            code,
            "this UDim2.new call only sets scale, and can be simplified using UDim2.fromScale"
                .to_owned(),
            primary_label,
            vec![format!(
                "try: UDim2.fromScale({}, {})",
                args.arg_0, args.arg_1
            )],
            Vec::new(),
        ),
    }
}

impl Lint for ManualFromScaleOrFromOffsetLint {
    type Config = ();
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Warning;
    const LINT_TYPE: LintType = LintType::Style;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(ManualFromScaleOrFromOffsetLint)
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

#[derive(PartialEq)]
enum UDim2ConstructorType {
    ScaleOnly,
    OffsetOnly,
}

struct UDim2ComplexArgs {
    complexity_type: UDim2ConstructorType,
    call_range: (usize, usize),
    arg_0: String,
    arg_1: String,
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
                if arguments.len() != 4 {
                    return;
                }

                let mut iter = arguments.iter();
                let x_scale = iter.next().unwrap().to_string();
                let x_offset = iter.next().unwrap().to_string();
                let y_scale = iter.next().unwrap().to_string();
                let y_offset = iter.next().unwrap().to_string();

                let only_offset = x_scale.parse::<f32>() == Ok(0.0) && y_scale.parse::<f32>() == Ok(0.0);
                let only_scale = x_offset.parse::<f32>() == Ok(0.0) && y_offset.parse::<f32>() == Ok(0.0);

                if only_offset && only_scale
                {
                    // Skip linting if all are zero
                }
                else if only_offset
                {
                    self.args.push(UDim2ComplexArgs {
                        call_range: range(call),
                        arg_0: x_offset,
                        arg_1: y_offset,
                        complexity_type: UDim2ConstructorType::OffsetOnly,
                    });
                }
                else if only_scale
                {
                    self.args.push(UDim2ComplexArgs {
                        call_range: range(call),
                        arg_0: x_scale,
                        arg_1: y_scale,
                        complexity_type: UDim2ConstructorType::ScaleOnly,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_manual_fromscale_or_fromoffset() {
        test_lint(
            ManualFromScaleOrFromOffsetLint::new(()).unwrap(),
            "roblox_manual_fromscale_or_fromoffset",
            "roblox_manual_fromscale_or_fromoffset",
        );
    }
}
