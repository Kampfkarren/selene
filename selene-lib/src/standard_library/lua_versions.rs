use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LuaVersion {
    Lua51,
    Lua52,
    Lua53,
    Lua54,
    Luau,
    LuaJIT,

    Unknown(String),
}

impl LuaVersion {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Lua51 => "lua51",
            Self::Lua52 => "lua52",
            Self::Lua53 => "lua53",
            Self::Lua54 => "lua54",
            Self::Luau => "luau",
            Self::LuaJIT => "luajit",
            Self::Unknown(value) => value,
        }
    }

    pub fn to_lua_version(&self) -> Result<full_moon::ast::LuaVersion, LuaVersionError> {
        match self {
            Self::Lua51 => Ok(full_moon::ast::LuaVersion::lua51()),

            #[cfg(feature = "lua52")]
            Self::Lua52 => Ok(full_moon::ast::LuaVersion::lua52()),

            #[cfg(feature = "lua53")]
            Self::Lua53 => Ok(full_moon::ast::LuaVersion::lua53()),

            #[cfg(feature = "lua54")]
            Self::Lua54 => Ok(full_moon::ast::LuaVersion::lua54()),

            #[cfg(feature = "roblox")]
            Self::Luau => Ok(full_moon::ast::LuaVersion::luau()),

            #[cfg(feature = "luajit")]
            Self::LuaJIT => Ok(full_moon::ast::LuaVersion::luajit()),

            Self::Unknown(value) => Err(LuaVersionError::Unknown(value)),

            #[allow(unreachable_patterns)]
            _ => Err(LuaVersionError::FeatureNotEnabled(self.to_str())),
        }
    }
}

impl FromStr for LuaVersion {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "lua51" => Ok(Self::Lua51),
            "lua52" => Ok(Self::Lua52),
            "lua53" => Ok(Self::Lua53),
            "lua54" => Ok(Self::Lua54),
            "luau" => Ok(Self::Luau),
            "luajit" => Ok(Self::LuaJIT),
            _ => Err(()),
        }
    }
}

pub enum LuaVersionError<'a> {
    FeatureNotEnabled(&'a str),
    Unknown(&'a str),
}

impl Serialize for LuaVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_str())
    }
}

impl<'de> Deserialize<'de> for LuaVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if let Ok(version) = Self::from_str(&value) {
            Ok(version)
        } else {
            Ok(Self::Unknown(value))
        }
    }
}
