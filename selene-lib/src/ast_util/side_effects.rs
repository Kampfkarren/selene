use full_moon::ast;

pub trait HasSideEffects {
    fn has_side_effects(&self) -> bool;
}

impl HasSideEffects for ast::Expression<'_> {
    fn has_side_effects(&self) -> bool {
        match self {
            ast::Expression::Parentheses { expression, .. }
            | ast::Expression::UnaryOperator { expression, .. } => expression.has_side_effects(),
            ast::Expression::Value { value, .. } => value.has_side_effects(),
        }
    }
}

impl HasSideEffects for ast::Prefix<'_> {
    fn has_side_effects(&self) -> bool {
        match self {
            ast::Prefix::Expression(expression) => expression.has_side_effects(),
            ast::Prefix::Name(_) => false,
        }
    }
}

impl HasSideEffects for ast::Suffix<'_> {
    fn has_side_effects(&self) -> bool {
        match self {
            ast::Suffix::Call(_) => true,
            ast::Suffix::Index(_) => false,
        }
    }
}

impl HasSideEffects for ast::Value<'_> {
    fn has_side_effects(&self) -> bool {
        match self {
            ast::Value::Function(_)
            | ast::Value::Number(_)
            | ast::Value::String(_)
            | ast::Value::Symbol(_) => false,
            ast::Value::FunctionCall(_) => true,
            ast::Value::ParseExpression(expression) => expression.has_side_effects(),
            ast::Value::TableConstructor(table_constructor) => {
                table_constructor.iter_fields().any(|field| match field {
                    ast::Field::ExpressionKey { key, value, .. } => {
                        key.has_side_effects() || value.has_side_effects()
                    }

                    ast::Field::NameKey { value, .. } => value.has_side_effects(),

                    ast::Field::NoKey(expression) => expression.has_side_effects(),
                })
            }
            ast::Value::Var(var) => var.has_side_effects(),
        }
    }
}

impl HasSideEffects for ast::Var<'_> {
    fn has_side_effects(&self) -> bool {
        match self {
            ast::Var::Expression(var_expr) => var_expr.has_side_effects(),
            ast::Var::Name(_) => false,
        }
    }
}

impl HasSideEffects for ast::VarExpression<'_> {
    fn has_side_effects(&self) -> bool {
        self.prefix().has_side_effects()
            || self.iter_suffixes().any(HasSideEffects::has_side_effects)
    }
}
