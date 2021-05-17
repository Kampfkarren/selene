use super::*;
use crate::ast_util::range;
use std::{collections::HashSet, convert::Infallible};

use full_moon::{
    ast::{self, Ast},
    tokenizer::{TokenReference, TokenType},
    visitors::Visitor,
};
use if_chain::if_chain;

pub struct IncorrectRoactUsageLint;

impl Rule for IncorrectRoactUsageLint {
    type Config = ();
    type Error = Infallible;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(IncorrectRoactUsageLint)
    }

    fn pass(&self, ast: &Ast, context: &Context) -> Vec<Diagnostic> {
        if !context.is_roblox() {
            return Vec::new();
        }

        let mut visitor = IncorrectRoactUsageVisitor::default();
        visitor.visit_ast(ast);

        let mut diagnostics = Vec::new();

        for invalid_property in visitor.invalid_properties {
            diagnostics.push(Diagnostic::new(
                "roblox_incorrect_roact_usage",
                format!(
                    "`{}` is not a property of `{}`",
                    invalid_property.property_name, invalid_property.class_name
                ),
                Label::new(invalid_property.range),
            ));
        }

        for unused_property in visitor.unused_properties {
            diagnostics.push(Diagnostic::new(
                "roblox_incorrect_roact_usage",
                format!(
                    "`{}` will never be applied for `{}`",
                    unused_property.property_name, unused_property.class_name
                ),
                Label::new(unused_property.range),
            ));
        }

        for unknown_class in visitor.unknown_class {
            diagnostics.push(Diagnostic::new(
                "roblox_incorrect_roact_usage",
                format!("`{}` is not a valid class", unknown_class.name),
                Label::new(unknown_class.range),
            ));
        }

        diagnostics
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn rule_type(&self) -> RuleType {
        RuleType::Correctness
    }
}

fn is_roact_create_element(prefix: &ast::Prefix, suffixes: &[&ast::Suffix]) -> bool {
    if_chain! {
        if let ast::Prefix::Name(prefix_token) = prefix;
        if prefix_token.token().to_string() == "Roact";
        if suffixes.len() == 1;
        if let ast::Suffix::Index(index) = suffixes[0];
        if let ast::Index::Dot { name, .. } = index;
        then {
            name.token().to_string() == "createElement"
        } else {
            false
        }
    }
}

#[derive(Debug, Default)]
struct IncorrectRoactUsageVisitor {
    definitions_of_create_element: HashSet<String>,
    invalid_properties: Vec<InvalidProperty>,
    unused_properties: Vec<InvalidProperty>,
    unknown_class: Vec<UnknownClass>,
}

#[derive(Debug)]
struct InvalidProperty {
    class_name: String,
    property_name: String,
    range: (usize, usize),
}

#[derive(Debug)]
struct UnknownClass {
    name: String,
    range: (usize, usize),
}

impl IncorrectRoactUsageVisitor {
    fn check_class_name(
        &mut self,
        token: &TokenReference,
    ) -> Option<&'static rbx_reflection::RbxClassDescriptor> {
        let name = if let TokenType::StringLiteral { literal, .. } = &*token.token_type() {
            literal.to_string()
        } else {
            return None;
        };

        match rbx_reflection::get_class_descriptor(&name) {
            option @ Some(_) => option,

            None => {
                self.unknown_class.push(UnknownClass {
                    name,
                    range: range(token),
                });

                None
            }
        }
    }
}

impl Visitor<'_> for IncorrectRoactUsageVisitor {
    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        // Check if caller is Roact.createElement or a variable defined to it
        let mut suffixes = call.iter_suffixes().collect::<Vec<_>>();
        let call_suffix = suffixes.pop();

        let mut check = false;

        if suffixes.is_empty() {
            // Call is foo(), not foo.bar()
            // Check if foo is a variable for Roact.createElement
            if let ast::Prefix::Name(name) = call.prefix() {
                if self
                    .definitions_of_create_element
                    .contains(&name.token().to_string())
                {
                    check = true;
                }
            }
        } else if suffixes.len() == 1 {
            // Call is foo.bar()
            // Check if foo.bar is Roact.createElement
            check = is_roact_create_element(call.prefix(), &suffixes);
        }

        if !check {
            return;
        }

        let (mut class, arguments) = if_chain! {
            if let Some(ast::Suffix::Call(call)) = call_suffix;
            if let ast::Call::AnonymousCall(arguments) = call;
            if let ast::FunctionArgs::Parentheses { arguments, .. } = arguments;
            if arguments.len() >= 2;
            let mut iter = arguments.iter();

            // Get first argument, check if it is a Roblox class
            let name_arg = iter.next().unwrap();
            if let ast::Expression::Value { value, binop, .. } = name_arg;
            if binop.is_none();
            if let ast::Value::String(token) = &**value;
            if let Some(class) = self.check_class_name(&token);

            // Get second argument, check if it is a table
            let arg = iter.next().unwrap();
            if let ast::Expression::Value { value, binop, .. } = arg;
            if binop.is_none();
            if let ast::Value::TableConstructor(table) = &**value;

            then {
                (class, table)
            } else {
                return;
            }
        };

        let mut valid_properties = HashSet::new();

        loop {
            for (property, descriptor) in class.iter_property_descriptors() {
                if descriptor.is_canonical() {
                    valid_properties.insert(property);
                }
            }

            if let Some(superclass) = class.superclass() {
                class = rbx_reflection::get_class_descriptor(superclass).unwrap();
            } else {
                break;
            }
        }

        for field in arguments.fields() {
            if let ast::Field::NameKey { key, .. } = field {
                let property_name = key.token().to_string();
                if !valid_properties.contains(property_name.as_str()) {
                    self.invalid_properties.push(InvalidProperty {
                        class_name: class.name().to_string(),
                        property_name,
                        range: range(key),
                    });
                } else if property_name == "Name" || property_name == "Parent" {
                    self.unused_properties.push(InvalidProperty {
                        class_name: class.name().to_string(),
                        property_name,
                        range: range(key),
                    })
                }
            }
        }
    }

    fn visit_local_assignment(&mut self, node: &ast::LocalAssignment) {
        for (name, expr) in node.name_list().iter().zip(node.expr_list().iter()) {
            if_chain! {
                if let ast::Expression::Value { value, binop, .. } = expr;
                if binop.is_none();
                if let ast::Value::Var(var) = &**value;
                if let ast::Var::Expression(var_expr) = var;
                if is_roact_create_element(var_expr.prefix(), &var_expr.iter_suffixes().collect::<Vec<_>>());
                then {
                    self.definitions_of_create_element.insert(name.token().to_string());
                }
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_roblox_incorrect_roact_usage() {
        test_lint(
            IncorrectRoactUsageLint::new(()).unwrap(),
            "roblox_incorrect_roact_usage",
            "roblox_incorrect_roact_usage",
        );
    }
}
