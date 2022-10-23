use mlua::ToLua;

use crate::lints::{AstContext, Context};

pub struct Contexts<'lua> {
    scope_manager: mlua::AnyUserData<'lua>,
    standard_library: mlua::AnyUserData<'lua>,
}

impl<'lua> Contexts<'lua> {
    pub fn from_scope<'scope>(
        scope: &mlua::Scope<'lua, 'scope>,
        context: &'scope Context,
        ast_context: &'scope AstContext,
    ) -> Self {
        Self {
            scope_manager: scope
                .create_nonstatic_userdata(&ast_context.scope_manager)
                .expect("couldn't create scope manager"),

            standard_library: scope
                .create_nonstatic_userdata(&context.standard_library)
                .expect("couldn't create standard library"),
        }
    }
}

impl<'lua> ToLua<'lua> for Contexts<'lua> {
    fn to_lua(self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::Value<'lua>> {
        let table = lua.create_table()?;
        table.set("scope_manager", self.scope_manager)?;
        table.set("standard_library", self.standard_library)?;
        table.to_lua(lua)
    }
}
