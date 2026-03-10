//! Errors specific to reading, writing or modifying a PE image.

use core::str::Utf8Error;

use image::ImageError;
use std::io::Error as IOError;

/// Error that can occur when reading and parsing bytes.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ReadError(pub String);
impl From<&str> for ReadError {
    fn from(error: &str) -> Self {
        ReadError(error.to_string())
    }
}
impl From<String> for ReadError {
    fn from(error: String) -> Self {
        ReadError(error)
    }
}

/// Errors that can occur when reading a PE image.
#[derive(Debug, thiserror::Error)]
pub enum ImageReadError {
    #[error("invalid utf8: {0}")]
    Utf8Error(Utf8Error),
    #[error("invalid bytes: {0}")]
    InvalidBytes(ReadError),
    #[error("invalid header: {0}")]
    InvalidHeader(String),
    #[error("missing section: {0}")]
    MissingSection(String),
    #[error("invalid section: {0}")]
    InvalidSection(String),
    #[error("io error: {0}")]
    IOError(IOError),
}
impl From<Utf8Error> for ImageReadError {
    fn from(error: Utf8Error) -> Self {
        ImageReadError::Utf8Error(error)
    }
}
impl From<ReadError> for ImageReadError {
    fn from(error: ReadError) -> Self {
        ImageReadError::InvalidBytes(error)
    }
}
impl From<IOError> for ImageReadError {
    fn from(error: IOError) -> Self {
        ImageReadError::IOError(error)
    }
}

/// Errors that can occur when writing a PE image.
#[derive(Debug, thiserror::Error)]
pub enum ImageWriteError {
    #[error("not enough space in file header")]
    NotEnoughSpaceInHeader,
    #[error("section points outside image: {0} > {1}")]
    InvalidSectionRange(u64, u64),
    #[error("io error: {0}")]
    IOError(IOError),
}
impl From<IOError> for ImageWriteError {
    fn from(error: IOError) -> Self {
        ImageWriteError::IOError(error)
    }
}

/// Errors that can occur when modifying resource data.
#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
    #[error("invalid table: {0}")]
    InvalidTable(String),
    #[error("invalid bytes: {0}")]
    InvalidBytes(ReadError),
    #[error("invalid icon: {0}")]
    InvalidIconResource(ImageError),
    #[error("io error: {0}")]
    IOError(IOError),
}
impl From<ReadError> for ResourceError {
    fn from(error: ReadError) -> Self {
        ResourceError::InvalidBytes(error)
    }
}

impl From<ImageError> for ResourceError {
    fn from(error: ImageError) -> Self {
        ResourceError::InvalidIconResource(error)
    }
}

impl From<IOError> for ResourceError {
    fn from(error: IOError) -> Self {
        ResourceError::IOError(error)
    }
}
