// Unimplemented methods will report having side effects for safety reasons.

use full_moon::ast;

pub trait HasSideEffects {
    fn has_side_effects(&self) -> bool;
}

impl HasSideEffects for ast::Expression {
    fn has_side_effects(&self) -> bool {
        #[cfg_attr(
            feature = "force_exhaustive_checks",
            deny(non_exhaustive_omitted_patterns)
        )]
        match self {
            ast::Expression::BinaryOperator { lhs, rhs, .. } => {
                lhs.has_side_effects() || rhs.has_side_effects()
            }
            ast::Expression::Parentheses { expression, .. }
            | ast::Expression::UnaryOperator { expression, .. } => expression.has_side_effects(),
            ast::Expression::Value { value, .. } => value.has_side_effects(),
            _ => false,
        }
    }
}

impl HasSideEffects for ast::Prefix {
    fn has_side_effects(&self) -> bool {
        #[cfg_attr(
            feature = "force_exhaustive_checks",
            deny(non_exhaustive_omitted_patterns)
        )]
        match self {
            ast::Prefix::Expression(expression) => expression.has_side_effects(),
            ast::Prefix::Name(_) => false,
            _ => true,
        }
    }
}

impl HasSideEffects for ast::Suffix {
    fn has_side_effects(&self) -> bool {
        #[cfg_attr(
            feature = "force_exhaustive_checks",
            deny(non_exhaustive_omitted_patterns)
        )]
        match self {
            ast::Suffix::Call(_) => true,
            ast::Suffix::Index(_) => false,
            _ => true,
        }
    }
}

impl HasSideEffects for ast::Value {
    fn has_side_effects(&self) -> bool {
        #[cfg_attr(
            feature = "force_exhaustive_checks",
            deny(non_exhaustive_omitted_patterns)
        )]
        match self {
            ast::Value::Function(_)
            | ast::Value::Number(_)
            | ast::Value::String(_)
            | ast::Value::Symbol(_) => false,
            ast::Value::FunctionCall(_) => true,
            ast::Value::ParenthesesExpression(expression) => expression.has_side_effects(),
            ast::Value::TableConstructor(table_constructor) => table_constructor
                .fields()
                .into_iter()
                .any(|field| match field {
                    ast::Field::ExpressionKey { key, value, .. } => {
                        key.has_side_effects() || value.has_side_effects()
                    }

                    ast::Field::NameKey { value, .. } => value.has_side_effects(),

                    ast::Field::NoKey(expression) => expression.has_side_effects(),

                    _ => true,
                }),
            ast::Value::Var(var) => var.has_side_effects(),

            #[cfg(feature = "roblox")]
            ast::Value::IfExpression(if_expression) => {
                if if_expression.if_expression().has_side_effects()
                    || if_expression.condition().has_side_effects()
                    || if_expression.else_expression().has_side_effects()
                {
                    return true;
                }

                if let Some(else_if_expressions) = if_expression.else_if_expressions() {
                    for else_if_expression in else_if_expressions {
                        if else_if_expression.condition().has_side_effects()
                            || else_if_expression.expression().has_side_effects()
                        {
                            return true;
                        }
                    }
                }

                false
            }

            #[cfg(feature = "roblox")]
            ast::Value::InterpolatedString(interpolated_string) => {
                for expression in interpolated_string.expressions() {
                    if expression.has_side_effects() {
                        return true;
                    }
                }

                false
            }

            _ => true,
        }
    }
}

impl HasSideEffects for ast::Var {
    fn has_side_effects(&self) -> bool {
        #[cfg_attr(
            feature = "force_exhaustive_checks",
            deny(non_exhaustive_omitted_patterns)
        )]
        match self {
            ast::Var::Expression(var_expr) => var_expr.has_side_effects(),
            ast::Var::Name(_) => false,
            _ => true,
        }
    }
}

impl HasSideEffects for ast::VarExpression {
    fn has_side_effects(&self) -> bool {
        self.prefix().has_side_effects() || self.suffixes().any(HasSideEffects::has_side_effects)
    }
}
