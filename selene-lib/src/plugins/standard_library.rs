use mlua::{ToLua, UserData};

use crate::standard_library::*;

use super::{
    either::{take_either, Either},
    lua_methods::{add_newindex_block, add_to_string_display},
};

impl UserData for &StandardLibrary {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        add_to_string_display("StandardLibrary", methods);
        add_newindex_block("StandardLibrary", methods);

        methods.add_method(
            "find_global",
            |lua, this, name_or_parts: mlua::Value| match take_either::<String, Vec<String>>(
                lua,
                name_or_parts,
                "global name as string",
                "global name as a table of strings",
            )? {
                Either::A(name) => Ok(this
                    .find_global(&name.split('.').collect::<Vec<_>>())
                    .cloned()),

                Either::B(parts) => Ok(this.find_global(&parts).cloned()),
            },
        );
    }
}

fn yaml_to_lua<'lua>(
    lua: &'lua mlua::Lua,
    yaml: &serde_yaml::Value,
) -> mlua::Result<mlua::Value<'lua>> {
    Ok(match yaml {
        serde_yaml::Value::Null => mlua::Value::Nil,
        serde_yaml::Value::Bool(bool) => mlua::Value::Boolean(*bool),
        serde_yaml::Value::Number(number) => {
            mlua::Value::Number(number.as_f64().expect("couldn't convert number to f64"))
        }
        serde_yaml::Value::String(string) => mlua::Value::String(lua.create_string(string)?),
        serde_yaml::Value::Sequence(sequence) => lua
            .create_table_from(
                sequence
                    .iter()
                    .enumerate()
                    .map(|(index, value)| {
                        let value = yaml_to_lua(lua, value)?;
                        Ok((mlua::Value::Number(index as f64), value))
                    })
                    .collect::<mlua::Result<Vec<_>>>()?
                    .into_iter(),
            )?
            .to_lua(lua)?,
        serde_yaml::Value::Mapping(map) => lua
            .create_table_from(
                map.iter()
                    .map(|(key, value)| {
                        let key = yaml_to_lua(lua, key)?;
                        let value = yaml_to_lua(lua, value)?;
                        Ok((key, value))
                    })
                    .collect::<mlua::Result<Vec<_>>>()?
                    .into_iter(),
            )?
            .to_lua(lua)?,
    })
}

impl UserData for Field {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("extra", |lua, this| {
            let table = lua.create_table()?;

            for (key, value) in &this.extra {
                table.set(key.as_str(), yaml_to_lua(lua, value)?)?;
            }

            Ok(table)
        });
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        add_to_string_display("Field", methods);
        add_newindex_block("Field", methods);
    }
}
