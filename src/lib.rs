//! Parse and write Iridium Short Burst Data (SBD) messages.
//!
//! # Background
//!
//! Iridium is both a [satellite constellation](https://en.wikipedia.org/wiki/Iridium_satellite_constellation)
//! and a [company](https://en.wikipedia.org/wiki/Iridium_Communications) that provides satellite
//! communications. The Iridium network is used by phones, modems, and other communication devices.
//!
//! One mode of transmitting data over the Iridium network is via Short Burst Data (SBD) messages.
//! These messages carry a payload of some small number of bytes, usually less than one thousand.
//! Messages can be Mobile-Originated (MO), meaning that they are sent *from* an Iridium modem, or
//! Mobile-Terminated (MT), meaning that the are sent *to* an Iridium modem. Mobile-Originated
//! messages are delivered either to an email address via MIME attachment, or directly to a given
//! IP address and port via TCP; this second method is called DirectIP.
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
//! To receive MO messages via DirectIP, a server is provided.
//! This server will listen for incoming messages forever, storing them in a `Storage`:
//!
//! ```no_run
//! let storage = sbd::storage::FilesystemStorage::open("/var/iridium").unwrap();
//! let mut server = sbd::directip::Server::new("0.0.0.0:10800", storage);
//! server.serve_forever();
//! ```
//!
//! Most of the functionality of this library is exposed by a single executable, named `sbd`.  Use
//! the `sbd` executable to inspect raw sbd files stores on a filesystem, discover sbd files on a
//! filesystem, and start that forever-running server to receive Iridium SBD DirectIP messages.

#![deny(missing_copy_implementations, missing_debug_implementations, missing_docs, trivial_casts,
        trivial_numeric_casts, unsafe_code, unstable_features, unused_extern_crates,
        unused_import_braces, unused_qualifications)]

pub mod directip;
pub mod error;
mod information_element;
pub mod mo;
pub mod storage;

pub use error::Error;

extern crate byteorder;
extern crate chrono;
#[macro_use]
extern crate log;
extern crate rustc_serialize;

/// Create-specific `Result`.
pub type Result<T> = std::result::Result<T, Error>;
