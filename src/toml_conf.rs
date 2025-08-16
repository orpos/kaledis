use std::{
    fmt::Display,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use clap_serde_derive::{
    clap::{self},
    serde::Serialize,
    ClapSerde,
};
use schemars::JsonSchema;
use serde::Deserialize;

use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString};

#[derive(ClapSerde, Serialize, Deserialize, JsonSchema, Debug)]
pub struct Audio {
    /// Request and use microphone capabilities in Android
    #[default(false)]
    #[arg(
        short,
        long,
        help = "Request and use microphone capabilities in Android"
    )]
    pub mic: bool,
    /// Keep background music playing when opening LOVE (boolean, iOS and Android only)
    #[default(true)]
    #[arg(
        short,
        long,
        help = "Keep background music playing when opening LOVE (boolean, iOS and Android only) "
    )]
    pub mix_with_system: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, clap_serde_derive::clap::ValueEnum, JsonSchema)]
pub enum FullscreenType {
    Desktop,
    Exclusive,
}
impl Display for FullscreenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(ClapSerde, Serialize, Deserialize, JsonSchema, Debug)]
pub struct Window {
    /// The window title
    #[default("Untitled".to_string())]
    #[arg(short, long, help = "The window title")]
    pub title: String,
    /// Filepath to an image to use as the window's icon
    #[arg(
        short,
        long,
        help = "Filepath to an image to use as the window's icon."
    )]
    pub icon: Option<PathBuf>,
    /// The window width
    #[default(800)]
    #[arg(short, long, help = "The window width.")]
    pub width: u32,
    /// The window height
    #[default(600)]
    #[arg(short, long, help = "The window height.")]
    pub height: u32,
    /// Remove all border visuals from the window
    #[default(false)]
    #[arg(short, long, help = "Remove all border visuals from the window.")]
    pub borderless: bool,
    /// Let the window be user-resizable (boolean)
    #[default(false)]
    #[arg(short, long, help = "Let the window be user-resizable.")]
    pub resizable: bool,
    /// Minimum window width if the window is resizable
    #[default(1)]
    #[arg(short, long, help = "Minimum window width if the window is resizable.")]
    pub minwidth: u32,
    /// Minimum window height if the window is resizable
    #[default(1)]
    #[arg(short, long, help = "Minimum window height if the window is resizable")]
    pub minheight: u32,
    /// Enable fullscreen
    #[default(false)]
    #[arg(short, long, help = "Enable fullscreen")]
    pub fullscreen: bool,
    // Choose between "desktop" fullscreen or "exclusive" fullscreen mode
    #[arg(
        short,
        long,
        help = "Choose between \"desktop\" fullscreen or \"exclusive\" fullscreen mode"
    )]
    #[default(FullscreenType::Desktop)]
    pub fullscreentype: FullscreenType,
    /// Vertical sync mode
    #[default(1)]
    #[arg(short, long, help = "Vertical sync mode")]
    pub vsync: u32,
    /// The number of samples to use with multi-sampled antialiasing
    #[default(0)]
    #[arg(
        short,
        long,
        help = "The number of samples to use with multi-sampled antialiasing"
    )]
    pub msaa: u32,
    /// The number of bits per sample in the depth buffer
    #[arg(
        short,
        long,
        help = "The number of bits per sample in the depth buffer"
    )]
    pub depth: Option<u32>,
    /// The number of bits per sample in the stencil buffer
    #[arg(
        short,
        long,
        help = "The number of bits per sample in the stencil buffer"
    )]
    pub stencil: Option<u32>,
    /// Index of the monitor to show the window in
    #[default(1)]
    #[arg(short, long, help = "Index of the monitor to show the window in")]
    pub display: u8,
    /// Enable high-dpi mode for the window on a Retina display (boolean)
    #[default(false)]
    #[arg(
        short,
        long,
        help = "Enable high-dpi mode for the window on a Retina display (boolean)"
    )]
    pub highdpi: bool,
    /// Enable automatic DPI scaling when highdpi is set to true as well (boolean)
    #[default(true)]
    #[arg(
        short,
        long,
        help = "Enable automatic DPI scaling when highdpi is set to true as well (boolean)"
    )]
    pub usedpiscale: bool,
    /// The x-coordinate of the window's position in the specified display
    #[arg(
        short,
        long,
        help = "The x-coordinate of the window's position in the specified display"
    )]
    pub x: Option<u32>,
    /// The y-coordinate of the window's position in the specified display
    #[arg(
        short,
        long,
        help = "The y-coordinate of the window's position in the specified display"
    )]
    pub y: Option<u32>,
}

#[derive(
    EnumString,
    EnumIter,
    Debug,
    Deserialize,
    Serialize,
    Clone,
    clap_serde_derive::clap::ValueEnum,
    PartialEq,
    Eq,
    JsonSchema,
)]
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

#[derive(ClapSerde, Serialize, Deserialize, JsonSchema, Debug)]
pub struct Project {
    #[arg(short, long, help = "Enable detection algorithm.")]
    pub detect_modules: Option<bool>,

    #[arg(short, long, help = "Save location.")]
    pub identity: Option<PathBuf>,

    /// Name of the project
    #[arg(short, long, help = "Name of the project")]
    pub name: String,

    /// Where the Love2D executable is located
    #[arg(short, long, help = "Where the Love2D executable is located")]
    pub love_path: PathBuf,

    /// What version of Love2D to use
    #[default("11.5".to_string())]
    #[arg(short, long, help = "What version of Love2D to use")]
    pub version: String,

    /// Allows a custom configuration file to be used, that will later be merged with the TOML whenever building the project
    #[arg(
        short,
        long,
        help = "Allows a custom configuration file to be used, that will later be merged with the TOML whenever building the project"
    )]
    pub custom_conf: Option<PathBuf>,

    /// Whenever to attach a console (Windows only)
    #[default(true)]
    #[arg(
        short,
        long,
        help = "Whenever to attach a console when running the game"
    )]
    pub console: bool,

    /// Enable the accelerometer on iOS and Android by exposing it as a Joystick (boolean)
    #[default(true)]
    #[arg(
        short,
        help = "Enable the accelerometer on iOS and Android by exposing it as a Joystick "
    )]
    pub accelerometer_joystick: bool,

    /// If it's true, allows saving files (and read from the save directory) in external storage on Android
    #[default(false)]
    #[arg(
        short,
        long,
        help = "If it's true, allows saving files (and read from the save directory) in external storage on Android"
    )]
    pub external_storage: bool,

    /// Enable gamma-correct rendering, when supported by the system (boolean)
    #[default(false)]
    #[arg(
        short,
        long,
        help = "Enable gamma-correct rendering, when supported by the system (boolean)"
    )]
    pub gamma_correct: bool,

    // #[arg(short, long, help = "Define what dependencies the project uses (deprecated) please use pesde dependencies instead")]
    // pub dependencies: Vec<String>,
    #[arg(short, long, help = "Define what path the project uses for src")]
    pub src_path: Option<String>,

    #[arg(
        short,
        long,
        help = "Define what path the project uses for assets (this will be auto generated as a type)"
    )]
    pub asset_path: Option<String>,
}

#[derive(ClapSerde, Serialize, Deserialize, JsonSchema, Debug)]
pub struct Config {
    #[clap_serde]
    #[command(flatten)]
    pub project: Project,

    #[clap_serde]
    #[command(flatten)]
    pub window: Window,

    #[clap_serde]
    #[command(flatten)]
    pub audio: Audio,

    #[default(Modules::iter().collect())]
    pub modules: Vec<Modules>,

    #[default(vec![])]
    pub exclude_modules: Vec<Modules>,
}

impl Config {
    pub fn from_toml_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let data = read_to_string(path)?;
        Ok(toml::from_str(&data)?)
    }
}
