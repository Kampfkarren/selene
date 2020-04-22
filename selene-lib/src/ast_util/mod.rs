use std::convert::{TryFrom, TryInto};

use full_moon::node::Node;

pub mod scopes;
mod side_effects;

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
