use full_moon::{ast, tokenizer::TokenReference, visitors::VisitorMut};

struct TriviaPurger;

impl VisitorMut for TriviaPurger {
    fn visit_token_reference(&mut self, token: TokenReference) -> TokenReference {
        TokenReference::new(Vec::new(), token.token().to_owned(), Vec::new())
    }
}

/// Returns a new Ast without any trivia
pub fn purge_trivia(ast: ast::Ast) -> ast::Ast {
    TriviaPurger.visit_ast(ast)
}
