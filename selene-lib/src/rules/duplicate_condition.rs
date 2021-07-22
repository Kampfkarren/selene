use crate::ast_util::HasSideEffects;

use super::*;
use std::convert::Infallible;

use full_moon::{
    ast::{self, Ast},
    node::Node,
    tokenizer,
    visitors::Visitor,
};

use quine_mc_cluskey as qmc;

// While the QMC algorithm goes up to 32, we limit to 8 for the same reason clippy does--it's really slow
const MAX_LITERALS: usize = 8;

pub struct DuplicateConditionLint;

impl Rule for DuplicateConditionLint {
    type Config = ();
    type Error = Infallible;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(DuplicateConditionLint)
    }

    fn pass(&self, ast: &Ast, _: &Context) -> Vec<Diagnostic> {
        let mut visitor = DuplicateConditionVisitor {
            diagnostics: Vec::new(),
        };

        visitor.visit_ast(ast);

        visitor.diagnostics
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn rule_type(&self) -> RuleType {
        RuleType::Correctness
    }
}

struct DuplicateConditionVisitor {
    diagnostics: Vec<Diagnostic>,
}

#[derive(Default)]
struct QmcBoolTransformer<'a> {
    layers: usize,
    literals: Vec<&'a ast::Expression>,
}

impl<'a> QmcBoolTransformer<'a> {
    fn grab_literal(&mut self, literal: &'a ast::Expression) -> Option<qmc::Bool> {
        if let Some(index) = self
            .literals
            .iter()
            .position(|predicate| predicate.similar(&literal))
        {
            return Some(qmc::Bool::Term(index as u8));
        }

        if self.literals.len() >= MAX_LITERALS {
            return None;
        }

        let old_len = self.literals.len();
        self.literals.push(literal);
        Some(qmc::Bool::Term(old_len as u8))
    }

    fn expression_to_qmc_bool(&mut self, expression: &'a ast::Expression) -> Option<qmc::Bool> {
        if expression.has_side_effects() {
            return None;
        }

        match expression {
            ast::Expression::BinaryOperator {
                lhs,
                binop: ast::BinOp::And(_),
                rhs,
            } => {
                self.layers += 1;

                Some(qmc::Bool::And(vec![
                    self.expression_to_qmc_bool(lhs)?,
                    self.expression_to_qmc_bool(rhs)?,
                ]))
            }

            ast::Expression::BinaryOperator {
                lhs,
                binop: ast::BinOp::Or(_),
                rhs,
            } => {
                self.layers += 1;

                Some(qmc::Bool::Or(vec![
                    self.expression_to_qmc_bool(lhs)?,
                    self.expression_to_qmc_bool(rhs)?,
                ]))
            }

            ast::Expression::Parentheses { expression, .. } => {
                self.expression_to_qmc_bool(expression)
            }

            ast::Expression::UnaryOperator {
                unop: ast::UnOp::Not(_),
                expression,
            } => {
                self.layers += 1;

                Some(qmc::Bool::Not(Box::new(
                    self.expression_to_qmc_bool(expression)?,
                )))
            }

            ast::Expression::Value { value, .. } => match &**value {
                ast::Value::Symbol(symbol) => match symbol.token().token_type() {
                    tokenizer::TokenType::Symbol { symbol } => match symbol {
                        tokenizer::Symbol::False | tokenizer::Symbol::Nil => Some(qmc::Bool::False),
                        tokenizer::Symbol::True => Some(qmc::Bool::True),

                        other => unreachable!("Reached unknown token type {}", other),
                    },

                    tokenizer::TokenType::Identifier { .. } => self.grab_literal(expression),

                    _ => self.grab_literal(expression),
                },

                ast::Value::ParenthesesExpression(expression) => {
                    self.expression_to_qmc_bool(expression)
                }

                ast::Value::Var(_) => self.grab_literal(expression),

                _ => None,
            },

            _ => None,
        }
    }

    fn format_bool(&self, to_format: qmc::Bool) -> Option<String> {
        Some(
            match to_format {
                qmc::Bool::Term(term) => Some(
                    self.literals
                        .get(term as usize)
                        .expect("received term that wasn't inside literals")
                        .to_string(),
                ),

                qmc::Bool::And(bools) => self.format_bools_with_separator(bools, "and"),
                qmc::Bool::Or(bools) => self.format_bools_with_separator(bools, "or"),

                qmc::Bool::Not(other_bool) => {
                    Some(format!("not {}", self.format_bool(*other_bool)?))
                }

                qmc::Bool::True => Some("true".to_owned()),

                // We can't remember if this is false or nil
                qmc::Bool::False => None,
            }?
            .trim()
            .to_owned(),
        )
    }

    fn format_bools_with_separator(
        &self,
        bools: Vec<qmc::Bool>,
        separator: &str,
    ) -> Option<String> {
        Some(
            bools
                .into_iter()
                .rev()
                .filter_map(|condition| self.format_bool(condition))
                .map(|formatted| formatted.trim().to_owned())
                .collect::<Vec<_>>()
                .join(&format!(" {} ", separator)),
        )
    }
}

impl DuplicateConditionVisitor {
    fn check_expression(&mut self, expression: &ast::Expression) {
        let mut transformer = QmcBoolTransformer::default();
        let qmc_bool = match transformer.expression_to_qmc_bool(expression) {
            Some(qmc_bool) => qmc_bool,
            None => return,
        };

        if transformer.layers == 0 {
            return;
        }

        let simplified = qmc_bool.simplify();

        if simplified.len() > 1 {
            unimplemented!("complicated expression: {}", expression);
        }

        let simplified = simplified.into_iter().next().unwrap();

        if simplified != qmc_bool {
            self.diagnostics.push(Diagnostic::new(
                "duplicate_condition",
                "condition can be simplified".to_owned(),
                Label::from_node(
                    expression,
                    // Some(format!("{:?} /// {:?}", qmc_bool, simplified)),
                    transformer
                        .format_bool(simplified)
                        .map(|simplification| format!("can be simplified to `{}`", simplification)),
                ),
            ))
        }
    }
}

// We don't visit expressions since that would mean (x and y or x) would capture both
// (x and y or x) and (x and y), or (y or x).
// We only want to capture the full expression.
impl Visitor for DuplicateConditionVisitor {
    fn visit_assignment(&mut self, node: &ast::Assignment) {
        for expression in node.expressions() {
            self.check_expression(expression);
        }

        for variable in node.variables() {
            if let ast::Var::Expression(var_expression) = variable {
                if let ast::Prefix::Expression(expression) = var_expression.prefix() {
                    self.check_expression(expression);
                }

                for suffix in var_expression.suffixes() {
                    if let ast::Suffix::Index(ast::Index::Brackets { expression, .. }) = suffix {
                        self.check_expression(expression);
                    }
                }
            }
        }
    }

    fn visit_else_if(&mut self, node: &ast::ElseIf) {
        self.check_expression(node.condition());
    }

    fn visit_function_args(&mut self, function_args: &ast::FunctionArgs) {
        if let ast::FunctionArgs::Parentheses { arguments, .. } = function_args {
            for argument in arguments {
                self.check_expression(argument);
            }
        }
    }

    fn visit_if(&mut self, node: &ast::If) {
        self.check_expression(node.condition());
    }

    fn visit_generic_for(&mut self, node: &ast::GenericFor) {
        for expression in node.expressions() {
            self.check_expression(expression);
        }
    }

    fn visit_field(&mut self, node: &ast::Field) {
        match node {
            ast::Field::ExpressionKey { key, value, .. } => {
                self.check_expression(key);
                self.check_expression(value);
            }

            ast::Field::NameKey { value, .. } => {
                self.check_expression(value);
            }

            ast::Field::NoKey(value) => self.check_expression(value),

            _ => {}
        }
    }

    fn visit_local_assignment(&mut self, node: &ast::LocalAssignment) {
        for expression in node.expressions() {
            self.check_expression(expression);
        }
    }

    fn visit_numeric_for(&mut self, node: &ast::NumericFor) {
        self.check_expression(node.start());
        self.check_expression(node.end());

        if let Some(step) = node.step() {
            self.check_expression(step);
        }
    }

    fn visit_repeat(&mut self, node: &ast::Repeat) {
        self.check_expression(node.until());
    }

    fn visit_return(&mut self, node: &ast::Return) {
        for expression in node.returns() {
            self.check_expression(expression);
        }
    }

    fn visit_while(&mut self, node: &ast::While) {
        self.check_expression(node.condition());
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_duplicate_condition() {
        test_lint(
            DuplicateConditionLint::new(()).unwrap(),
            "duplicate_condition",
            "duplicate_condition",
        );
    }
}
