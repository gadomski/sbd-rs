//! Parse and write Iridium Short Burst Data (SBD) messages.
//!
//! Iridium is both a [satellite constellation](https://en.wikipedia.org/wiki/Iridium_satellite_constellation)
//! and a [company](https://en.wikipedia.org/wiki/Iridium_Communications) that provides satellite
//! communications. The Iridium network is used by phones, modems, and other communication devices.
//!
//! One mode of transmitting data over the Iridium network is via Short Burst Data (SBD) messages.
//! These messages carry a payload of some small number of bytes, usually less than one thousand.
//! Messages can be Mobile-Originated (MO), meaning that they are sent *from* an Iridium modem, or
//! Mobile-Terminated (MT), meaning that the are sent *to* an Iridium modem. Mobile-originated
//! messages are delivered either to an email address via MIME attachment, or directly to a given
//! IP address and port via TCP; this second method is called DirectIP.
//!
//! This is a simple library for reading mobile originated SBD messages from a stream, decoding
//! their headers and data payloads, and writing them back to a stream. This library does not
//! handle mobile terminated messages.

mod information_element;
pub mod filesystem;
pub mod message;

pub use message::Message;

extern crate byteorder;
extern crate chrono;
#[macro_use]
extern crate enum_primitive;
extern crate num;
extern crate tempdir;

use std::result;

#[derive(Debug)]
pub enum Error {
    ByteorderError(byteorder::Error),
    IoError(std::io::Error),
    InvalidImei,
    InvalidProtocolRevisionNumber(u8),
    MissingMobileOriginatedHeader,
    MissingMobileOriginatedPayload,
    Oversized, // Oversized doesn't demand a size since we don't want to find out how much there really is
    Undersized(usize),
}

impl From<byteorder::Error> for Error {
    fn from(err: byteorder::Error) -> Error {
        Error::ByteorderError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IoError(err)
    }
}

pub type Result<T> = result::Result<T, Error>;
