//! Error module.

use information_element::{InformationElement, InformationElementType};
use std;
use std::collections::HashMap;
use std::ffi::OsString;
use walkdir;

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
        /// Invalid IMEI number.
        InvalidImei {
            description("invalid imei number")
            display("invalid imei number")
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
        NotADirectory(s: OsString) {
            description("the os string is not a directory")
            display("this os string is not a directory: {}", s.to_string_lossy())
        }
        /// An oversized message.
        ///
        /// Oversized doesn't demand a size since we don't want to find out how much there really is.
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
        UnhandledInformationElements(ies: HashMap<InformationElementType, InformationElement>) {
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
