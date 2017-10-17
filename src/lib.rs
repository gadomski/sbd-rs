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
#![recursion_limit="128"]

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
        /// Invalid information element identifier.
        InvalidInformationElementIdentifier(n: u8) {
            description("invalid information element identifier")
            display("invalid information element identifier: {}", n)
        }
        /// The timestamp is negative, but only positive ones are supported.
        NegativeTimestamp(timestamp: i64) {
            description("only positive timestamps are allowed in mo messages")
            display("negative timestamp: {}", timestamp)
        }
        /// No header on a MO message.
        NoHeader {
            description("no header on a mo message")
        }
        /// No payload on a MO message.
        NoPayload {
            description("no payload on a mo message")
        }
        /// We expected a directory, but this isn't one.
        ///
        /// TODO can this be a PathBuf?
        NotADirectory(s: std::ffi::OsString) {
            description("the os string is not a directory")
            display("this os string is not a directory: {}", s.to_string_lossy())
        }
        /// The overall message length is too long.
        OverallMessageLength(len: usize) {
            description("the overall message length is too long")
            display("the overall message length is too long: {}", len)
        }
        /// The payload is too long.
        PayloadTooLong(len: usize) {
            description("the mo payload is too long")
            display("the payload is too long: {}", len)
        }
        /// Two headers in an MO message.
        TwoHeaders {
            description("two headers in a MO message")
        }
        /// Two payloads in an MO message.
        TwoPayloads {
            description("two payloads in a MO message")
        }
        /// Wrapper around `std::str::Utf8Error`.
        Utf8(err: std::str::Utf8Error) {
            from()
            cause(err)
            description(err.description())
            display("utf8 error: {}", err)
        }
        /// The session status is unknown.
        UnknownSessionStatus(n: u8) {
            description("unknown session status")
            display("uknown session status code: {}", n)
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
