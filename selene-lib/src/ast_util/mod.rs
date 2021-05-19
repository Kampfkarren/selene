use std::convert::{TryFrom, TryInto};

use full_moon::{
    ast::{self, Ast},
    node::Node,
    tokenizer::{self, Position},
};

mod purge_trivia;
pub mod scopes;
mod side_effects;
pub mod visit_nodes;

pub use purge_trivia::purge_trivia;
pub use side_effects::HasSideEffects;

pub fn is_type_function(name: &str, roblox: bool) -> bool {
    name == "type" || (name == "typeof" && roblox)
}

pub fn range<'a, N: Node<'a>, P: TryFrom<usize>>(node: N) -> (P, P)
where
    <P as TryFrom<usize>>::Error: std::fmt::Debug,
{
    let (start, end) = node.range().unwrap();
    (
        start
            .bytes()
            .try_into()
            .expect("range start_position couldn't convert"),
        end.bytes()
            .try_into()
            .expect("range end_position couldn't convert"),
    )
}

pub fn first_code<'ast>(ast: &Ast<'ast>) -> Option<(Position, Position)> {
    match ast.nodes().iter_stmts().next() {
        Some(first_stmt) => first_stmt.range(),
        None => ast.nodes().last_stmt().and_then(Node::range),
    }
}

pub fn is_vararg(expression: &ast::Expression) -> bool {
    if_chain::if_chain! {
        if let ast::Expression::Value { value, .. } = expression;
        if let ast::Value::Symbol(token) = &**value;
        if let tokenizer::TokenType::Symbol { symbol } = token.token().token_type();
        if let tokenizer::Symbol::Ellipse = symbol;

        then {
            true
        } else {
            false
        }
    }
}

pub fn is_function_call(expression: &ast::Expression) -> bool {
    if_chain::if_chain! {
        if let ast::Expression::Value { value, .. } = expression;
        if let ast::Value::FunctionCall(_) = &**value;

        then {
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use full_moon::parse;

    #[test]
    fn test_first_code() {
        let code = r"-- hello world
        -- line two
        lineThree()
        -- line four";

        let first_code = first_code(&parse(code).unwrap()).expect("first_code returned None");

        assert_eq!(first_code.0.line(), 3);
    }
}
