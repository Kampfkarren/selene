use mlua::UserData;

// Mostly copied from full-moon-lua-types
pub fn add_to_string_display<'lua, T: UserData>(
    name: &'static str,
    methods: &mut impl mlua::UserDataMethods<'lua, T>,
) {
    methods.add_meta_method(mlua::MetaMethod::ToString, move |_, this, _: ()| {
        Ok(format!("{name}({:x})", this as *const _ as usize))
    });
}

pub fn add_newindex_block<'lua, T: UserData>(
    name: &'static str,
    methods: &mut impl mlua::UserDataMethods<'lua, T>,
) {
    methods.add_meta_method(
        mlua::MetaMethod::NewIndex,
        move |_, _, (_, _): (String, mlua::Value)| -> mlua::Result<()> {
            Err(mlua::Error::RuntimeError(format!(
                "can't mutate {name} directly",
            )))
        },
    );
}
