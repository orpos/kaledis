use full_moon::LuaVersion;
use serde::{Deserialize, Serialize};

pub mod injector;
pub mod manifest;
pub mod modifiers;
pub mod polyfill;
pub mod transpile;
pub mod utils;

/// Represents lua versions that implement serde
#[non_exhaustive]
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TargetVersion {
    Lua51,
    Lua52,
    Lua53,
    Luau,
    Default,
}

impl Default for TargetVersion {
    fn default() -> Self {
        TargetVersion::Default
    }
}

impl TargetVersion {
    pub fn to_lua_version(&self) -> LuaVersion {
        match &self {
            TargetVersion::Lua51 => LuaVersion::lua51(),
            TargetVersion::Lua52 => LuaVersion::lua52(),
            TargetVersion::Lua53 => LuaVersion::lua53(),
            TargetVersion::Luau => LuaVersion::luau(),
            TargetVersion::Default => LuaVersion::default(),
        }
    }
}
