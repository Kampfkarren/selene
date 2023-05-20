use super::*;
use crate::{
    ast_util::{range, strip_parentheses},
    standard_library::RobloxClass,
};
use std::{
    collections::{BTreeMap, HashMap},
    convert::Infallible,
};

use full_moon::{
    ast::{self, Ast},
    tokenizer::{TokenReference, TokenType},
    visitors::Visitor,
};
use if_chain::if_chain;

pub struct IncorrectRoactUsageLint;

// Assumes string includes quotes at start and end
fn is_lua_valid_table_key_identifier(string: &String) -> bool {
    // Valid identifier cannot start with numbers
    let first_char = string.chars().nth(1).unwrap();
    if !first_char.is_alphabetic() && first_char != '_' {
        return false;
    }

    string
        .chars()
        .skip(1)
        .take(string.len() - 2)
        .all(|c| c.is_alphanumeric() || c == '_')
}

fn get_lua_table_key_format(expression: &ast::Expression) -> String {
    match expression {
        ast::Expression::Value { value, .. } => match &**value {
            ast::Value::String(token) => {
                let string = token.to_string();
                if is_lua_valid_table_key_identifier(&string) {
                    string[1..string.len() - 1].to_string()
                } else {
                    format!("[{}]", string)
                }
            }
            _ => format!("[{}]", expression),
        },
        _ => format!("[{}]", expression),
    }
}

impl Lint for IncorrectRoactUsageLint {
    type Config = ();
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Error;
    const LINT_TYPE: LintType = LintType::Correctness;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(IncorrectRoactUsageLint)
    }

    fn pass(&self, ast: &Ast, context: &Context, _: &AstContext) -> Vec<Diagnostic> {
        if !context.is_roblox() {
            return Vec::new();
        }

        let roblox_classes = &context.standard_library.roblox_classes;

        // Old roblox standard library
        if roblox_classes.is_empty() {
            return Vec::new();
        }

        let mut visitor = IncorrectRoactUsageVisitor {
            definitions_of_create_element: HashMap::new(),
            invalid_events: Vec::new(),
            invalid_properties: Vec::new(),
            unknown_class: Vec::new(),

            roblox_classes,
        };

        visitor.visit_ast(ast);

        let mut diagnostics = Vec::new();

        for invalid_event in visitor.invalid_events {
            diagnostics.push(Diagnostic::new(
                "roblox_incorrect_roact_usage",
                format!(
                    "`{}` is not a valid event for `{}`",
                    invalid_event.event_name, invalid_event.class_name
                ),
                Label::new(invalid_event.range),
            ));
        }

        for invalid_property in visitor.invalid_properties {
            match invalid_property.property_name.as_str() {
                "Name" => {
                    diagnostics.push(Diagnostic::new_complete(
                        "roblox_incorrect_roact_usage",
                        format!(
                            "`{}` is assigned through the element's key for Roblox instances",
                            invalid_property.property_name
                        ),
                        Label::new(invalid_property.range),
                        vec![format!(
                            "try: {} = {}(...)",
                            get_lua_table_key_format(&invalid_property.property_value),
                            invalid_property.create_element_expression,
                        )],
                        Vec::new(),
                    ));
                }
                _ => {
                    diagnostics.push(Diagnostic::new(
                        "roblox_incorrect_roact_usage",
                        format!(
                            "`{}` is not a property of `{}`",
                            invalid_property.property_name, invalid_property.class_name
                        ),
                        Label::new(invalid_property.range),
                    ));
                }
            }
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
}

fn is_roact_or_react_create_element(
    prefix: &ast::Prefix,
    suffixes: &[&ast::Suffix],
) -> Option<LibraryName> {
    if_chain! {
        if let ast::Prefix::Name(prefix_token) = prefix;
        if let Some(library_name) = match prefix_token.token().to_string().as_str() {
            "Roact" => Some(LibraryName::Roact),
            "React" => Some(LibraryName::React),
            _ => None,
        };
        if suffixes.len() == 1;
        if let ast::Suffix::Index(ast::Index::Dot { name, .. }) = suffixes[0];
        if name.token().to_string() == "createElement";
        then {
            Some(library_name)
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum LibraryName {
    Roact,
    React,
}

#[derive(Debug)]
struct IncorrectRoactUsageVisitor<'a> {
    definitions_of_create_element: HashMap<String, LibraryName>,
    invalid_events: Vec<InvalidEvent>,
    invalid_properties: Vec<InvalidProperty>,
    unknown_class: Vec<UnknownClass>,

    roblox_classes: &'a BTreeMap<String, RobloxClass>,
}

#[derive(Debug)]
struct InvalidEvent {
    class_name: String,
    event_name: String,
    range: (usize, usize),
}

#[derive(Debug)]
struct InvalidProperty {
    class_name: String,
    property_name: String,
    property_value: ast::Expression,
    create_element_expression: String,
    range: (usize, usize),
}

#[derive(Debug)]
struct UnknownClass {
    name: String,
    range: (usize, usize),
}

impl<'a> IncorrectRoactUsageVisitor<'a> {
    fn check_class_name(&mut self, token: &TokenReference) -> Option<(String, &'a RobloxClass)> {
        let name = if let TokenType::StringLiteral { literal, .. } = token.token_type() {
            literal.to_string()
        } else {
            return None;
        };

        match self.roblox_classes.get(&name) {
            Some(roblox_class) => Some((name, roblox_class)),

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

impl<'a> Visitor for IncorrectRoactUsageVisitor<'a> {
    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        // Check if caller is Roact.createElement or a variable defined to it
        let mut suffixes = call.suffixes().collect::<Vec<_>>();
        let call_suffix = suffixes.pop();

        let mut library_name = None;
        let mut create_element_expression = String::new();

        if suffixes.is_empty() {
            // Call is foo(), not foo.bar()
            // Check if foo is a variable for Roact.createElement
            if let ast::Prefix::Name(name) = call.prefix() {
                if let Some(react_name) = self
                    .definitions_of_create_element
                    .get(&name.token().to_string())
                {
                    library_name = Some(*react_name);
                    create_element_expression = name.token().to_string();
                }
            }
        } else if suffixes.len() == 1 {
            // Call is foo.bar()
            // Check if foo.bar is Roact.createElement
            library_name = is_roact_or_react_create_element(call.prefix(), &suffixes);

            if let ast::Prefix::Name(name) = call.prefix() {
                create_element_expression = format!("{}{}", name.token(), suffixes[0]);
            }
        }

        let react_name = match library_name {
            Some(name) => name,
            None => return,
        };

        let ((name, class), arguments) = if_chain! {
            if let Some(ast::Suffix::Call(ast::Call::AnonymousCall(
                ast::FunctionArgs::Parentheses { arguments, .. }
            ))) = call_suffix;
            if !arguments.is_empty();
            let mut iter = arguments.iter();

            // Get first argument, check if it is a Roblox class
            let name_arg = iter.next().unwrap();
            if let ast::Expression::Value { value, .. } = name_arg;
            if let ast::Value::String(token) = &**value;
            if let Some((name, class)) = self.check_class_name(token);

            // Get second argument, check if it is a table
            if let Some(ast::Expression::Value { value, .. }) = iter.next();
            if let ast::Value::TableConstructor(table) = &**value;

            then {
                ((name, class), table)
            } else {
                return;
            }
        };

        for field in arguments.fields() {
            match field {
                ast::Field::NameKey { key, value, .. } => {
                    let property_name = key.token().to_string();

                    if react_name == LibraryName::React
                        && ["ref", "key", "children"].contains(&property_name.as_str())
                    {
                        continue;
                    }

                    if !class.has_property(self.roblox_classes, &property_name)
                        || property_name == "Name"
                    {
                        self.invalid_properties.push(InvalidProperty {
                            class_name: name.clone(),
                            property_name,
                            property_value: value.clone(),
                            create_element_expression: create_element_expression.clone().to_owned(),
                            range: range(key),
                        });
                    }
                }

                ast::Field::ExpressionKey { brackets, key, .. } => {
                    let key = strip_parentheses(key);

                    if_chain::if_chain! {
                        if let ast::Expression::Value { value, .. } = key;
                        if let ast::Value::Var(ast::Var::Expression(var_expression)) = &**value;

                        if let ast::Prefix::Name(constant_roact_name) = var_expression.prefix();
                        if ["Roact", "React"].contains(&constant_roact_name.token().to_string().as_str());

                        let mut suffixes = var_expression.suffixes();
                        if let Some(ast::Suffix::Index(ast::Index::Dot { name: constant_event_name, .. })) = suffixes.next();
                        if constant_event_name.token().to_string() == "Event";

                        if let Some(ast::Suffix::Index(ast::Index::Dot { name: event_name, .. })) = suffixes.next();
                        then {
                            let event_name = event_name.token().to_string();
                            if !class.has_event(self.roblox_classes, &event_name) {
                                self.invalid_events.push(InvalidEvent {
                                    class_name: name.clone(),
                                    event_name,
                                    range: range(brackets),
                                });
                            }
                        }
                    }
                }

                _ => {}
            }
        }
    }

    fn visit_local_assignment(&mut self, node: &ast::LocalAssignment) {
        for (name, expr) in node.names().iter().zip(node.expressions().iter()) {
            if_chain! {
                if let ast::Expression::Value { value, .. } = expr;
                if let ast::Value::Var(ast::Var::Expression(var_expr)) = &**value;
                if let Some(roact_or_react) = is_roact_or_react_create_element(var_expr.prefix(), &var_expr.suffixes().collect::<Vec<_>>());
                then {
                    self.definitions_of_create_element.insert(name.token().to_string(), roact_or_react);
                }
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_mixed_roact_react_usage() {
        test_lint(
            IncorrectRoactUsageLint::new(()).unwrap(),
            "roblox_incorrect_roact_usage",
            "mixed_roact_react_usage",
        );
    }

    #[test]
    fn test_old_roblox_std() {
        test_lint(
            IncorrectRoactUsageLint::new(()).unwrap(),
            "roblox_incorrect_roact_usage",
            "old_roblox_std",
        );
    }

    #[test]
    fn test_roblox_incorrect_react_usage() {
        test_lint(
            IncorrectRoactUsageLint::new(()).unwrap(),
            "roblox_incorrect_roact_usage",
            "roblox_incorrect_react_usage",
        );
    }

    #[test]
    fn test_roblox_incorrect_roact_usage() {
        test_lint(
            IncorrectRoactUsageLint::new(()).unwrap(),
            "roblox_incorrect_roact_usage",
            "roblox_incorrect_roact_usage",
        );
    }
}
