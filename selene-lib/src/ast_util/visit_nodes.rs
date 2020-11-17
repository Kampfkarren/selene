use full_moon::{
    ast::*,
    node::Node,
    tokenizer::{Token, TokenReference},
    visitors::Visitor,
};

#[cfg(feature = "roblox")]
use full_moon::ast::types::*;

pub trait NodeVisitor<'ast> {
    fn visit_node<'a>(&mut self, node: &'a dyn Node<'ast>, visitor_type: VisitorType);

    fn visit_nodes<'a>(&mut self, ast: &'a Ast<'ast>)
    where
        Self: Sized,
    {
        let mut node_visitor = NodeVisitorLogic { callback: self };

        node_visitor.visit_ast(ast);
    }
}

struct NodeVisitorLogic<'a, 'ast> {
    callback: &'a mut dyn NodeVisitor<'ast>,
}

macro_rules! make_node_visitor {
    ({
        $($visitor:ident($struct:ident),)+

        $(#[$meta:meta] {
            $($meta_visitor:ident($meta_ast_type:ident),)+
        })+
    }, {
        $($token_visitor:ident,)+
    }) => {
        paste::paste! {
            impl<'ast> Visitor<'ast> for NodeVisitorLogic<'_, 'ast> {
                $(
                    fn $visitor(&mut self, node: &$struct<'ast>) {
                        self.callback.visit_node(node, VisitorType::[<$visitor:camel>]);
                    }
                )+

                $(
                    fn $token_visitor(&mut self, node: &Token<'ast>) {
                        self.callback.visit_node(node, VisitorType::[<$token_visitor:camel>]);
                    }
                )+

                $(
                    $(
                        #[$meta]
                        fn $meta_visitor(&mut self, node: &$meta_ast_type<'ast>) {
                            self.callback.visit_node(node, VisitorType::[<$meta_visitor:camel>]);
                        }
                    )+
                )+
            }

            #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
            pub enum VisitorType {
                $(
                    [<$visitor:camel>],
                )+

                $(
                    [<$token_visitor:camel>],
                )+

                $(
                    $(
                        #[$meta]
                        [<$meta_visitor:camel>],
                    )+
                )+
            }
        }
    };
}

make_node_visitor!({
    visit_anonymous_call(FunctionArgs),
    visit_assignment(Assignment),
    visit_bin_op(BinOpRhs),
    visit_block(Block),
    visit_call(Call),
    visit_do(Do),
    visit_else_if(ElseIf),
    visit_eof(TokenReference),
    visit_expression(Expression),
    visit_field(Field),
    visit_function_args(FunctionArgs),
    visit_function_body(FunctionBody),
    visit_function_call(FunctionCall),
    visit_function_declaration(FunctionDeclaration),
    visit_function_name(FunctionName),
    visit_generic_for(GenericFor),
    visit_if(If),
    visit_index(Index),
    visit_local_assignment(LocalAssignment),
    visit_local_function(LocalFunction),
    visit_last_stmt(LastStmt),
    visit_method_call(MethodCall),
    visit_numeric_for(NumericFor),
    visit_parameter(Parameter),
    visit_prefix(Prefix),
    visit_return(Return),
    visit_repeat(Repeat),
    visit_stmt(Stmt),
    visit_suffix(Suffix),
    visit_table_constructor(TableConstructor),
    visit_un_op(UnOp),
    visit_value(Value),
    visit_var(Var),
    visit_var_expression(VarExpression),
    visit_while(While),

    #[cfg(feature = "roblox")] {
        visit_as_assertion(AsAssertion),
        visit_compound_assignment(CompoundAssignment),
        visit_compound_op(CompoundOp),
        visit_exported_type_declaration(ExportedTypeDeclaration),
        visit_generic_declaration(GenericDeclaration),
        visit_indexed_type_info(IndexedTypeInfo),
        visit_type_declaration(TypeDeclaration),
        visit_type_field(TypeField),
        visit_type_field_key(TypeFieldKey),
        visit_type_info(TypeInfo),
        visit_type_specifier(TypeSpecifier),
    }
}, {
    visit_identifier,
    visit_multi_line_comment,
    visit_number,
    visit_single_line_comment,
    visit_string_literal,
    visit_symbol,
    visit_token,
    visit_whitespace,
});

#[cfg(test)]
mod tests {
    use super::*;
    use full_moon::parse;
    use std::cmp::{max, min};

    #[test]
    fn test_visit_nodes() {
        let code = "local foo = 1";
        let ast = parse(code).unwrap();

        struct TestVisitor {
            smallest_range: usize,
            largest_range: usize,
            visited_something_called_foo: bool,
        }

        impl NodeVisitor<'_> for TestVisitor {
            fn visit_node<'a>(&mut self, node: &'a dyn Node<'_>, _visitor_type: VisitorType) {
                self.smallest_range =
                    min(self.smallest_range, node.start_position().unwrap().bytes());
                self.largest_range = max(self.largest_range, node.end_position().unwrap().bytes());

                if node
                    .tokens()
                    .into_iter()
                    .fold(String::new(), |total, token| {
                        format!("{}{}", total, token.to_string())
                    })
                    .trim()
                    == "foo".to_owned()
                {
                    self.visited_something_called_foo = true;
                }
            }
        }

        let mut visitor = TestVisitor {
            smallest_range: 100,
            largest_range: 0,
            visited_something_called_foo: false,
        };

        visitor.visit_nodes(&ast);

        assert!(visitor.visited_something_called_foo);
        assert_eq!(visitor.smallest_range, 0);
        assert_eq!(visitor.largest_range, code.len());
    }
}
