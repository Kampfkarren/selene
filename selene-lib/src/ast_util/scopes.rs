use std::{borrow::Cow, collections::HashSet};

use full_moon::{
    ast,
    node::Node,
    tokenizer::{Symbol, TokenKind, TokenReference, TokenType},
    visitors::Visitor,
};
use id_arena::{Arena, Id};

type Range = (usize, usize);

#[derive(Debug, Default)]
pub struct ScopeManager {
    pub scopes: Arena<Scope>,
    pub references: Arena<Reference>,
    pub variables: Arena<Variable>,
    pub initial_scope: Option<Id<Scope>>,
}

impl ScopeManager {
    pub fn new(ast: &ast::Ast) -> Self {
        let scope_visitor = ScopeVisitor::from_ast(ast);
        scope_visitor.scope_manager
    }

    pub fn reference_at_byte(&self, byte: usize) -> Option<&Reference> {
        for (_, reference) in &self.references {
            if byte >= reference.identifier.0 && byte <= reference.identifier.1 {
                return Some(reference);
            }
        }

        None
    }

    fn variable_in_scope(&self, scope: Id<Scope>, variable_name: &str) -> VariableInScope {
        if let Some(scope) = self.scopes.get(scope) {
            for variable_id in scope.variables.iter().rev() {
                let variable = &self.variables[*variable_id];
                if variable.name == variable_name {
                    return VariableInScope::Found(*variable_id);
                }
            }

            if scope.blocked.iter().any(|blocked| blocked == variable_name) {
                return VariableInScope::Blocked;
            }
        }

        VariableInScope::NotFound
    }
}

#[derive(Debug, Default)]
pub struct Scope {
    block: Range,
    references: Vec<Id<Reference>>,
    variables: Vec<Id<Variable>>,
    blocked: Vec<Cow<'static, str>>,
}

#[derive(Debug)]
pub struct Reference {
    pub identifier: Range,
    pub name: String,
    pub resolved: Option<Id<Variable>>,
    // TODO: Does this matter even?
    pub write_expr: Option<Range>,
    pub scope_id: Id<Scope>,
    pub read: bool,
    pub write: bool,
}

#[derive(Debug, Default)]
pub struct Variable {
    pub definitions: Vec<Range>,
    pub identifiers: Vec<Range>,
    pub name: String,
    pub references: Vec<Id<Reference>>,
    pub shadowed: Option<Id<Variable>>,
    pub is_self: bool,
}

#[derive(Default)]
struct ScopeVisitor {
    scope_manager: ScopeManager,
    scope_stack: Vec<Id<Scope>>,

    // sigh
    else_blocks: HashSet<Range>,
}

#[derive(Debug)]
pub enum VariableInScope {
    Found(Id<Variable>),
    NotFound,
    Blocked,
}

fn create_scope<N: Node>(node: N) -> Option<Scope> {
    if let Some((start, end)) = node.range() {
        Some(Scope {
            block: (start.bytes(), end.bytes()),
            blocked: Vec::new(),
            references: Vec::new(),
            variables: Vec::new(),
        })
    } else {
        None
    }
}

fn range<N: Node>(node: N) -> (usize, usize) {
    let (start, end) = node.range().unwrap();
    (start.bytes(), end.bytes())
}

impl ScopeVisitor {
    fn from_ast(ast: &ast::Ast) -> Self {
        if let Some(scope) = create_scope(ast.nodes()) {
            let mut scopes = Arena::new();
            let id = scopes.alloc(scope);

            let mut output = ScopeVisitor {
                scope_manager: ScopeManager {
                    scopes,
                    references: Arena::new(),
                    variables: Arena::new(),
                    initial_scope: Some(id),
                },

                scope_stack: vec![id],
                else_blocks: HashSet::new(),
            };

            assert!(output.scope_stack.len() == 1, "scopes not all popped");

            output.visit_ast(ast);
            output
        } else {
            ScopeVisitor::default()
        }
    }

    fn current_scope_id(&self) -> Id<Scope> {
        *self.scope_stack.last().unwrap()
    }

    fn current_scope(&mut self) -> &mut Scope {
        self.scope_manager
            .scopes
            .get_mut(self.current_scope_id())
            .unwrap()
    }

    fn find_variable(&self, variable_name: &str) -> Option<(Id<Variable>, Id<Scope>)> {
        for scope_id in self.scope_stack.iter().rev().copied() {
            match self
                .scope_manager
                .variable_in_scope(scope_id, variable_name)
            {
                VariableInScope::Found(id) => return Some((id, scope_id)),
                VariableInScope::NotFound => {}
                VariableInScope::Blocked => return None,
            }
        }

        None
    }

    fn read_expression(&mut self, expression: &ast::Expression) {
        match expression {
            ast::Expression::Parentheses { expression, .. }
            | ast::Expression::UnaryOperator { expression, .. } => {
                self.read_expression(expression);
            }

            ast::Expression::BinaryOperator { lhs, rhs, .. } => {
                self.read_expression(lhs);
                self.read_expression(rhs);
            }

            ast::Expression::Value { value, .. } => match &**value {
                ast::Value::Function((name, _)) => {
                    self.read_name(name);
                }

                ast::Value::FunctionCall(call) => {
                    self.read_prefix(call.prefix());
                    for suffix in call.suffixes() {
                        self.read_suffix(suffix);
                    }
                }

                ast::Value::TableConstructor(table) => {
                    self.read_table_constructor(table);
                }

                ast::Value::ParenthesesExpression(expression) => self.read_expression(expression),

                ast::Value::Symbol(symbol) => {
                    if *symbol.token_type()
                        == (TokenType::Symbol {
                            symbol: Symbol::Ellipse,
                        })
                    {
                        self.read_name(symbol);
                    }
                }

                ast::Value::Var(var) => self.read_var(var),

                _ => {}
            },

            _ => {}
        }
    }

    fn read_prefix(&mut self, prefix: &ast::Prefix) {
        match prefix {
            ast::Prefix::Expression(expression) => self.read_expression(expression),
            ast::Prefix::Name(name) => self.read_name(name),
            _ => {}
        }
    }

    fn read_suffix(&mut self, suffix: &ast::Suffix) {
        match suffix {
            ast::Suffix::Call(call) => self.visit_call(call),
            ast::Suffix::Index(index) => self.visit_index(index),
            _ => {}
        }
    }

    fn read_name(&mut self, token: &TokenReference) {
        if token.token_kind() == TokenKind::Identifier
            || *token.token_type()
                == (TokenType::Symbol {
                    symbol: Symbol::Ellipse,
                })
        {
            self.reference_variable(
                &token.token().to_string(),
                Reference {
                    identifier: range(token),
                    name: token.token().to_string(),
                    read: true,
                    resolved: None,
                    scope_id: self.current_scope_id(),
                    write: false,
                    write_expr: None,
                },
            );
        }
    }

    fn read_table_constructor(&mut self, table: &ast::TableConstructor) {
        for field in table.fields() {
            match field {
                ast::Field::ExpressionKey { key, value, .. } => {
                    self.read_expression(key);
                    self.read_expression(value);
                }

                ast::Field::NameKey { value, .. } => {
                    self.read_expression(value);
                }

                ast::Field::NoKey(value) => {
                    self.read_expression(value);
                }

                _ => {}
            }
        }
    }

    fn read_var(&mut self, var: &ast::Var) {
        match var {
            ast::Var::Expression(var_expr) => {
                self.read_prefix(var_expr.prefix());
                for suffix in var_expr.suffixes() {
                    self.read_suffix(suffix);
                }
            }

            ast::Var::Name(name) => self.read_name(name),

            _ => {}
        }
    }

    fn write_name(&mut self, token: &TokenReference, write_expr: Option<Range>) {
        if token.token_kind() == TokenKind::Identifier {
            self.reference_variable(
                &token.token().to_string(),
                Reference {
                    identifier: range(token),
                    name: token.token().to_string(),
                    read: false,
                    resolved: None,
                    scope_id: self.current_scope_id(),
                    write: true,
                    write_expr,
                },
            );
        }
    }

    fn define_name(&mut self, token: &TokenReference, definition_range: Range) {
        self.define_name_full(&token.token().to_string(), range(token), definition_range);
    }

    fn define_name_full_with_variable(
        &mut self,
        name: &str,
        range: Range,
        definition_range: Range,
        variable: Variable,
    ) -> Id<Variable> {
        let shadowed = self.find_variable(name).map(|(var, _)| var);

        let id = self.scope_manager.variables.alloc(Variable {
            name: name.to_owned(),
            shadowed,
            ..variable
        });

        self.current_scope().variables.push(id);

        let variable = &mut self.scope_manager.variables[id];

        variable.definitions.push(definition_range);
        variable.identifiers.push(range);

        id
    }

    fn define_name_full(
        &mut self,
        name: &str,
        range: Range,
        definition_range: Range,
    ) -> Id<Variable> {
        self.define_name_full_with_variable(name, range, definition_range, Variable::default())
    }

    fn try_hoist(&mut self) {
        let latest_reference_id = *self.current_scope().references.last().unwrap();
        let (name, identifier, write_expr) = {
            let reference = self
                .scope_manager
                .references
                .get(latest_reference_id)
                .unwrap();

            (
                reference.name.to_owned(),
                reference.identifier,
                reference.identifier, // This is the write_expr, but it's not great
            )
        };

        if self.find_variable(&name).is_none() {
            let id = self.define_name_full(&name, identifier, write_expr);

            for (_, reference) in &mut self.scope_manager.references {
                if reference.read && reference.name == name && reference.resolved.is_none() {
                    reference.resolved = Some(id);
                }
            }
        }
    }

    fn reference_variable(&mut self, name: &str, mut reference: Reference) {
        let reference_id = if let Some((variable, _)) = self.find_variable(name) {
            reference.resolved = Some(variable);

            let reference_id = self.scope_manager.references.alloc(reference);

            self.scope_manager
                .variables
                .get_mut(variable)
                .unwrap()
                .references
                .push(reference_id);

            reference_id
        } else {
            self.scope_manager.references.alloc(reference)
        };

        self.current_scope().references.push(reference_id);
    }

    fn open_scope<N: Node>(&mut self, node: N) {
        let scope = create_scope(node).unwrap_or_else(Default::default);
        let scope_id = self.scope_manager.scopes.alloc(scope);
        self.scope_stack.push(scope_id);
    }

    fn close_scope(&mut self) {
        self.scope_stack.pop();
        assert!(
            !self.scope_stack.is_empty(),
            "close_scope popped off the last of the stack"
        );
    }
}

impl Visitor for ScopeVisitor {
    fn visit_assignment(&mut self, assignment: &ast::Assignment) {
        let mut expressions = assignment.expressions().iter();

        for var in assignment.variables() {
            let expression = expressions.next();
            if let Some(expression) = expression {
                self.read_expression(expression);
            }

            let name = match var {
                ast::Var::Expression(var_expr) => match var_expr.prefix() {
                    ast::Prefix::Expression(expression) => {
                        self.read_expression(expression);
                        continue;
                    }

                    ast::Prefix::Name(name) => {
                        if var_expr.suffixes().next().is_some() {
                            self.read_name(name);
                        }

                        name
                    }

                    _ => continue,
                },

                ast::Var::Name(name) => name,

                _ => continue,
            };

            self.write_name(name, expression.map(range));
            if let ast::Var::Name(_) = var {
                self.try_hoist();
            }
        }
    }

    fn visit_local_assignment(&mut self, local_assignment: &ast::LocalAssignment) {
        let mut expressions = local_assignment.expressions().iter();

        for name_token in local_assignment.names() {
            let expression = expressions.next();

            if let Some(expression) = expression {
                self.read_expression(expression);
            }

            self.define_name(name_token, range(local_assignment));

            if let Some(expression) = expression {
                self.write_name(name_token, Some(range(expression)));
            }
        }
    }

    fn visit_block(&mut self, block: &ast::Block) {
        if let Some((start, end)) = block.range() {
            if self
                .else_blocks
                .get(&(start.bytes(), end.bytes()))
                .is_some()
            {
                self.close_scope(); // close the if or elseif's block
                self.open_scope(block);
            }
        }
    }

    fn visit_block_end(&mut self, block: &ast::Block) {
        if let Some((start, end)) = block.range() {
            if self
                .else_blocks
                .get(&(start.bytes(), end.bytes()))
                .is_some()
            {
                self.close_scope();
            }
        }
    }

    fn visit_call(&mut self, call: &ast::Call) {
        let arguments = match call {
            ast::Call::AnonymousCall(args) => args,
            ast::Call::MethodCall(method_call) => method_call.args(),
            _ => return,
        };

        match arguments {
            ast::FunctionArgs::Parentheses { arguments, .. } => {
                for argument in arguments {
                    self.read_expression(argument);
                }
            }

            ast::FunctionArgs::TableConstructor(table_constructor) => {
                self.read_table_constructor(table_constructor);
            }

            _ => {}
        }
    }

    #[cfg(feature = "roblox")]
    fn visit_compound_assignment(&mut self, compound_assignment: &ast::types::CompoundAssignment) {
        self.read_var(compound_assignment.lhs());
        self.read_expression(compound_assignment.rhs());
    }

    fn visit_do(&mut self, do_block: &ast::Do) {
        self.open_scope(do_block);
    }

    fn visit_do_end(&mut self, _: &ast::Do) {
        self.close_scope();
    }

    fn visit_else_if(&mut self, else_if: &ast::ElseIf) {
        self.close_scope(); // close the if or other elseif blocks' scope
        self.read_expression(else_if.condition());
        self.open_scope(else_if);
    }

    fn visit_function_args(&mut self, args: &ast::FunctionArgs) {
        if let ast::FunctionArgs::Parentheses { arguments, .. } = args {
            for argument in arguments {
                self.read_expression(argument);
            }
        }
    }

    fn visit_function_body(&mut self, body: &ast::FunctionBody) {
        self.open_scope(body);

        self.current_scope().blocked.push(Cow::Borrowed("..."));

        for parameter in body.parameters() {
            if let ast::Parameter::Ellipse(token) | ast::Parameter::Name(token) = parameter {
                self.define_name(token, range(token));
            }
        }
    }

    fn visit_function_body_end(&mut self, _: &ast::FunctionBody) {
        self.close_scope();
    }

    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        self.read_prefix(call.prefix());
    }

    fn visit_function_declaration(&mut self, declaration: &ast::FunctionDeclaration) {
        let name = declaration.name();

        let mut names = name.names().iter();
        let base = names.next().unwrap();

        if names.next().is_some() {
            self.write_name(base, Some(range(declaration.name())));
        }

        self.read_name(base);

        if names.next().is_none() {
            self.try_hoist();
        }

        if let Some(name) = name.method_name() {
            self.open_scope(declaration.body());
            self.define_name_full_with_variable(
                "self",
                range(name),
                range(name),
                Variable {
                    is_self: true,
                    ..Variable::default()
                },
            );
        }
    }

    fn visit_function_declaration_end(&mut self, declaration: &ast::FunctionDeclaration) {
        if declaration.name().method_name().is_some() {
            self.close_scope();
        }
    }

    fn visit_generic_for(&mut self, generic_for: &ast::GenericFor) {
        for expression in generic_for.expressions().iter() {
            self.read_expression(expression);
        }

        self.open_scope(generic_for.block());

        for name in generic_for.names() {
            self.define_name(name, range(name));
            self.write_name(name, None);
        }
    }

    fn visit_generic_for_end(&mut self, _: &ast::GenericFor) {
        self.close_scope();
    }

    fn visit_index(&mut self, index: &ast::Index) {
        if let ast::Index::Brackets { expression, .. } = index {
            self.read_expression(expression);
        }
    }

    fn visit_if(&mut self, if_block: &ast::If) {
        self.read_expression(if_block.condition());
        self.open_scope(if_block.block());

        if let Some(else_block) = if_block.else_block() {
            if else_block.range().is_some() {
                self.else_blocks.insert(range(else_block));
            }
        }
    }

    fn visit_if_end(&mut self, if_block: &ast::If) {
        // else clean themselves up
        if if_block.else_block().is_none() {
            self.close_scope();
        }
    }

    fn visit_local_function(&mut self, local_function: &ast::LocalFunction) {
        self.define_name(local_function.name(), range(local_function.name()));
        self.open_scope(local_function.body());
    }

    fn visit_local_function_end(&mut self, _: &ast::LocalFunction) {
        self.close_scope();
    }

    fn visit_numeric_for(&mut self, numeric_for: &ast::NumericFor) {
        let variable_range = (
            numeric_for
                .index_variable()
                .start_position()
                .unwrap()
                .bytes(),
            numeric_for
                .start_end_comma()
                .start_position()
                .unwrap()
                .bytes(),
        );

        self.open_scope(numeric_for);
        self.define_name(numeric_for.index_variable(), variable_range);

        self.write_name(
            numeric_for.index_variable(),
            Some(range(numeric_for.start())),
        );

        self.read_expression(numeric_for.start());
        self.read_expression(numeric_for.end());

        if let Some(step) = numeric_for.step() {
            self.read_expression(step);
        }

        self.open_scope(numeric_for.block());
    }

    fn visit_numeric_for_end(&mut self, _: &ast::NumericFor) {
        self.close_scope();
        self.close_scope();
    }

    fn visit_repeat(&mut self, repeat: &ast::Repeat) {
        // Variables inside the read block are accessible in the until
        // So we read the entire statement, not just repeat.block()
        self.open_scope(repeat.block());
    }

    fn visit_repeat_end(&mut self, repeat: &ast::Repeat) {
        self.read_expression(repeat.until());
        self.close_scope();
    }

    fn visit_return(&mut self, return_stmt: &ast::Return) {
        for value in return_stmt.returns() {
            self.read_expression(value);
        }
    }

    fn visit_while(&mut self, while_loop: &ast::While) {
        self.read_expression(while_loop.condition());
        self.open_scope(while_loop.block());
    }

    fn visit_while_end(&mut self, _: &ast::While) {
        self.close_scope();
    }
}
