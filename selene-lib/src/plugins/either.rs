// Copied from full-moon-lua-types
use mlua::FromLua;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Either<A, B> {
    A(A),
    B(B),
}

pub fn take_either<'lua, A: FromLua<'lua>, B: FromLua<'lua>>(
    lua: &'lua mlua::Lua,
    lua_value: mlua::Value<'lua>,
    a_detail: &str,
    b_detail: &str,
) -> mlua::Result<Either<A, B>> {
    let type_name = lua_value.type_name();

    // Values are cheap to clone, they're mostly just references
    if let Ok(a) = A::from_lua(lua_value.clone(), lua) {
        return Ok(Either::A(a));
    }

    if let Ok(b) = B::from_lua(lua_value, lua) {
        return Ok(Either::B(b));
    }

    Err(mlua::Error::external(format!(
        "expected either {a_detail} or {b_detail}, received {}",
        type_name,
    )))
}
