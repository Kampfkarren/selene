use std::convert::{TryFrom, TryInto};

use full_moon::{
    ast::{self, Ast},
    node::Node,
    tokenizer::{self, Position, TokenReference},
};

mod extract_static_token;
mod loop_tracker;
pub mod name_paths;
mod purge_trivia;
pub mod scopes;
mod side_effects;
mod strip_parentheses;
pub mod visit_nodes;

pub use extract_static_token::extract_static_token;
pub use loop_tracker::LoopTracker;
pub use purge_trivia::purge_trivia;
pub use side_effects::HasSideEffects;
pub use strip_parentheses::strip_parentheses;

pub fn is_type_function(name: &str, roblox: bool) -> bool {
    name == "type" || (name == "typeof" && roblox)
}

pub fn range<N: Node, P: TryFrom<usize>>(node: N) -> (P, P)
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

pub fn first_code(ast: &Ast) -> Option<(Position, Position)> {
    match ast.nodes().stmts().next() {
        Some(first_stmt) => first_stmt.range(),
        None => ast.nodes().last_stmt().and_then(Node::range),
    }
}

pub fn is_vararg(expression: &ast::Expression) -> bool {
    if_chain::if_chain! {
        if let ast::Expression::Value { value, .. } = expression;
        if let ast::Value::Symbol(token) = &**value;
        if let tokenizer::TokenType::Symbol {
            symbol: tokenizer::Symbol::Ellipse,
        } = token.token().token_type();

        then {
            true
        } else {
            false
        }
    }
}

pub fn is_function_call(expression: &ast::Expression) -> bool {
    if let ast::Expression::Value { value, .. } = expression {
        if let ast::Value::FunctionCall(_) = &**value {
            return true;
        }
    }

    false
}

pub fn expression_to_ident(expression: &ast::Expression) -> Option<&TokenReference> {
    if let ast::Expression::Value { value, .. } = expression {
        if let ast::Value::Var(ast::Var::Name(name)) = &**value {
            return Some(name);
        }
    }

    None
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
