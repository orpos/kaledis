//! Resource **edit**or for **p**ortable **e**xecutables.
//!
//! Supports:
//! * Parsing and introspection of portable executables
//! * Resource editing and icon replacement
//! * Resource transfer between files
//!
//! See [`Image`] for the main entry point for parsing, querying and updating a portable executable image.
//!
//! See [`ResourceDirectory`] for working with resource directories.
//!
//! # Examples
//!
//! ### Replacing the icon of an executable
//! ```
//! use editpe::Image;
//!
//! let mut image = Image::parse_file("damocles.exe")?;
//!
//! // get the resource directory
//! let mut resources = image.resource_directory().cloned().unwrap_or_default();
//! // set the icon file
//! resources.set_main_icon_file("sword.png")?;
//! // set the resource directory in the image
//! image.set_resource_directory(resources)?;
//!
//! // write an executable image with all changes applied
//! image.write_file("damocles.exe");
//! ```
//!
//! ### Transferring resources between executables
//! ```
//! use editpe::Image;
//!
//! let image = Image::parse_file("damocles.exe")?;
//! // get the resource directory from the source
//! let resources = image.resource_directory()?;
//!
//! let mut image = Image::parse_file("fortuna.exe")?;
//! // copy the resource directory to the target
//! image.set_resource_directory(resources)?;
//!
//! // write an executable image with all changes applied
//! image.write_file("fortuna.exe");
//! ```
//!
//! # Cargo features
//!
//! ### Default features
//!
//! - `std`: Enables standard library features, including reading and writing files.
//! - `images`: Enables support for converting and resizing images in other formats when setting icons. Also enables `std`.

#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg_hide))]
#![cfg_attr(docsrs, doc(cfg_hide(doc)))]

pub(crate) mod errors;
pub(crate) mod image;
pub(crate) mod resource;
pub(crate) mod util;

pub mod constants;
pub mod types;

pub use errors::*;
pub use image::*;
pub use resource::*;
