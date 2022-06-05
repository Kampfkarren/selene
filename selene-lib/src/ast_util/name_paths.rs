use full_moon::ast;

pub fn take_while_keep_going(suffix: &ast::Suffix, keep_going: &mut bool) -> bool {
    let result = *keep_going;
    *keep_going = !matches!(suffix, ast::Suffix::Call(_));
    result
}

pub fn name_path_from_prefix_suffix<'a, S: Iterator<Item = &'a ast::Suffix>>(
    prefix: &'a ast::Prefix,
    suffixes: S,
) -> Option<Vec<String>> {
    if let ast::Prefix::Name(ref name) = prefix {
        let mut names = vec![name.token().to_string()];

        let mut keep_going = true;

        for suffix in suffixes.take_while(|suffix| take_while_keep_going(suffix, &mut keep_going)) {
            match suffix {
                ast::Suffix::Call(call) => {
                    if let ast::Call::MethodCall(method_call) = call {
                        names.push(method_call.name().token().to_string());
                    }
                }

                ast::Suffix::Index(ast::Index::Dot { name, .. }) => {
                    names.push(name.token().to_string());
                }

                _ => return None,
            }
        }

        Some(names)
    } else {
        None
    }
}

pub fn name_path(expression: &ast::Expression) -> Option<Vec<String>> {
    if let ast::Expression::Value { value, .. } = expression {
        if let ast::Value::Var(var) = &**value {
            match var {
                ast::Var::Expression(expression) => {
                    name_path_from_prefix_suffix(expression.prefix(), expression.suffixes())
                }

                ast::Var::Name(name) => Some(vec![name.to_string()]),

                _ => None,
            }
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use full_moon::visitors::Visitor;

    #[test]
    fn test_name_path() {
        let ast = full_moon::parse("local x = foo; local y = foo.bar.baz").unwrap();

        struct NamePathTestVisitor {
            paths: Vec<Vec<String>>,
        }

        impl Visitor for NamePathTestVisitor {
            fn visit_local_assignment(&mut self, node: &ast::LocalAssignment) {
                self.paths.push(
                    name_path(node.expressions().into_iter().next().unwrap())
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
