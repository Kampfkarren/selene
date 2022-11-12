use super::*;
use crate::ast_util::range;
use std::convert::Infallible;

use full_moon::{
    ast::{self, Ast, TableConstructor},
    visitors::Visitor,
};

use serde::Deserialize;

#[derive(Clone, Copy, Deserialize)]
pub struct HighCyclomaticComplexityConfig {
    maximum_complexity: u16,
}

impl Default for HighCyclomaticComplexityConfig {
    fn default() -> Self {
        Self {
            // eslint defaults to 20, but testing on OSS Lua shows that 20 is too aggressive
            maximum_complexity: 40,
        }
    }
}

#[derive(Default)]
pub struct HighCyclomaticComplexityLint {
    config: HighCyclomaticComplexityConfig,
}

impl Lint for HighCyclomaticComplexityLint {
    type Config = HighCyclomaticComplexityConfig;
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Allow;
    const LINT_TYPE: LintType = LintType::Style;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(HighCyclomaticComplexityLint { config })
    }

    fn pass(&self, ast: &Ast, _: &Context, _: &AstContext) -> Vec<Diagnostic> {
        let mut visitor = HighCyclomaticComplexityVisitor {
            positions: Vec::new(),
            config: self.config,
        };

        visitor.visit_ast(ast);

        visitor
            .positions
            .into_iter()
            .map(|(position, complexity)| {
                Diagnostic::new(
                    "high_cyclomatic_complexity",
                    format!(
                        "cyclomatic complexity is too high ({complexity} > {})",
                        self.config.maximum_complexity
                    ),
                    Label::new(position),
                )
            })
            .collect()
    }
}

struct HighCyclomaticComplexityVisitor {
    positions: Vec<((u32, u32), u16)>,
    config: HighCyclomaticComplexityConfig,
}

fn count_table_complexity(table: &TableConstructor, starting_complexity: u16) -> u16 {
    let mut complexity = starting_complexity;

    for field in table.fields() {
        #[cfg_attr(
            feature = "force_exhaustive_checks",
            deny(non_exhaustive_omitted_patterns)
        )]
        match field {
            ast::Field::ExpressionKey { key, value, .. } => {
                complexity = count_expression_complexity(key, complexity);
                complexity = count_expression_complexity(value, complexity);
            }

            ast::Field::NameKey { value, .. } => {
                complexity = count_expression_complexity(value, complexity);
            }

            ast::Field::NoKey(expression) => {
                complexity = count_expression_complexity(expression, complexity);
            }

            _ => {}
        }
    }
    complexity
}

fn count_arguments_complexity(function_args: &ast::FunctionArgs, starting_complexity: u16) -> u16 {
    let mut complexity = starting_complexity;

    #[cfg_attr(
        feature = "force_exhaustive_checks",
        deny(non_exhaustive_omitted_patterns)
    )]
    match function_args {
        ast::FunctionArgs::Parentheses { arguments, .. } => {
            for argument in arguments {
                complexity = count_expression_complexity(argument, complexity)
            }
            complexity
        }
        ast::FunctionArgs::TableConstructor(table) => {
            complexity = count_table_complexity(table, complexity);
            complexity
        }
        ast::FunctionArgs::String(_) => complexity,
        _ => complexity,
    }
}

fn count_suffix_complexity(suffix: &ast::Suffix, starting_complexity: u16) -> u16 {
    let mut complexity = starting_complexity;

    #[cfg_attr(
        feature = "force_exhaustive_checks",
        deny(non_exhaustive_omitted_patterns)
    )]
    match suffix {
        ast::Suffix::Index(ast::Index::Brackets { expression, .. }) => {
            complexity = count_expression_complexity(expression, complexity)
        }
        #[cfg_attr(
            feature = "force_exhaustive_checks",
            deny(non_exhaustive_omitted_patterns)
        )]
        ast::Suffix::Call(call) => match call {
            ast::Call::AnonymousCall(arguments) => {
                complexity = count_arguments_complexity(arguments, complexity)
            }
            ast::Call::MethodCall(method_call) => {
                complexity = count_arguments_complexity(method_call.args(), complexity)
            }
            _ => {}
        },
        _ => {}
    }

    complexity
}

fn count_expression_complexity(expression: &ast::Expression, starting_complexity: u16) -> u16 {
    let mut complexity = starting_complexity;

    #[cfg_attr(
        feature = "force_exhaustive_checks",
        deny(non_exhaustive_omitted_patterns)
    )]
    match expression {
        ast::Expression::BinaryOperator {
            lhs, binop, rhs, ..
        } => {
            #[cfg_attr(
                feature = "force_exhaustive_checks",
                allow(non_exhaustive_omitted_patterns)
            )]
            if matches!(binop, ast::BinOp::And(_) | ast::BinOp::Or(_)) {
                complexity += 1;
            }

            complexity = count_expression_complexity(lhs, complexity);
            complexity = count_expression_complexity(rhs, complexity);

            complexity
        }

        ast::Expression::Parentheses { expression, .. } => {
            count_expression_complexity(expression, complexity)
        }

        ast::Expression::UnaryOperator { expression, .. } => {
            count_expression_complexity(expression, complexity)
        }

        #[cfg_attr(
            feature = "force_exhaustive_checks",
            deny(non_exhaustive_omitted_patterns)
        )]
        ast::Expression::Value { value, .. } => match &**value {
            // visit_value already tracks this
            ast::Value::Function(_) => complexity,

            ast::Value::FunctionCall(call) => {
                if let ast::Prefix::Expression(prefix_expression) = call.prefix() {
                    complexity = count_expression_complexity(prefix_expression, complexity)
                }
                for suffix in call.suffixes() {
                    complexity = count_suffix_complexity(suffix, complexity)
                }

                complexity
            }

            ast::Value::ParenthesesExpression(paren_expression) => {
                count_expression_complexity(paren_expression, complexity)
            }

            ast::Value::Number(_) => complexity,
            ast::Value::String(_) => complexity,
            ast::Value::Symbol(_) => complexity,

            ast::Value::TableConstructor(table) => count_table_complexity(table, complexity),

            ast::Value::Var(ast::Var::Expression(var_expression)) => {
                for suffix in var_expression.suffixes() {
                    complexity = count_suffix_complexity(suffix, complexity)
                }
                complexity
            }

            ast::Value::Var(ast::Var::Name(_)) => complexity,

            #[cfg(feature = "roblox")]
            ast::Value::IfExpression(if_expression) => {
                complexity += 1;
                if let Some(else_if_expressions) = if_expression.else_if_expressions() {
                    for else_if_expression in else_if_expressions {
                        complexity += 1;
                        complexity = count_expression_complexity(
                            else_if_expression.expression(),
                            complexity,
                        );
                    }
                }
                complexity
            }

            #[cfg(feature = "roblox")]
            ast::Value::InterpolatedString(interpolated_string) => {
                for expression in interpolated_string.expressions() {
                    complexity = count_expression_complexity(expression, complexity)
                }

                complexity
            }

            _ => complexity,
        },

        _ => complexity,
    }
}

fn count_block_complexity(block: &ast::Block, starting_complexity: u16) -> u16 {
    let mut complexity = starting_complexity;

    // we don't immediately return from the matched blocks so that we can add in any complexity from the last statement
    for statement in block.stmts() {
        #[cfg_attr(
            feature = "force_exhaustive_checks",
            deny(non_exhaustive_omitted_patterns)
        )]
        match statement {
            ast::Stmt::Assignment(assignment) => {
                for var in assignment.variables() {
                    if let ast::Var::Expression(var_expression) = var {
                        for suffix in var_expression.suffixes() {
                            complexity = count_suffix_complexity(suffix, complexity)
                        }
                    }
                }
                for expression in assignment.expressions() {
                    complexity = count_expression_complexity(expression, complexity);
                }
            }

            ast::Stmt::Do(do_) => {
                complexity = count_block_complexity(do_.block(), complexity);
            }

            ast::Stmt::FunctionCall(call) => {
                if let ast::Prefix::Expression(prefix_expression) = call.prefix() {
                    complexity = count_expression_complexity(prefix_expression, complexity)
                }
                for suffix in call.suffixes() {
                    complexity = count_suffix_complexity(suffix, complexity)
                }
            }

            // visit_function_declaration already tracks this
            ast::Stmt::FunctionDeclaration(_) => {}

            ast::Stmt::GenericFor(generic_for) => {
                complexity += 1;
                for expression in generic_for.expressions() {
                    complexity = count_expression_complexity(expression, complexity);
                }
                complexity = count_block_complexity(generic_for.block(), complexity);
            }

            ast::Stmt::If(if_block) => {
                complexity += 1;
                complexity = count_expression_complexity(if_block.condition(), complexity);
                complexity = count_block_complexity(if_block.block(), complexity);

                if let Some(else_if_statements) = if_block.else_if() {
                    for else_if in else_if_statements {
                        complexity += 1;
                        complexity = count_expression_complexity(else_if.condition(), complexity);
                        complexity = count_block_complexity(else_if.block(), complexity);
                    }
                }
            }

            ast::Stmt::LocalAssignment(local_assignment) => {
                for expression in local_assignment.expressions() {
                    complexity = count_expression_complexity(expression, complexity);
                }
            }

            // visit_local_function tracks this
            ast::Stmt::LocalFunction(_) => {}

            ast::Stmt::NumericFor(numeric_for) => {
                complexity += 1;
                complexity = count_expression_complexity(numeric_for.start(), complexity);
                complexity = count_expression_complexity(numeric_for.end(), complexity);

                if let Some(step_expression) = numeric_for.step() {
                    complexity = count_expression_complexity(step_expression, complexity);
                }

                complexity = count_block_complexity(numeric_for.block(), complexity);
            }

            ast::Stmt::Repeat(repeat_block) => {
                complexity = count_expression_complexity(repeat_block.until(), complexity + 1);
                complexity = count_block_complexity(repeat_block.block(), complexity);
            }

            ast::Stmt::While(while_block) => {
                complexity = count_expression_complexity(while_block.condition(), complexity + 1);
                complexity = count_block_complexity(while_block.block(), complexity);
            }

            #[cfg(feature = "roblox")]
            ast::Stmt::CompoundAssignment(compound_expression) => {
                complexity = count_expression_complexity(compound_expression.rhs(), complexity)
            }

            #[cfg(feature = "roblox")]
            ast::Stmt::ExportedTypeDeclaration(_) => {
                // doesn't contribute dynamic branches
            }

            #[cfg(feature = "roblox")]
            ast::Stmt::TypeDeclaration(_) => {
                // doesn't contain branch points
            }

            #[cfg(feature = "lua52")]
            ast::Stmt::Goto(_) => {
                // not a dynamic branch point itself
            }

            #[cfg(feature = "lua52")]
            ast::Stmt::Label(_) => {
                // not a dynamic branch point itself
            }

            _ => {}
        }
    }

    if let Some(ast::LastStmt::Return(return_stmt)) = block.last_stmt() {
        for return_expression in return_stmt.returns() {
            complexity = count_expression_complexity(return_expression, complexity);
        }
    }

    complexity
}

impl Visitor for HighCyclomaticComplexityVisitor {
    fn visit_local_function(&mut self, local_function: &ast::LocalFunction) {
        let complexity = count_block_complexity(local_function.body().block(), 1);
        if complexity > self.config.maximum_complexity {
            self.positions.push((
                (
                    range(local_function.function_token()).0,
                    range(local_function.body().parameters_parentheses()).1,
                ),
                complexity,
            ));
        }
    }

    fn visit_function_declaration(&mut self, function_declaration: &ast::FunctionDeclaration) {
        let complexity = count_block_complexity(function_declaration.body().block(), 1);
        if complexity > self.config.maximum_complexity {
            self.positions.push((
                (
                    range(function_declaration.function_token()).0,
                    range(function_declaration.body().parameters_parentheses()).1,
                ),
                complexity,
            ));
        }
    }

    fn visit_value(&mut self, value: &ast::Value) {
        if let ast::Value::Function((_, function_body)) = value {
            let complexity = count_block_complexity(function_body.block(), 1);
            if complexity > self.config.maximum_complexity {
                self.positions.push((
                    (
                        value.start_position().unwrap().bytes() as u32,
                        range(function_body.parameters_parentheses()).1,
                    ),
                    complexity,
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    #[cfg(feature = "roblox")]
    fn test_high_cyclomatic_complexity() {
        test_lint(
            HighCyclomaticComplexityLint::new(HighCyclomaticComplexityConfig::default()).unwrap(),
            "high_cyclomatic_complexity",
            "high_cyclomatic_complexity",
        );
    }

    #[test]
    #[cfg(feature = "roblox")]
    fn test_complex_var_expressions() {
        test_lint(
            HighCyclomaticComplexityLint::new(HighCyclomaticComplexityConfig::default()).unwrap(),
            "high_cyclomatic_complexity",
            "complex_var_expressions",
        );
    }

    #[test]
    fn test_lua51_basic_complexity() {
        test_lint(
            HighCyclomaticComplexityLint::new(HighCyclomaticComplexityConfig {
                maximum_complexity: 1,
            })
            .unwrap(),
            "high_cyclomatic_complexity",
            "lua51_basic_complexity",
        );
    }
}
