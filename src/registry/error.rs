use std::ffi::OsString;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("String conversion error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("RON ser/de error: {0}")]
    SpannedError(#[from] ron::error::SpannedError),
    #[error("Failed to convert OsString: {0:?}")]
    OsStringConversion(std::ffi::OsString),
}

impl From<OsString> for RegistryError {
    fn from(value: OsString) -> Self {
        Self::OsStringConversion(value)
    }
}
