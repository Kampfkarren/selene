use full_moon::{
    tokenizer::TokenReference,
    visitors::{VisitMut, VisitorMut},
};

struct TriviaPurger;

impl VisitorMut for TriviaPurger {
    fn visit_token_reference(&mut self, token: TokenReference) -> TokenReference {
        TokenReference::new(Vec::new(), token.token().to_owned(), Vec::new())
    }
}

/// Returns a new Ast without any trivia
#[profiling::function]
pub fn purge_trivia<V: Clone + VisitMut>(node: &V) -> V {
    node.clone().visit_mut(&mut TriviaPurger)
}
