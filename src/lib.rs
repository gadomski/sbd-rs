//! Parse and write Iridium Short Burst Data (SBD) messages.
//!
//! # Background
//!
//! Iridium is both a
//! [satellite constellation](https://en.wikipedia.org/wiki/Iridium_satellite_constellation)
//! and a [company](https://en.wikipedia.org/wiki/Iridium_Communications) that provides satellite
//! communications. The Iridium network is used by phones, modems, and other communication devices.
//!
//! One mode of transmitting data over the Iridium network is via Short Burst Data (SBD) messages.
//! These messages carry a payload of some small number of bytes, usually less than one thousand.
//! Messages can be Mobile-Originated (MO), meaning that they are sent *from* an Iridium modem, or
//! Mobile-Terminated (MT), meaning that the are sent *to* an Iridium modem. Mobile-Originated
//! messages are delivered either to an email address via MIME attachment, or directly to a given
//! IP address and port via TCP; this second method is called `DirectIP`.
//!
//! # Usage
//!
//! This is a simple library for reading mobile originated SBD messages from a stream, decoding
//! their headers and data payloads, and writing them back to a stream. This library does not
//! handle mobile terminated messages.
//!
//! MO messages can be read from a byte stream:
//!
//! ```
//! let mut file = std::fs::File::open("data/0-mo.sbd").unwrap();
//! let message = sbd::mo::Message::read_from(file).unwrap();
//! ```
//!
//! To receive MO messages via `DirectIP`, a server is provided.
//! This server will listen for incoming messages forever, storing them in a `Storage`:
//!
//! ```no_run
//! let storage = sbd::storage::FilesystemStorage::open("/var/iridium").unwrap();
//! let mut server = sbd::directip::Server::new("0.0.0.0:10800", storage);
//! server.serve_forever();
//! ```
//!
//! Most of the functionality of this library is exposed by a single executable, named `sbd`.  Use
//! the `sbd` executable to inspect raw sbd files stores on a filesystem, interrogate sbd files on a
//! filesystem, and start that forever-running server to receive Iridium SBD `DirectIP` messages.

#![deny(missing_copy_implementations, missing_debug_implementations, missing_docs, trivial_casts,
        trivial_numeric_casts, unsafe_code, unstable_features, unused_extern_crates,
        unused_import_braces, unused_qualifications)]

extern crate byteorder;
extern crate chrono;
#[macro_use]
extern crate log;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate serde_derive;
extern crate walkdir;

pub mod directip;
pub mod information_element;
pub mod mo;
pub mod storage;

/// Create-specific `Result`.
pub type Result<T> = std::result::Result<T, Error>;

quick_error! {
    /// Crate-specific errors
    #[derive(Debug)]
    pub enum Error {
        /// A wrapper around a `std::io::Error`.
        Io(err: std::io::Error) {
            from()
            cause(err)
            description(err.description())
            display("io error: {}", err)
        }
        /// Invalid protocol revision number.
        InvalidProtocolRevisionNumber(n: u8) {
            description("invalid protocol revision number")
            display("invalid protocol revision number: {}", n)
        }
        /// Missing mobile originated header.
        MissingMobileOriginatedHeader {
            description("the mobile originated header is missing")
            display("the mobile originated header is missing")
        }
        /// Missing mobile originated payload.
        MissingMobileOriginatedPayload {
            description("the mobile originated payload is missing")
            display("the mobile originated payload is missing")
        }
        /// We expected a directory, but this isn't one.
        ///
        /// TODO can this be a PathBuf?
        NotADirectory(s: std::ffi::OsString) {
            description("the os string is not a directory")
            display("this os string is not a directory: {}", s.to_string_lossy())
        }
        /// An oversized message.
        ///
        /// Oversized doesn't demand a size since we don't want to find out how much there really
        /// is.
        Oversized {
            description("the message is oversized")
            display("the message is oversized")
        }
        /// An undersized message.
        Undersized(size: usize) {
            description("the message is undersized")
            display("the message is undersized: {}", size)
        }
        /// Some information elements weren't handled during reading.
        ///
        /// This is bad, because we might not write those IEs back out.
        UnhandledInformationElements(ies: std::collections::HashMap<information_element::InformationElementType, information_element::InformationElement>) {
            description("some information elements are unhandled")
            display("did not handle these information elements: {:?}", ies)
        }
        /// Wrapper around `std::str::Utf8Error`.
        Utf8(err: std::str::Utf8Error) {
            from()
            cause(err)
            description(err.description())
            display("utf8 error: {}", err)
        }
        /// Wrapper around `walkdir::Error`.
        WalkDir(err: walkdir::Error) {
            from()
            cause(err)
            description(err.description())
            display("walkdir error: {}", err)
        }
    }
}
