use std::{
    collections::HashMap,
    fmt::Display,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use clap_serde_derive::serde::Serialize;
use crate::dalbit::polyfill::{Polyfill, DEFAULT_INJECTION_PATH};
use schemars::JsonSchema;
use serde::Deserialize;

use strum_macros::{EnumIter, EnumString};
use url::Url;

macro_rules! default_create {
    ($type: expr, $value: expr) => {
        paste::paste! {
            pub fn [<fn_ $value>]() -> $type {
                $value
            }
        }
    };
    ($type: tt, $value: expr, $name : tt) => {
        pub fn $name() -> $type {
            $value
        }
    };
}

mod defaults {
    use strum::IntoEnumIterator;

    use crate::toml_conf::Modules;

    default_create!(bool, true);
    default_create!(bool, false);
    default_create!(u8, 1, u8_1);
    default_create!(u32, 1, u32_1);
    default_create!(u32, 0, u32_0);
    default_create!(u32, 800, u32_800);
    default_create!(u32, 600, u32_600);
    default_create!(String, "Untitled".to_string(), untitled);
    default_create!(String, "11.5".to_string(), love_version);
    pub fn modules() -> Vec<Modules> {
        Modules::iter().collect()
    }
    pub fn empty_modules() -> Vec<Modules> {
        vec![]
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct CustomPolyfillConfig {
    // The location of the custom polyfill. It can be a git repository or a local file path.
    pub location: Option<String>,
    pub configs: Option<HashMap<String, bool>>,
}

impl CustomPolyfillConfig {
    pub async fn polyfill(&self) -> anyhow::Result<Polyfill> {
        let mut pol = self.get_polyfill().await?;
        if let Some(configs) = &self.configs {
            pol.config = configs.clone();
        }
        Ok(pol)
    }
    async fn get_polyfill(&self) -> anyhow::Result<Polyfill> {
        if let Some(path) = &self.location {
            // Relative path
            if path.starts_with(".") {
                let abs_path = tokio::fs::canonicalize(path).await?;
                return Ok(Polyfill::new(
                    Url::from_directory_path(abs_path).unwrap(),
                    DEFAULT_INJECTION_PATH.into(),
                ));
            }
            return Ok(Polyfill::new(
                Url::parse(path)?,
                DEFAULT_INJECTION_PATH.into(),
            ));
        }
        Ok(Polyfill::default())
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct Audio {
    /// Request and use microphone capabilities in Android
    #[serde(default = "defaults::fn_false")]
    #[schemars(with = "Option<bool>")]
    pub mic: bool,
    #[serde(default = "defaults::fn_true")]
    /// Keep background music playing when opening LOVE (boolean, iOS and Android only)
    pub mix_with_system: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, JsonSchema, Default)]
pub enum FullscreenType {
    #[default]
    Desktop,
    Exclusive,
}
impl Display for FullscreenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct Window {
    /// The window title
    #[serde(default = "defaults::untitled")]
    pub title: String,
    /// Filepath to an image to use as the window's icon
    pub icon: Option<PathBuf>,
    /// The window width
    #[serde(default = "defaults::u32_800")]
    pub width: u32,
    /// The window height
    #[serde(default = "defaults::u32_600")]
    pub height: u32,
    /// Remove all border visuals from the window
    #[serde(default = "defaults::fn_false")]
    #[schemars(with = "Option<bool>")]
    pub borderless: bool,
    /// Let the window be user-resizable (boolean)
    #[serde(default = "defaults::fn_false")]
    #[schemars(with = "Option<bool>")]
    pub resizable: bool,
    /// Minimum window width if the window is resizable
    #[serde(default = "defaults::u32_1")]
    pub minwidth: u32,
    /// Minimum window height if the window is resizable
    #[serde(default = "defaults::u32_1")]
    pub minheight: u32,
    /// Enable fullscreen
    #[serde(default = "defaults::fn_false")]
    #[schemars(with = "Option<bool>")]
    pub fullscreen: bool,
    // Choose between "desktop" fullscreen or "exclusive" fullscreen mode
    #[serde(default)]
    #[schemars(with = "Option<FullscreenType>")]
    pub fullscreentype: FullscreenType,
    /// Vertical sync mode
    #[serde(default = "defaults::u32_1")]
    pub vsync: u32,
    /// The number of samples to use with multi-sampled antialiasing
    #[serde(default = "defaults::u32_0")]
    #[schemars(with = "Option<u32>")]
    pub msaa: u32,
    /// The number of bits per sample in the depth buffer
    pub depth: Option<u32>,
    /// The number of bits per sample in the stencil buffer
    pub stencil: Option<u32>,
    /// Index of the monitor to show the window in
    #[serde(default = "defaults::u8_1")]
    pub display: u8,
    /// Enable high-dpi mode for the window on a Retina display (boolean)
    #[serde(default = "defaults::fn_false")]
    #[schemars(with = "Option<bool>")]
    pub highdpi: bool,
    /// Enable automatic DPI scaling when highdpi is set to true as well (boolean)
    #[serde(default = "defaults::fn_true")]
    pub usedpiscale: bool,
    /// The x-coordinate of the window's position in the specified display
    pub x: Option<u32>,
    /// The y-coordinate of the window's position in the specified display
    pub y: Option<u32>,
}

#[derive(EnumString, EnumIter, Debug, Deserialize, Serialize, Clone, PartialEq, Eq, JsonSchema)]
pub enum Modules {
    /// Enable the audio module
    Audio,
    /// Enable the data module
    Data,
    /// Enable the event module
    Event,
    /// Enable the font module
    Font,
    /// Enable the graphics module
    Graphics,
    /// Enable the image module
    Image,
    /// Enable the joystick module
    Joystick,
    /// Enable the keyboard module
    Keyboard,
    /// Enable the math module
    Math,
    /// Enable the mouse module
    Mouse,
    /// Enable the physics module
    Physics,
    /// Enable the sound module
    Sound,
    /// Enable the system module
    System,
    /// Enable the thread module
    Thread,
    /// Enable the timer module, Disabling it will result 0 delta time in love.update
    Timer,
    /// Enable the touch module
    Touch,
    /// Enable the video module
    Video,
    /// Enable the window module
    Window,
}

impl Display for Modules {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct Project {
    /// Enable detection algorithm.
    pub detect_modules: Option<bool>,
    /// Disables automatic override of globals.d.luau."
    pub using_custom_globals: Option<bool>,
    /// Save location."
    pub identity: Option<PathBuf>,
    /// Name of the project"
    pub name: String,
    /// Where the Love2D executable is located
    pub love_path: PathBuf,
    /// What version of Love2D to use
    #[serde(default = "defaults::love_version")]
    /// What version of Love2D to use"
    pub version: String,

    /// Allows a custom configuration file to be used, that will later be merged with the TOML whenever building the project
    pub custom_conf: Option<PathBuf>,

    /// Whenever to attach a console (Windows only)
    #[serde(default = "defaults::fn_true")]
    pub console: bool,

    /// Enable the accelerometer on iOS and Android by exposing it as a Joystick (boolean)
    #[serde(default = "defaults::fn_true")]
    pub accelerometer_joystick: bool,

    /// If it's true, allows saving files (and read from the save directory) in external storage on Android
    #[serde(default = "defaults::fn_false")]
    #[schemars(with = "Option<bool>")]
    pub external_storage: bool,

    /// Enable gamma-correct rendering, when supported by the system (boolean)
    #[serde(default = "defaults::fn_false")]
    #[schemars(with = "Option<bool>")]
    pub gamma_correct: bool,

    /// Define what path the project uses for src
    pub src_path: Option<String>,

    /// Define what path the project uses for assets (this will be auto generated as a type)
    pub asset_path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct Config {
    pub project: Project,
    pub window: Window,
    pub audio: Audio,
    pub polyfill: Option<CustomPolyfillConfig>,
    #[serde(default = "defaults::modules")]
    pub modules: Vec<Modules>,
    #[serde(default = "defaults::empty_modules")]
    #[schemars(with = "Option<Vec<Modules>>")]
    pub exclude_modules: Vec<Modules>,
    #[serde(default = "defaults::fn_false")]
    #[schemars(with = "Option<bool>")]
    pub experimental_hmr: bool,
}

impl Config {
    pub fn from_toml_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let data = read_to_string(path)?;
        Ok(toml::from_str(&data)?)
    }
}
