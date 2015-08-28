//! Generic message handling.
//!
//! This module provides the ability to read SBD messages from and write SBD messages to streams.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use byteorder::{ReadBytesExt, BigEndian};

use {Error, Result};
use information_element::InformationElement;

/// An SBD message.
#[derive(Debug, Default)]
pub struct Message {
    protocol_revision_number: u8,
    overall_message_length: u16,
    information_elements: Vec<InformationElement>,
}

impl Message {

    /// Reads in a message from a file path.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd::message::Message;
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Message> {
        let file = try!(File::open(path));
        Message::read_from(file)
    }

    /// Reads in a message from an object that implements `Read`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::File;
    /// use sbd::message::Message;
    /// let mut file = File::open("data/0-mo.sbd").unwrap();
    /// let message = Message::read_from(file).unwrap();
    pub fn read_from<R: Read>(mut readable: R) -> Result<Message> {
        let mut message: Message = Default::default();
        message.protocol_revision_number = try!(readable.read_u8());
        if message.protocol_revision_number != 1 {
            return Err(Error::InvalidProtocolRevisionNumber(message.protocol_revision_number));
        }
        Ok(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;

    #[test]
    fn from_path() {
        Message::from_path("data/0-mo.sbd").unwrap();
    }

    #[test]
    fn from_read() {
        let file = File::open("data/0-mo.sbd").unwrap();
        Message::read_from(file).unwrap();
    }

    #[test]
    fn from_path_that_doesnt_exist() {
        assert!(Message::from_path("notafile.sbd").is_err());
    }

    #[test]
    fn from_path_that_is_not_an_sbd_message() {
        assert!(Message::from_path("data/1-invalid.sbd").is_err());
    }
}
