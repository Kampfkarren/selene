use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use mlua::{MetaMethod, UserData};

use crate::ast_util::scopes::{Reference, ScopeManager, Variable};

use super::lua_methods::{add_newindex_block, add_to_string_display};

impl UserData for &ScopeManager {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        add_to_string_display("ScopeManager", methods);
        add_newindex_block("ScopeManager", methods);

        methods.add_method(
            "resolve_reference",
            |_, this, ReferenceWithId { reference, .. }: ReferenceWithId| match reference.resolved {
                Some(resolved_id) => {
                    Ok(this
                        .variables
                        .get(resolved_id)
                        .map(|variable| VariableWithId {
                            variable: variable.clone(),
                            id: Id(resolved_id),
                        }))
                }
                None => Ok(None),
            },
        );

        methods.add_method("reference_at_byte", |_, this, byte: usize| {
            Ok(this
                .reference_at_byte_with_id(byte)
                .map(|(id, reference)| ReferenceWithId {
                    id: Id(id),
                    reference: reference.clone(),
                }))
        });

        methods.add_method("variable_at_byte", |_, this, byte: usize| {
            Ok(this
                .variable_at_byte_with_id(byte)
                .map(|(id, variable)| VariableWithId {
                    id: Id(id),
                    variable: variable.clone(),
                }))
        })
    }
}

#[derive(Clone)]
struct ReferenceWithId {
    id: Id<Reference>,
    reference: Reference,
}

impl UserData for ReferenceWithId {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("id", |_, this| Ok(this.id));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        add_to_string_display("Reference", methods);
        add_newindex_block("Reference", methods);
    }
}

#[derive(Clone)]
struct VariableWithId {
    id: Id<Variable>,
    variable: Variable,
}

impl UserData for VariableWithId {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("id", |_, this| Ok(this.id));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        add_to_string_display("Variable", methods);
        add_newindex_block("Variable", methods);
    }
}

struct Id<T>(id_arena::Id<T>);

impl<T> Id<T> {
    fn add_tostring<'lua>(methods: &mut impl mlua::UserDataMethods<'lua, Self>)
    where
        Self: UserData,
    {
        methods.add_meta_method(MetaMethod::ToString, |_, Id(id), ()| {
            let mut hasher = DefaultHasher::new();
            id.hash(&mut hasher);

            Ok(format!(
                "Id({})#{}",
                std::any::type_name::<T>(),
                hasher.finish()
            ))
        });
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T> Copy for Id<T> {}

impl UserData for Id<Reference> {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        Self::add_tostring(methods);
        add_newindex_block("Id(Reference)", methods);
    }
}

impl UserData for Id<Variable> {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        Self::add_tostring(methods);
        add_newindex_block("Id(Variable)", methods);
    }
}
