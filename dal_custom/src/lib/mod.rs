use full_moon::LuaVersion;
use serde::{Deserialize, Serialize};

pub mod injector;
pub mod manifest;
pub mod modifiers;
pub mod polyfill;
pub mod transpiler;
pub mod utils;

/// Represents lua versions that implement serde
#[non_exhaustive]
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TargetVersion {
    Lua51,
    Lua52,
    Lua53,
	Luau,
    Default,
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
