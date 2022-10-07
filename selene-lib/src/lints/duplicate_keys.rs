use crate::ast_util::range;

use super::*;
use std::{collections::HashMap, convert::Infallible};

use full_moon::{
    ast::{self, Ast},
    tokenizer,
    visitors::Visitor,
};

pub struct DuplicateKeysLint;

impl Lint for DuplicateKeysLint {
    type Config = ();
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Error;
    const LINT_TYPE: LintType = LintType::Correctness;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(DuplicateKeysLint)
    }

    fn pass(&self, ast: &Ast, _: &Context, _: &AstContext) -> Vec<Diagnostic> {
        let mut visitor = DuplicateKeysVisitor {
            duplicates: Vec::new(),
        };

        visitor.visit_ast(ast);

        visitor
            .duplicates
            .iter()
            .map(|duplicate| {
                Diagnostic::new_complete(
                    "duplicate_keys",
                    format!("key `{}` is already declared", duplicate.name),
                    Label::new(duplicate.position),
                    Vec::new(),
                    vec![Label::new_with_message(
                        duplicate.original_declaration,
                        format!("`{}` originally declared here", duplicate.name),
                    )],
                )
            })
            .collect()
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum KeyType {
    /// A number key type, such as `4` in `{ [4] = "foo" }`, or `1` inferred from `{"foo"}`
    Number,
    /// A string key type, or a named identifier, such as `foo` in `{ ["foo"] = "bar" }` or `{ foo = "bar" }`
    String,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Key {
    key_type: KeyType,
    name: String,
}

struct DuplicateKey {
    name: String,
    position: (usize, usize),
    original_declaration: (usize, usize),
}

struct DuplicateKeysVisitor {
    duplicates: Vec<DuplicateKey>,
}

/// Attempts to evaluate an expression key such as `"foobar"` in `["foobar"] = true` to a named identifier, `foobar`.
/// Also extracts `5` from `[5] = true`.
/// Only works for string literal expression keys, or constant number keys.
fn expression_to_key(expression: &ast::Expression) -> Option<Key> {
    if let ast::Expression::Value { value, .. } = expression {
        if let ast::Value::String(token) | ast::Value::Number(token) = &**value {
            return match token.token().token_type() {
                tokenizer::TokenType::StringLiteral { literal, .. } => Some(Key {
                    key_type: KeyType::String,
                    name: literal.to_string(),
                }),
                tokenizer::TokenType::Number { text, .. } => Some(Key {
                    key_type: KeyType::Number,
                    name: text.to_string(),
                }),
                _ => None,
            };
        }
    }

    None
}

impl DuplicateKeysVisitor {
    fn check_field(
        &mut self,
        declared_fields: &mut HashMap<Key, (usize, usize)>,
        key: Key,
        field_range: (usize, usize),
    ) {
        if let Some(original_declaration) = declared_fields.get(&key) {
            self.duplicates.push(DuplicateKey {
                name: key.name,
                position: field_range,
                original_declaration: *original_declaration,
            });
        } else {
            declared_fields.insert(key, field_range);
        }
    }
}

impl Visitor for DuplicateKeysVisitor {
    fn visit_table_constructor(&mut self, node: &ast::TableConstructor) {
        let mut declared_fields = HashMap::new();
        let mut number_index: usize = 0;

        for field in node.fields() {
            let field_range = range(field);

            #[cfg_attr(
                feature = "force_exhaustive_checks",
                deny(non_exhaustive_omitted_patterns)
            )]
            match field {
                ast::Field::NameKey { key, .. } => {
                    let key = Key {
                        key_type: KeyType::String,
                        name: key.token().to_string(),
                    };
                    self.check_field(&mut declared_fields, key, field_range);
                }

                ast::Field::ExpressionKey { key, .. } => {
                    if let Some(key) = expression_to_key(key) {
                        self.check_field(&mut declared_fields, key, field_range);
                    }
                }

                ast::Field::NoKey(_) => {
                    number_index += 1;
                    let key = Key {
                        key_type: KeyType::Number,
                        name: number_index.to_string(),
                    };
                    self.check_field(&mut declared_fields, key, field_range)
                }

                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_duplicate_keys() {
        test_lint(
            DuplicateKeysLint::new(()).unwrap(),
            "duplicate_keys",
            "duplicate_keys",
        );
    }

    #[test]
    fn test_duplicate_keys_number_indices() {
        test_lint(
            DuplicateKeysLint::new(()).unwrap(),
            "duplicate_keys",
            "number_indices",
        );
    }
}
