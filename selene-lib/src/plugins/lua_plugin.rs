use std::{
    cell::RefCell,
    error::Error,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex, RwLock,
    },
};

use mlua::StdLib;
use once_cell::unsync::OnceCell;

use crate::rules::*;

use super::config::PluginConfig;

static LUA_PLUGIN_COUNT: AtomicU32 = AtomicU32::new(0);

// static LUA: OnceCell<Arc<Mutex<mlua::Lua>>> = OnceCell::new();
thread_local! {
    static LUA: OnceCell<&'static mlua::Lua> = OnceCell::new();
}

pub struct LuaPlugin {
    pub name: String,
    pub severity: Severity,

    registry_key: String,
    diagnostics: Arc<Mutex<Vec<Diagnostic>>>,
}

impl LuaPlugin {
    // PLUGIN TODO: Make this eyre and kill the Box<dyn Error>
    pub fn new(plugin_config: &PluginConfig) -> Result<Self, Box<dyn Error>> {
        LUA.with(|lua| {
            let lua = lua.get_or_try_init::<_, Box<dyn Error>>(|| {
                let lua = mlua::Lua::new();
                lua.sandbox(true)?;
                lua.load_from_std_lib(StdLib::ALL_SAFE)?;
                Ok(lua.into_static())
            })?;

            let plugin_contents = std::fs::read_to_string(&plugin_config.source)?;

            Ok(lua.load(&plugin_contents).eval()?)
        })
    }

    pub fn pass(
        &self,
        ast: Arc<Mutex<full_moon_lua_types::Ast>>,
        context: &Context,
        ast_context: &AstContext,
    ) -> mlua::Result<Vec<Diagnostic>> {
        let diagnostics = Arc::clone(&self.diagnostics);

        LUA.with(|lua| -> mlua::Result<()> {
            let lua = lua.get().expect("Lua not initialized");

            // PLUGIN TODO: i don't like that this happens regardless of if lint is actually called.
            // Could fix by collating the diagnostics at the end (makes it easier to create more complete lint calls)
            let full_name = self.full_name();

            lua.globals().set(
                "lint",
                lua.create_function_mut(move |_, (message, has_range): (String, mlua::Value)| {
                    let has_range_userdata = match &has_range {
                        mlua::Value::UserData(userdata) => userdata,
                        _ => todo!("non userdata passed to lint"),
                    };

                    let metatable = has_range_userdata
                        .get_metatable()
                        .expect("NYI: get_metatable() fail");

                    let index = metatable
                        .get::<_, mlua::Function>(mlua::MetaMethod::Index)
                        .expect("NYI: get __index fail");

                    let range_function: mlua::Function = index
                        .call((has_range_userdata.clone(), "range"))
                        .expect("NYI: handle __index fail or range doesn't exist");

                    let range: (u32, u32) = range_function
                        .call(has_range)
                        .expect("NYI: handle range call fail");

                    let mut diagnostics = diagnostics.lock().unwrap();

                    diagnostics.push(Diagnostic::new(
                        full_name.clone(),
                        message,
                        Label::new(range),
                    ));

                    Ok(())
                })?,
            )?;

            let pass: mlua::Function<'static> = lua.named_registry_value(&self.registry_key)?;
            pass.call(ast)?;

            Ok(())
        })?;

        let diagnostics = Arc::clone(&self.diagnostics);
        let mut lock = diagnostics.lock().unwrap();

        Ok(lock.drain(..).collect())
    }

    pub fn full_name(&self) -> String {
        format!("plugin_{}", self.name)
    }
}

impl mlua::FromLua<'static> for LuaPlugin {
    // PLUGIN TODO: Make all these hand crafted errors
    fn from_lua(value: mlua::Value<'static>, lua: &'static mlua::Lua) -> mlua::Result<Self> {
        let table = mlua::Table::from_lua(value, lua)?;

        let name = table.get("name")?;
        let pass = table.get::<_, mlua::Function<'static>>("pass")?;

        let severity = match table.get::<_, String>("severity")?.as_str() {
            "allow" => Severity::Allow,
            "error" => Severity::Error,
            "warning" => Severity::Warning,
            other => {
                return Err(mlua::Error::external(format!(
                    r#"invalid severity "{other}". must be "allow", "error", or "warning"#
                )))
            }
        };

        let registry_key = format!("plugin_{}", LUA_PLUGIN_COUNT.fetch_add(1, Ordering::SeqCst));

        lua.set_named_registry_value(&registry_key, pass)?;

        Ok(Self {
            name,
            severity,

            registry_key,

            diagnostics: Arc::new(Mutex::new(Vec::new())),
        })
    }
}
