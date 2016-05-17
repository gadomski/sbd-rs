//! Error module.

use std;
use std::ffi::OsString;
use std::fmt;

use byteorder;

/// Crate-specific errors
#[derive(Debug)]
pub enum Error {
    /// An error while reading bytes from a stream with the byteorder crate.
    Byteorder(byteorder::Error),
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
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Byteorder(ref err) => write!(f, "Byteorder error: {}", err),
            Error::Io(ref err) => write!(f, "IO error: {}", err),
            Error::InvalidImei => write!(f, "Invalid IMEI number"),
            Error::InvalidProtocolRevisionNumber(number) => {
                write!(f, "Invalid protocl revision number: {}", number)
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
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Byteorder(ref err) => err.description(),
            Error::Io(ref err) => err.description(),
            Error::InvalidImei => "invalid IMEI number",
            Error::InvalidProtocolRevisionNumber(_) => "invalid protocol revision number",
            Error::MissingMobileOriginatedHeader => "missing mobile originated header",
            Error::MissingMobileOriginatedPayload => "missing mobile originated payload",
            Error::NotADirectory(_) => "directory expected but it wasn't",
            Error::Oversized => "oversized message",
            Error::Undersized(_) => "undersized message",
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::Byteorder(ref err) => Some(err),
            Error::Io(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<byteorder::Error> for Error {
    fn from(err: byteorder::Error) -> Error {
        Error::Byteorder(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}
