//! Error module.

use information_element::{InformationElement, InformationElementType};
use std;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fmt;
use walkdir;

/// Crate-specific errors
#[derive(Debug)]
pub enum Error {
    /// A wrapper around a `std::io::Error`.
    Io(std::io::Error),
    /// Invalid IMEI number.
    InvalidImei,
    /// Invalid protocol revision number.
    InvalidProtocolRevisionNumber(u8),
    /// Missing mobile originated header.
    MissingMobileOriginatedHeader,
    /// Missing mobile originated payload.
    MissingMobileOriginatedPayload,
    /// We expected a directory, but this isn't one.
    NotADirectory(OsString),
    /// An oversized message.
    ///
    /// Oversized doesn't demand a size since we don't want to find out how much there really is.
    Oversized,
    /// An undersized message.
    Undersized(usize),
    /// Some information elements weren't handled during reading.
    ///
    /// This is bad, because we might not write those IEs back out.
    UnhandledInformationElements(HashMap<InformationElementType, InformationElement>),
    /// Wrapper around `std::str::Utf8Error`.
    Utf8(std::str::Utf8Error),
    /// Wrapper around `walkdir::Error`.
    WalkDir(walkdir::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "IO error: {}", err),
            Error::InvalidImei => write!(f, "Invalid IMEI number"),
            Error::InvalidProtocolRevisionNumber(number) => {
                write!(f, "Invalid protocol revision number: {}", number)
            }
            Error::MissingMobileOriginatedHeader => write!(f, "Missing mobile origianted header"),
            Error::MissingMobileOriginatedPayload => write!(f, "Missing mobile orignated payload"),
            Error::NotADirectory(ref path) => {
                write!(f,
                       "Not a directory: {}",
                       path.clone().into_string().unwrap_or("<undisplayable path>".to_string()))
            }
            Error::Oversized => write!(f, "Oversized message"),
            Error::Undersized(size) => write!(f, "Undersized message: {}", size),
            Error::UnhandledInformationElements(ref ies) => {
                write!(f, "Unhandled information elements: {:?}", ies)
            }
            Error::Utf8(ref err) => write!(f, "Utf8 error: {}", err),
            Error::WalkDir(ref err) => write!(f, "WalkDir error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
            Error::InvalidImei => "invalid IMEI number",
            Error::InvalidProtocolRevisionNumber(_) => "invalid protocol revision number",
            Error::MissingMobileOriginatedHeader => "missing mobile originated header",
            Error::MissingMobileOriginatedPayload => "missing mobile originated payload",
            Error::NotADirectory(_) => "directory expected but it wasn't",
            Error::Oversized => "oversized message",
            Error::Undersized(_) => "undersized message",
            Error::UnhandledInformationElements(_) => "unhandled information elements",
            Error::Utf8(ref err) => err.description(),
            Error::WalkDir(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::Utf8(ref err) => Some(err),
            Error::WalkDir(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<walkdir::Error> for Error {
    fn from(err: walkdir::Error) -> Error {
        Error::WalkDir(err)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Error {
        Error::Utf8(err)
    }
}
