use std::convert::{TryFrom, TryInto};

use full_moon::node::Node;

pub mod scopes;
mod side_effects;

pub use side_effects::HasSideEffects;

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
            .expect("range start_position couldn't convert"),
    )
}
