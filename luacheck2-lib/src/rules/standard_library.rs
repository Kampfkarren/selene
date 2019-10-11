use super::*;
use std::convert::Infallible;

use full_moon::{
    ast::{self, Ast},
    node::Node,
    tokenizer::{Token, TokenKind, TokenReference},
    visitors::Visitor,
};

pub struct StandardLibraryLint;

impl Rule for StandardLibraryLint {
    type Config = ();
    type Error = Infallible;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(StandardLibraryLint)
    }

    fn pass(&self, ast: &Ast, context: &Context) -> Vec<Diagnostic> {
        let mut visitor = StandardLibraryVisitor {
            standard_library: &context.standard_library,
        };

        visitor.visit_ast(ast);

        unimplemented!()
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn rule_type(&self) -> RuleType {
        RuleType::Correctness
    }
}

pub struct StandardLibraryVisitor<'std> {
    standard_library: &'std StandardLibrary,
}

fn name_path(expression: &ast::Expression) -> Option<Vec<String>> {
    if let ast::Expression::Value { value, .. } = expression {
        if let ast::Value::Var(var) = &**value {
            match var {
                ast::Var::Expression(expression) => {
                    if let ast::Prefix::Name(ref name) = expression.prefix() {
                        let mut names = Vec::new();
                        names.push(name.to_string());

                        for suffix in expression.iter_suffixes() {
                            if let ast::Suffix::Index(index) = suffix {
                                if let ast::Index::Dot { name, .. } = index {
                                    names.push(name.to_string());
                                } else {
                                    return None;
                                }
                            }
                        }

                        Some(names)
                    } else {
                        None
                    }
                }

                ast::Var::Name(name) => Some(vec![name.to_string()]),
            }
        } else {
            None
        }
    } else {
        None
    }
}

// TODO: Test shadowing
impl Visitor<'_> for StandardLibraryVisitor<'_> {
    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        let field = match call.prefix() {
            ast::Prefix::Expression(expression) => {
                if let Some(name_path) = name_path(expression) {
                    self.standard_library.find_global(name_path)
                } else {
                    None
                }
            }

            ast::Prefix::Name(ref name) => {
                self.standard_library.find_global(vec![name.to_string()])
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_path() {
        let ast = full_moon::parse("local x = foo; local y = foo.bar.baz").unwrap();

        struct NamePathTestVisitor {
            paths: Vec<Vec<String>>,
        }

        impl Visitor<'_> for NamePathTestVisitor {
            fn visit_local_assignment(&mut self, node: &ast::LocalAssignment) {
                self.paths.push(
                    name_path(node.expr_list().into_iter().next().unwrap())
                        .expect("name_path returned None"),
                );
            }
        }

        let mut visitor = NamePathTestVisitor { paths: Vec::new() };

        visitor.visit_ast(&ast);

        assert_eq!(
            visitor.paths,
            vec![
                vec!["foo".to_owned()],
                vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()],
            ]
        );
    }
}
