use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex,
    },
};

use eyre::Context as EyreContext;
use full_moon_lua_types::{AnyNode, AstToLua};
use mlua::{FromLua, StdLib};
use once_cell::unsync::OnceCell;

use crate::{ast_util::purge_trivia, lints::*};

use super::{config::PluginConfig, context::Contexts, lockfile::Lockfile};

static LUA_PLUGIN_COUNT: AtomicU32 = AtomicU32::new(0);

thread_local! {
    static LUA: OnceCell<&'static mlua::Lua> = OnceCell::new();
}

fn plugin_registry_key(key: u32) -> String {
    format!("plugin_{key}")
}

pub struct LuaPlugin {
    plugin_contents: String,
    plugin_info: LuaPluginInfo,

    diagnostics: Arc<Mutex<Vec<Diagnostic>>>,
}

struct LuaPluginInfo {
    name: String,
    severity: Severity,

    registry_key: u32,
}

// Keeping this here so that when we do eventually support selene in different directories,
// we can avoid giving ourselves too much work.
fn root_dir() -> eyre::Result<PathBuf> {
    std::env::current_dir().wrap_err("could not get current directory")
}

enum PluginSource {
    FileSystem(PathBuf),
    PluginHub { output: PathBuf, source: String },
}

fn resolve_plugin_source(source: &Path, lockfile: &mut Lockfile) -> eyre::Result<PluginSource> {
    let source_plus_root = root_dir()?.join(source);
    if source_plus_root.exists() {
        return Ok(PluginSource::FileSystem(source_plus_root));
    }

    if source.starts_with("github.com") {
        return Ok(PluginSource::PluginHub {
            output: super::github::resolve_github_source(source, lockfile)?,
            source: source.to_string_lossy().to_string(),
        });
    }

    let Some(first_component) = source.components().next() else {
        return Err(eyre::eyre!("plugin source is empty"));
    };

    let mut caveat = "";

    let first_component_string = first_component.as_os_str().to_string_lossy();
    if first_component_string.contains('.') {
        caveat = "\nif this is a url, only github.com is supported";
    }

    Err(eyre::eyre!(
        "could not find plugin source `{}`{caveat}",
        source.display(),
    ))
}

pub fn load_plugins_from_config(plugin_config: &PluginConfig) -> eyre::Result<Vec<LuaPlugin>> {
    let mut plugins = Vec::new();

    let mut lockfile = Lockfile::open(&root_dir()?)?;

    let plugin_source = resolve_plugin_source(&plugin_config.source, &mut lockfile)?;

    match plugin_source {
        PluginSource::FileSystem(path) => {
            let plugin = LuaPlugin::new(plugin_config, &path)?;
            plugins.push(plugin);
        }

        PluginSource::PluginHub {
            output: path,
            source,
        } => {
            let lints_dir = path.join("lints");

            if !lints_dir.exists() {
                return Err(eyre::eyre!(
                    "plugin hub source `{source}` does not contain a `lints` directory (extracted to `{}`)",
                    path.display(),
                ));
            }

            for entry in std::fs::read_dir(lints_dir)? {
                let entry = entry?;
                let path = entry.path();

                let stem = match path.file_stem() {
                    Some(stem) => stem.to_string_lossy().to_string(),
                    None => continue,
                };

                if path.is_dir() || !stem.ends_with(".lint") {
                    continue;
                }

                plugins.push(LuaPlugin::new(plugin_config, &path)?);
            }
        }
    }

    lockfile.save().wrap_err("couldn't write to lockfile")?;

    Ok(plugins)
}

impl LuaPlugin {
    pub fn new(_plugin_config: &PluginConfig, resolved_source: &Path) -> eyre::Result<Self> {
        let plugin_contents = std::fs::read_to_string(resolved_source)?;

        LUA.with(|lua| {
            let lua = lua.get_or_try_init::<_, eyre::Error>(|| {
                let lua = mlua::Lua::new();
                lua.sandbox(true)?;
                lua.load_from_std_lib(StdLib::ALL_SAFE)?;
                Ok(lua.into_static())
            })?;

            Ok(LuaPlugin {
                plugin_info: lua.load(&plugin_contents).eval()?,
                diagnostics: Arc::new(Mutex::new(Vec::new())),

                plugin_contents,
            })
        })
    }

    pub fn name(&self) -> &str {
        &self.plugin_info.name
    }

    pub fn severity(&self) -> Severity {
        self.plugin_info.severity
    }

    // Needed because Lua is made once per thread
    fn init_lua(&self) -> mlua::Result<()> {
        LUA.with(|lua| {
            if lua.get().is_some() {
                return Ok(());
            }

            let lua = lua.get_or_try_init::<_, mlua::Error>(|| {
                let lua = mlua::Lua::new();
                lua.sandbox(true)?;
                lua.load_from_std_lib(StdLib::ALL_SAFE)?;
                Ok(lua.into_static())
            })?;

            let lua_value: mlua::Value = lua.load(&self.plugin_contents).eval()?;
            parse_plugin_info_with_registry_key(lua, lua_value, self.plugin_info.registry_key)?;

            Ok(())
        })
    }

    pub fn pass(
        &self,
        ast: Arc<Mutex<full_moon_lua_types::Ast>>,
        context: &Context,
        ast_context: &AstContext,
    ) -> mlua::Result<Vec<Diagnostic>> {
        if self.init_lua().is_err() {
            return Ok(Vec::new());
        }

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

            lua.globals().set(
                "purge_trivia",
                lua.create_function(move |lua, any_node: AnyNode| {
                    LuaPlugin::purge_trivia(lua, any_node)
                })?,
            )?;

            let pass: mlua::Function =
                lua.named_registry_value(&plugin_registry_key(self.plugin_info.registry_key))?;
            lua.scope(|scope| pass.call((ast, Contexts::from_scope(scope, context, ast_context))))?;

            Ok(())
        })?;

        let diagnostics = Arc::clone(&self.diagnostics);
        let mut lock = diagnostics.lock().unwrap();

        Ok(lock.drain(..).collect())
    }

    pub fn full_name(&self) -> String {
        format!("plugin_{}", self.name())
    }

    #[rustfmt::skip]
    fn purge_trivia(lua: &mlua::Lua, any_node: AnyNode) -> mlua::Result<mlua::Value> {
        match any_node {
            AnyNode::Ast(ast) => purge_trivia(ast.nodes()).ast_to_lua(lua),
            AnyNode::Assignment(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::BinOp(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::Block(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::Call(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::Do(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::ElseIf(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::Expression(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::Field(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::FunctionArgs(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::FunctionBody(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::FunctionCall(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::FunctionDeclaration(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::FunctionName(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::GenericFor(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::If(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::Index(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::LastStmt(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::LocalAssignment(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::LocalFunction(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::MethodCall(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::NumericFor(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::Parameter(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::Prefix(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::Repeat(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::Return(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::Stmt(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::Suffix(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::TableConstructor(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::UnOp(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::Value(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::Var(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::VarExpression(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::While(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::ContainedSpan(node) => purge_trivia(&node).ast_to_lua(lua),
            AnyNode::TokenReference(node) => purge_trivia(&node).ast_to_lua(lua),
        }
    }
}

// PLUGIN TODO: Make all these hand crafted errors
fn parse_plugin_info_with_registry_key(
    lua: &'static mlua::Lua,
    value: mlua::Value<'static>,
    registry_key: u32,
) -> mlua::Result<LuaPluginInfo> {
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

    lua.set_named_registry_value(&plugin_registry_key(registry_key), pass)?;

    Ok(LuaPluginInfo {
        name,
        severity,

        registry_key,
    })
}

impl mlua::FromLua<'static> for LuaPluginInfo {
    fn from_lua(value: mlua::Value<'static>, lua: &'static mlua::Lua) -> mlua::Result<Self> {
        parse_plugin_info_with_registry_key(
            lua,
            value,
            LUA_PLUGIN_COUNT.fetch_add(1, Ordering::SeqCst),
        )
    }
}
