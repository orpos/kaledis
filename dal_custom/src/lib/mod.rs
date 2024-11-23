use full_moon::LuaVersion;
use serde::{Deserialize, Serialize};

pub mod manifest;
pub mod modifiers;
pub mod polyfill;
pub mod transpiler;

#[non_exhaustive]
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TargetVersion {
    Lua51,
    Lua52,
    Lua53,
    Default,
}

impl TargetVersion {
    pub fn to_lua_version(&self) -> LuaVersion {
        match &self {
            TargetVersion::Lua51 => LuaVersion::lua51(),
            TargetVersion::Lua52 => LuaVersion::lua52(),
            TargetVersion::Lua53 => LuaVersion::lua53(),
            TargetVersion::Default => LuaVersion::default(),
        }
    }
}
