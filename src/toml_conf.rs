use std::{
    collections::HashMap,
    fmt::Display,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use crate::dalbit::polyfill::{DEFAULT_INJECTION_PATH, Polyfill};
use clap_serde_derive::serde::Serialize;
use schemars::JsonSchema;
use serde::Deserialize;

use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString};
use url::Url;

#[derive(EnumString, EnumIter, Debug, Deserialize, Serialize, Clone, PartialEq, Eq, JsonSchema)]
pub enum Orientation {
    Portrait,
    Landscape,
}

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
    use crate::toml_conf::Orientation;

    default_create!(bool, true);
    default_create!(bool, false);
    default_create!(u8, 1, u8_1);
    default_create!(u32, 1, u32_1);
    default_create!(u32, 0, u32_0);
    default_create!(u32, 800, u32_800);
    default_create!(u32, 600, u32_600);
    default_create!(String, "Untitled".to_string(), untitled);
    // default_create!(String, "11.5".to_string(), love_version);
    pub fn default_orientation() -> Orientation {
        Orientation::Landscape
    }
    pub fn modules() -> Vec<Modules> {
        Modules::iter().collect()
    }
    // pub fn empty_modules() -> Vec<Modules> {
    //     vec![]
    // }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct CustomPolyfillConfig {
    // The location of the custom polyfill. It can be a git repository or a local file path.
    pub location: Option<String>,
    pub configs: Option<HashMap<String, bool>>,
}

impl CustomPolyfillConfig {
    pub async fn polyfill(&self) -> color_eyre::Result<Polyfill> {
        let mut pol = self.get_polyfill().await?;
        if let Some(configs) = &self.configs {
            pol.config = configs.clone();
        }
        Ok(pol)
    }
    async fn get_polyfill(&self) -> color_eyre::Result<Polyfill> {
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

#[derive(
    EnumString, EnumIter, Debug, Deserialize, Serialize, Clone, PartialEq, Eq, JsonSchema, Hash,
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

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct Project {
    /// Disables automatic override of globals.d.luau."
    pub using_custom_globals: Option<bool>,
    /// Save location."
    pub identity: Option<PathBuf>,
    /// Name of the project"
    pub name: String,

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
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct LayoutConfig {
    #[serde(default)]
    pub bundle: Vec<String>,
    #[serde(default)]
    pub external: Vec<String>,
    #[serde(default)]
    pub code: String,
}

//
//  conf.toml config files
//
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct LoveConfig {
    pub project: Project,
    pub window: Window,
    pub audio: Audio,
    #[serde(default = "defaults::modules")]
    pub modules: Vec<Modules>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]

pub struct MacosConfig {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct AndroidConfig {
    pub version_code: u32,
    pub game_id: String,
    // If not provided we will use the version
    pub version_name: Option<String>,
    // If not provided we will use the project_name
    pub game_name: Option<String>,
    #[serde(default = "defaults::default_orientation")]
    pub orientation: Orientation,
    #[serde(default)]
    pub uses_microphone: bool,
    #[serde(default)]
    pub touchscreen: bool,
    #[serde(default)]
    pub bluetooth: bool,
    #[serde(default)]
    pub gamepad: bool,
    #[serde(default)]
    pub usb_host: bool,
    #[serde(default)]
    pub external_mouse_input: bool,
    #[serde(default)]
    pub audio_pro: bool,
    #[serde(default)]
    pub audio_low_latency: bool,
}

impl AndroidConfig {
    pub fn to_string(&self, project_name: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<manifest package="{}"
        android:versionCode="{}"
        android:versionName="{}"
        android:installLocation="auto"
		xmlns:android="http://schemas.android.com/apk/res/android">
    <uses-permission android:name="android.permission.INTERNET" />
    <uses-permission android:name="android.permission.VIBRATE" />
    <uses-permission android:name="android.permission.BLUETOOTH" />
    <uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" android:maxSdkVersion="18" />
    {}

    <!-- OpenGL ES 2.0 -->
    <uses-feature android:glEsVersion="0x00020000" />
    <!-- Touchscreen support -->
    <uses-feature android:name="android.hardware.touchscreen" android:required="{}" />
    <!-- Game controller support -->
    <uses-feature android:name="android.hardware.bluetooth" android:required="{}" />
    <uses-feature android:name="android.hardware.gamepad" android:required="{}" />
    <uses-feature android:name="android.hardware.usb.host" android:required="{}" />
    <!-- External mouse input events -->
    <uses-feature android:name="android.hardware.type.pc" android:required="{}" />
    <!-- Low latency audio -->
    <uses-feature android:name="android.hardware.audio.low_latency" android:required="{}" />
    <uses-feature android:name="android.hardware.audio.pro" android:required="{}" />

    <application
            android:allowBackup="true"
            android:icon="@drawable/love"
            android:label="{}"
            android:usesCleartextTraffic="true" >
        <activity
                android:name="org.love2d.android.GameActivity"
                android:exported="true"
                android:configChanges="orientation|screenSize|smallestScreenSize|screenLayout|keyboard|keyboardHidden|navigation"
                android:label="{}"
                android:launchMode="singleInstance"
                android:screenOrientation="{}"
                android:resizeableActivity="false"
                android:theme="@android:style/Theme.NoTitleBar.Fullscreen" >
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
                <category android:name="tv.ouya.intent.category.GAME" />
            </intent-filter>
            <intent-filter>
                <action android:name="android.hardware.usb.action.USB_DEVICE_ATTACHED" />
            </intent-filter>
        </activity>
    </application>
</manifest>
        "#,
            self.game_id,
            &self.version_code,
            self.version_name
                .as_ref()
                .unwrap_or(&self.version_code.to_string()),
            if self.uses_microphone {
                "<uses-permission android:name=\"android.permission.RECORD_AUDIO\" />"
            } else {
                ""
            },
            self.touchscreen,
            self.bluetooth,
            self.gamepad,
            self.usb_host,
            self.external_mouse_input,
            self.audio_low_latency,
            self.audio_pro,
            self.game_name.as_ref().unwrap_or(&project_name.to_owned()),
            self.game_name.as_ref().unwrap_or(&project_name.to_owned()),
            if let Orientation::Landscape = self.orientation {
                "landscape"
            } else {
                "portrait"
            }
        )
    }
}

//
//  kaledis.toml config files
//
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct KaledisConfig {
    pub project_name: String,
    // If not provided we will use the project_name
    pub android: Option<AndroidConfig>,
    pub mac: Option<MacosConfig>,
    pub custom_android_manifest: Option<String>,
    pub icon: Option<String>,
    pub polyfill: Option<CustomPolyfillConfig>,
    pub layout: LayoutConfig,
    #[serde(default = "defaults::fn_false")]
    #[schemars(with = "Option<bool>")]
    pub detect_modules: bool,
    #[serde(default = "defaults::fn_true")]
    #[schemars(with = "Option<bool>")]
    pub hmr: bool,
    // for custom ports you have to provide a folder for each platform
    pub love: String,
}

impl KaledisConfig {
    pub fn from_toml_file<P: AsRef<Path>>(path: P) -> color_eyre::Result<Self> {
        let data = read_to_string(path)?;
        Ok(toml::from_str(&data)?)
    }
}

fn generate_module_string(imported_modules: Vec<Modules>) -> String {
    let mut output = String::new();
    for module in Modules::iter() {
        output += &format!(
            "t.modules.{}={}\n\t",
            &module.to_string().to_lowercase(),
            imported_modules.contains(&module)
        );
    }
    output
}
fn format_option<T: ToString>(value: Option<T>) -> String {
    value.map(|x| x.to_string()).unwrap_or("nil".to_string())
}
impl LoveConfig {
    pub fn from_toml_file<P: AsRef<Path>>(path: P) -> color_eyre::Result<Self> {
        let data = read_to_string(path)?;
        Ok(toml::from_str(&data)?)
    }
    pub fn to_string(&self, used_modules: Vec<Modules>) -> String {
        format!(
            r#"function love.conf(t)
    t.identity = {}
    t.appendidentity = {}
    t.console = {}
    t.accelerometerjoystick = {}
    t.externalstorage = {}
    t.gammacorrect = {}

    t.audio.mic = {}
    t.audio.mixwithsystem = {}

    t.window.title = {:?}
    t.window.icon = {}
    t.window.width = {}
    t.window.height = {}
    t.window.borderless = {}
    t.window.resizable = {}
    t.window.minwidth = {}
    t.window.minheight = {}
    t.window.fullscreen = {}
    t.window.fullscreentype = {}
    t.window.vsync = {}
    t.window.msaa = {}
    t.window.depth = {}
    t.window.stencil = {}
    t.window.display = {}
    t.window.highdpi = {}
    t.window.usedpiscale = {}
    t.window.x = {}
    t.window.y = {}
    {}
end"#,
            format_option(
                self.project
                    .identity
                    .as_ref()
                    .map(|x| x.to_string_lossy().to_string())
            ),
            "false",
            self.project.console,
            self.project.accelerometer_joystick,
            self.project.external_storage,
            self.project.gamma_correct,
            self.audio.mic,
            self.audio.mix_with_system,
            self.window.title,
            format_option(
                self.window
                    .icon
                    .as_ref()
                    .map(|x| x.to_string_lossy().to_string())
            ),
            self.window.width,
            self.window.height,
            self.window.borderless,
            self.window.resizable,
            self.window.minwidth,
            self.window.minheight,
            self.window.fullscreen,
            match self.window.fullscreentype {
                crate::toml_conf::FullscreenType::Desktop => "\"desktop\"",
                crate::toml_conf::FullscreenType::Exclusive => "\"exclusive\"",
            },
            self.window.vsync,
            self.window.msaa,
            format_option(self.window.depth),
            format_option(self.window.stencil),
            self.window.display,
            self.window.highdpi,
            self.window.usedpiscale,
            format_option(self.window.x),
            format_option(self.window.y),
            generate_module_string(used_modules)
        )
    }
}
