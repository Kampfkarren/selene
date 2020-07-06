use full_moon::{ast, tokenizer::TokenReference, visitors::VisitorMut};

struct TriviaPurger;

impl<'ast> VisitorMut<'ast> for TriviaPurger {
    fn visit_token_reference(&mut self, token: TokenReference<'ast>) -> TokenReference<'ast> {
        TokenReference::new(Vec::new(), token.token().to_owned(), Vec::new())
    }
}

/// Returns a new Ast without any trivia
pub fn purge_trivia<'ast>(ast: ast::Ast<'ast>) -> ast::Ast<'ast> {
    TriviaPurger.visit_ast(ast)
}
